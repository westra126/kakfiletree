use std::path::{Path, PathBuf};

pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub children: Vec<TreeNode>,
}

pub struct FlatItem {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub depth: usize,
}

impl TreeNode {
    pub fn new(path: &Path) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        let is_dir = path.is_dir();
        let children = if is_dir {
            Self::read_children(path)
        } else {
            vec![]
        };
        TreeNode {
            name,
            path: path.to_path_buf(),
            is_dir,
            is_expanded: true,
            children,
        }
    }

    fn read_children(dir: &Path) -> Vec<TreeNode> {
        let mut entries: Vec<_> = match std::fs::read_dir(dir) {
            Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
            Err(_) => return vec![],
        };
        entries.sort_by(|a, b| {
            let ad = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let bd = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
            bd.cmp(&ad).then_with(|| {
                a.file_name()
                    .to_string_lossy()
                    .to_lowercase()
                    .cmp(&b.file_name().to_string_lossy().to_lowercase())
            })
        });
        entries
            .iter()
            .map(|e| {
                let path = e.path();
                let name = e.file_name().to_string_lossy().to_string();
                let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let children = if is_dir {
                    Self::read_children(&path)
                } else {
                    vec![]
                };
                TreeNode {
                    name,
                    path,
                    is_dir,
                    is_expanded: false,
                    children,
                }
            })
            .collect()
    }

    pub fn expand_to(&mut self, target: &Path) {
        if self.is_dir && target.starts_with(&self.path) {
            self.is_expanded = true;
            for child in &mut self.children {
                child.expand_to(target);
            }
        }
    }

    pub fn refresh(&mut self) {
        if self.is_dir {
            let expanded = self.collect_expanded();
            self.children = Self::read_children(&self.path);
            self.restore_expanded(&expanded);
        }
    }

    fn collect_expanded(&self) -> Vec<PathBuf> {
        let mut r = vec![];
        if self.is_expanded {
            r.push(self.path.clone());
        }
        for c in &self.children {
            r.extend(c.collect_expanded());
        }
        r
    }

    fn restore_expanded(&mut self, expanded: &[PathBuf]) {
        self.is_expanded = expanded.contains(&self.path);
        for c in &mut self.children {
            c.restore_expanded(expanded);
        }
    }

    pub fn toggle(&mut self, target: &Path) -> bool {
        if self.path == target && self.is_dir {
            self.is_expanded = !self.is_expanded;
            return true;
        }
        for c in &mut self.children {
            if c.toggle(target) {
                return true;
            }
        }
        false
    }

    pub fn flatten(&self, show_hidden: bool, filter: &str) -> Vec<FlatItem> {
        let mut r = vec![];
        for c in &self.children {
            c.flatten_rec(0, show_hidden, filter, &mut r);
        }
        r
    }

    fn flatten_rec(&self, depth: usize, show_hidden: bool, filter: &str, r: &mut Vec<FlatItem>) {
        if !show_hidden && self.name.starts_with('.') {
            return;
        }
        if !filter.is_empty() {
            let nm = self.name.to_lowercase().contains(&filter.to_lowercase());
            let dm = self.has_match(filter, show_hidden);
            if !nm && !dm {
                return;
            }
        }
        let auto_expand = !filter.is_empty() && self.has_match(filter, show_hidden);
        let expanded = self.is_expanded || auto_expand;
        r.push(FlatItem {
            name: self.name.clone(),
            path: self.path.clone(),
            is_dir: self.is_dir,
            is_expanded: expanded,
            depth,
        });
        if self.is_dir && expanded {
            for c in &self.children {
                c.flatten_rec(depth + 1, show_hidden, filter, r);
            }
        }
    }

    fn has_match(&self, filter: &str, show_hidden: bool) -> bool {
        for c in &self.children {
            if !show_hidden && c.name.starts_with('.') {
                continue;
            }
            if c.name.to_lowercase().contains(&filter.to_lowercase()) {
                return true;
            }
            if c.is_dir && c.has_match(filter, show_hidden) {
                return true;
            }
        }
        false
    }
}
