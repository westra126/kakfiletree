use std::io;
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

mod actions;
mod app;
mod kak;
mod tree;
mod ui;

use app::{App, Mode, PromptKind};
use kak::KakClient;

#[derive(Parser)]
#[command(name = "kakfiletree", about = "File tree for Kakoune")]
struct Cli {
    #[arg(long)]
    session: String,
    #[arg(long)]
    client: String,
    #[arg(long, default_value = ".")]
    root: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let kak = KakClient {
        session: cli.session,
        client: cli.client,
    };

    let root = std::fs::canonicalize(&cli.root).unwrap_or(cli.root);
    let mut app = App::new(&root, kak);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::event::EnableMouseCapture,
        crossterm::event::EnableFocusChange
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if matches!(app.mode, Mode::Normal) {
                        app.message = None;
                    }
                    handle_key(&mut app, key);
                }
                Event::Mouse(mouse) => handle_mouse(&mut app, mouse),
                Event::FocusGained => {
                    app.refresh_git_status();
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    Ok(())
}

fn handle_key(app: &mut App, key: event::KeyEvent) {
    match &app.mode {
        Mode::Normal => handle_normal_key(app, key),
        Mode::Filter => handle_filter_key(app, key),
        Mode::Help => app.mode = Mode::Normal,
        Mode::Prompt(_) => handle_prompt_key(app, key),
    }
}

fn handle_normal_key(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true
        }
        KeyCode::Char('j') | KeyCode::Down => app.move_down(),
        KeyCode::Char('k') | KeyCode::Up => app.move_up(),
        KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => app.open_selected(),
        KeyCode::Char('h') | KeyCode::Left => {
            if let Some(item) = app.flat.get(app.selected) {
                if item.is_dir && item.is_expanded {
                    app.toggle_expand();
                } else {
                    app.go_to_parent();
                }
            }
        }
        KeyCode::Tab => app.toggle_expand(),
        KeyCode::Char('/') => app.start_filter(),
        KeyCode::Char('n') => app.start_prompt(PromptKind::NewFile),
        KeyCode::Char('N') => app.start_prompt(PromptKind::NewDir),
        KeyCode::Char('d') | KeyCode::Delete => {
            if let Some(item) = app.flat.get(app.selected) {
                let path = item.path.clone();
                app.start_prompt(PromptKind::Delete(path));
            }
        }
        KeyCode::Char('r') => {
            if let Some(item) = app.flat.get(app.selected) {
                let path = item.path.clone();
                app.start_prompt(PromptKind::Rename(path));
            }
        }
        KeyCode::Char('y') => app.yank_path(),
        KeyCode::Char('p') => {
            if let Some(item) = app.flat.get(app.selected) {
                let path = item.path.clone();
                app.start_prompt(PromptKind::Copy(path));
            }
        }
        KeyCode::Char('.') => app.toggle_hidden(),
        KeyCode::Char('?') => app.mode = Mode::Help,
        KeyCode::Char('R') => app.refresh_tree(),
        _ => {}
    }
}

fn handle_filter_key(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Esc => app.cancel_filter(),
        KeyCode::Enter => app.confirm_filter(),
        KeyCode::Backspace => {
            app.filter_text.pop();
            app.apply_filter();
        }
        KeyCode::Char(c) => {
            app.filter_text.push(c);
            app.apply_filter();
        }
        _ => {}
    }
}

fn handle_prompt_key(app: &mut App, key: event::KeyEvent) {
    let is_delete = matches!(&app.mode, Mode::Prompt(PromptKind::Delete(_)));

    if is_delete {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                app.prompt_text = "y".to_string();
                app.confirm_prompt();
            }
            _ => app.cancel_prompt(),
        }
        return;
    }

    match key.code {
        KeyCode::Esc => app.cancel_prompt(),
        KeyCode::Enter => app.confirm_prompt(),
        KeyCode::Backspace => {
            app.prompt_text.pop();
        }
        KeyCode::Char(c) => {
            app.prompt_text.push(c);
        }
        _ => {}
    }
}

fn handle_mouse(app: &mut App, mouse: event::MouseEvent) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if matches!(app.mode, Mode::Normal) && !app.flat.is_empty() {
                let row = mouse.row.saturating_sub(app.tree_start_y) as usize;
                let item_row = row + app.scroll_offset;
                if item_row < app.flat.len() {
                    app.handle_click(item_row);
                }
            }
        }
        MouseEventKind::ScrollUp => app.move_up(),
        MouseEventKind::ScrollDown => app.move_down(),
        _ => {}
    }
}
