use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

use crate::actions;
use crate::kak::KakClient;
use crate::tree::{FlatItem, TreeNode};

pub enum Mode {
    Normal,
    Filter,
    Help,
    Prompt(PromptKind),
}

pub enum PromptKind {
    NewFile,
    NewDir,
    Rename(PathBuf),
    Delete(PathBuf),
    Copy(PathBuf),
}

pub struct App {
    pub root: TreeNode,
    pub flat: Vec<FlatItem>,
    pub selected: usize,
    pub mode: Mode,
    pub filter_text: String,
    pub prompt_text: String,
    pub show_hidden: bool,
    pub kak: KakClient,
    pub scroll_offset: usize,
    pub visible_height: usize,
    pub tree_start_y: u16,
    pub last_click: Option<(Instant, usize)>,
    pub message: Option<String>,
    pub git_statuses: HashMap<PathBuf, String>,
    pub git_dirty_dirs: HashSet<PathBuf>,
    pub should_quit: bool,
}

impl App {
    pub fn new(root_path: &std::path::Path, kak: KakClient) -> Self {
        let root = TreeNode::new(root_path);
        let flat = root.flatten(false, "");
        let mut app = App {
            root,
            flat,
            selected: 0,
            mode: Mode::Normal,
            filter_text: String::new(),
            prompt_text: String::new(),
            show_hidden: false,
            kak,
            scroll_offset: 0,
            visible_height: 0,
            tree_start_y: 0,
            last_click: None,
            message: None,
            git_statuses: HashMap::new(),
            git_dirty_dirs: HashSet::new(),
            should_quit: false,
        };
        app.refresh_git_status();
        app
    }

    pub fn refresh_flat(&mut self) {
        let old_path = self.flat.get(self.selected).map(|item| item.path.clone());
        self.flat = self.root.flatten(self.show_hidden, &self.filter_text);
        if let Some(path) = old_path {
            if let Some(idx) = self.flat.iter().position(|item| item.path == path) {
                self.selected = idx;
            } else if self.selected >= self.flat.len() && !self.flat.is_empty() {
                self.selected = self.flat.len() - 1;
            }
        } else if self.selected >= self.flat.len() && !self.flat.is_empty() {
            self.selected = self.flat.len() - 1;
        }
    }

    pub fn update_scroll(&mut self) {
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
        if self.visible_height > 0 && self.selected >= self.scroll_offset + self.visible_height {
            self.scroll_offset = self.selected - self.visible_height + 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.flat.is_empty() && self.selected + 1 < self.flat.len() {
            self.selected += 1;
        }
    }

    pub fn toggle_expand(&mut self) {
        if let Some(item) = self.flat.get(self.selected) {
            if item.is_dir {
                let path = item.path.clone();
                self.root.toggle(&path);
                self.refresh_flat();
            }
        }
    }

    pub fn open_selected(&mut self) {
        if let Some(item) = self.flat.get(self.selected) {
            if item.is_dir {
                self.toggle_expand();
            } else {
                let path = item.path.clone();
                self.kak.open_file(&path);
            }
        }
    }

    pub fn go_to_parent(&mut self) {
        if let Some(item) = self.flat.get(self.selected) {
            if let Some(parent_path) = item.path.parent().map(|p| p.to_path_buf()) {
                if let Some(idx) = self.flat.iter().position(|item| item.path == parent_path) {
                    self.selected = idx;
                }
            }
        }
    }

    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.refresh_flat();
    }

    pub fn refresh_tree(&mut self) {
        self.root.refresh();
        self.refresh_git_status();
        self.refresh_flat();
        self.message = Some("Refreshed".to_string());
    }

    pub fn refresh_git_status(&mut self) {
        self.git_statuses.clear();
        self.git_dirty_dirs.clear();
        if let Ok(output) = std::process::Command::new("git")
            .args(["-C", &self.root.path.display().to_string(), "status", "--porcelain"])
            .output()
        {
            if let Ok(text) = String::from_utf8(output.stdout) {
                for line in text.lines() {
                    if let Some(status) = line.get(..2) {
                        let path_str = if status.contains('R') {
                            if let Some(arrow) = line.find(" -> ") {
                                &line[arrow + 4..]
                            } else {
                                &line[3..]
                            }
                        } else {
                            &line[3..]
                        };
                        let full_path = self.root.path.join(path_str);
                        self.git_statuses.insert(full_path.clone(), status.to_string());

                        let mut parent = full_path.parent();
                        while let Some(p) = parent {
                            if p.starts_with(&self.root.path) {
                                self.git_dirty_dirs.insert(p.to_path_buf());
                            } else {
                                break;
                            }
                            parent = p.parent();
                        }
                    }
                }
            }
        }
    }

    pub fn handle_click(&mut self, row: usize) {
        let now = Instant::now();
        if let Some((last_time, last_row)) = self.last_click {
            if row == last_row && now.duration_since(last_time).as_millis() < 400 {
                self.selected = row;
                self.open_selected();
                self.last_click = None;
                return;
            }
        }
        self.selected = row;
        self.last_click = Some((now, row));
    }

    pub fn start_filter(&mut self) {
        self.mode = Mode::Filter;
        self.filter_text.clear();
    }

    pub fn apply_filter(&mut self) {
        let text = self.filter_text.clone();
        self.filter_text = text.clone();
        self.selected = 0;
        self.refresh_flat();
    }

    pub fn cancel_filter(&mut self) {
        self.filter_text.clear();
        self.mode = Mode::Normal;
        self.refresh_flat();
    }

    pub fn confirm_filter(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn start_prompt(&mut self, kind: PromptKind) {
        self.prompt_text.clear();
        if let PromptKind::Rename(ref path) = kind {
            self.prompt_text = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
        }
        self.mode = Mode::Prompt(kind);
    }

    pub fn confirm_prompt(&mut self) {
        let mode = std::mem::replace(&mut self.mode, Mode::Normal);
        let mut target_path: Option<PathBuf> = None;

        match mode {
            Mode::Prompt(PromptKind::NewFile) => {
                if !self.prompt_text.is_empty() {
                    let base = self.current_dir();
                    let path = base.join(&self.prompt_text);
                    target_path = Some(path.clone());
                    match actions::create_file(&path) {
                        Ok(_) => {
                            self.message =
                                Some(format!("Created {}", self.prompt_text));
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                            target_path = None;
                        }
                    }
                }
            }
            Mode::Prompt(PromptKind::NewDir) => {
                if !self.prompt_text.is_empty() {
                    let base = self.current_dir();
                    let path = base.join(&self.prompt_text);
                    target_path = Some(path.clone());
                    match actions::create_dir(&path) {
                        Ok(_) => {
                            self.message =
                                Some(format!("Created dir {}", self.prompt_text));
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                            target_path = None;
                        }
                    }
                }
            }
            Mode::Prompt(PromptKind::Rename(old_path)) => {
                if !self.prompt_text.is_empty() {
                    let new_path = old_path.parent().unwrap().join(&self.prompt_text);
                    target_path = Some(new_path.clone());
                    match actions::rename(&old_path, &new_path) {
                        Ok(_) => {
                            self.message =
                                Some(format!("Renamed to {}", self.prompt_text));
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                            target_path = None;
                        }
                    }
                }
            }
            Mode::Prompt(PromptKind::Copy(src)) => {
                if !self.prompt_text.is_empty() {
                    let base = self.current_dir();
                    let dst = base.join(&self.prompt_text);
                    target_path = Some(dst.clone());
                    match actions::copy_path(&src, &dst) {
                        Ok(_) => {
                            self.message =
                                Some(format!("Copied to {}", self.prompt_text));
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                            target_path = None;
                        }
                    }
                }
            }
            Mode::Prompt(PromptKind::Delete(path)) => {
                if self.prompt_text.to_lowercase() == "y" {
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    match actions::delete(&path) {
                        Ok(_) => {
                            self.message = Some(format!("Deleted {}", name));
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                        }
                    }
                }
            }
            _ => {}
        }

        if let Some(ref path) = target_path {
            self.root.expand_to(path);
        }
        self.refresh_tree();
        if let Some(ref path) = target_path {
            if let Some(idx) = self.flat.iter().position(|item| item.path == *path) {
                self.selected = idx;
            }
        }
    }

    pub fn cancel_prompt(&mut self) {
        self.mode = Mode::Normal;
        self.prompt_text.clear();
    }

    fn current_dir(&self) -> PathBuf {
        if let Some(item) = self.flat.get(self.selected) {
            if item.is_dir {
                return item.path.clone();
            } else {
                return item.path.parent().unwrap_or(&item.path).to_path_buf();
            }
        }
        self.root.path.clone()
    }

    pub fn yank_path(&mut self) {
        if let Some(item) = self.flat.get(self.selected) {
            self.message = Some(format!("{}", item.path.display()));
        }
    }
}
