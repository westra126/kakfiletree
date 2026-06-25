use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{App, Mode, PromptKind};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Min(1),   // tree + left margin
            Constraint::Length(1), // status bar
        ])
        .split(area);

    // Title: current root path
    let title = Paragraph::new(app.root.path.display().to_string())
        .style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(title, vchunks[0]);

    // Tree area with left margin
    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(vchunks[1]);

    let tree_area = hchunks[1];
    app.tree_start_y = tree_area.y;
    app.visible_height = tree_area.height as usize;
    app.update_scroll();

    draw_tree(frame, app, tree_area);
    draw_status_bar(frame, app, vchunks[2]);

    if matches!(app.mode, Mode::Help) {
        draw_help_popup(frame, area);
    }
}

fn draw_tree(frame: &mut Frame, app: &mut App, area: Rect) {
    if app.flat.is_empty() {
        let text = if app.filter_text.is_empty() {
            "Empty directory"
        } else {
            "No matches"
        };
        let paragraph = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .flat
        .iter()
        .map(|item| {
            let indent = "  ".repeat(item.depth);
            let icon = if item.is_dir {
                if item.is_expanded {
                    "▾ "
                } else {
                    "▸ "
                }
            } else {
                "  "
            };
            let style = if item.is_dir {
                let mut s = Style::default().add_modifier(Modifier::BOLD);
                if app.git_dirty_dirs.contains(&item.path) {
                    s = s.fg(Color::Yellow);
                }
                s
            } else if let Some(status) = app.git_statuses.get(&item.path) {
                Style::default().fg(git_color(status))
            } else {
                Style::default()
            };
            let text = format!("{}{}{}", indent, icon, item.name);
            ListItem::new(Line::from(text).style(style))
        })
        .collect();

    let list = List::new(items).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = ListState::default();
    state.select(Some(app.selected));

    frame.render_stateful_widget(list, area, &mut state);
    app.scroll_offset = state.offset();
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (text, style) = match &app.mode {
        Mode::Normal => {
            if let Some(ref msg) = app.message {
                (msg.clone(), Style::default().fg(Color::Green))
            } else {
                (String::new(), Style::default())
            }
        }
        Mode::Filter => (
            format!("Filter: {}█", app.filter_text),
            Style::default().fg(Color::Yellow),
        ),
        Mode::Help => (
            "Press any key to close".to_string(),
            Style::default().fg(Color::Cyan),
        ),
        Mode::Prompt(kind) => match kind {
            PromptKind::NewFile => (
                format!("New file: {}█", app.prompt_text),
                Style::default().fg(Color::Cyan),
            ),
            PromptKind::NewDir => (
                format!("New dir: {}█", app.prompt_text),
                Style::default().fg(Color::Cyan),
            ),
            PromptKind::Rename(old) => (
                format!(
                    "Rename: {} → {}█",
                    old.file_name().unwrap_or_default().to_string_lossy(),
                    app.prompt_text
                ),
                Style::default().fg(Color::Cyan),
            ),
            PromptKind::Delete(path) => (
                format!(
                    "Delete {}? y/n",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ),
                Style::default().fg(Color::Red),
            ),
            PromptKind::Copy(_) => (
                format!("Copy to: {}█", app.prompt_text),
                Style::default().fg(Color::Cyan),
            ),
        },
    };

    let paragraph = Paragraph::new(text).style(style);
    frame.render_widget(paragraph, area);
}

fn draw_help_popup(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled(
            " Keybindings ",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Line::from(""),
        Line::from(" j / ↓        Move down"),
        Line::from(" k / ↑        Move up"),
        Line::from(" l / → / Enter  Expand dir or open file"),
        Line::from(" h / ←        Collapse dir or go to parent"),
        Line::from(" Tab          Toggle expand/collapse"),
        Line::from(" /            Filter (substring)"),
        Line::from(" Esc          Cancel filter"),
        Line::from(" n            New file"),
        Line::from(" N            New directory"),
        Line::from(" d / Del      Delete (confirm with y)"),
        Line::from(" r            Rename"),
        Line::from(" y            Yank path"),
        Line::from(" p            Copy/paste file"),
        Line::from(" .            Toggle hidden files"),
        Line::from(" R            Refresh tree"),
        Line::from(" ?            Show this help"),
        Line::from(" q / Ctrl-c   Quit"),
    ];

    let popup = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::default().fg(Color::White));

    let popup_area = centered_rect(area, 46, 22);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup, popup_area);
}

fn centered_rect(r: Rect, width: u16, height: u16) -> Rect {
    let popup_width = width.min(r.width);
    let popup_height = height.min(r.height);
    let x = r.x + (r.width.saturating_sub(popup_width) / 2);
    let y = r.y + (r.height.saturating_sub(popup_height) / 2);
    Rect::new(x, y, popup_width, popup_height)
}

fn git_color(status: &str) -> Color {
    match status.chars().next() {
        Some('?') => Color::Red,
        Some('A') => Color::Green,
        Some('M') => Color::Yellow,
        Some('D') => Color::Red,
        Some('R') => Color::Cyan,
        _ => {
            if status.contains('M') {
                Color::Yellow
            } else if status.contains('A') {
                Color::Green
            } else if status.contains('D') {
                Color::Red
            } else {
                Color::White
            }
        }
    }
}
