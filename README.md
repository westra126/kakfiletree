# kakfiletree

A clickable, filterable file tree for [Kakoune](https://kakoune.org) — built in Rust with [ratatui](https://ratatui.rs).

Opens in a tmux split to the left. Double-click a file to open it in Kakoune.

## Features

- **File tree** with expandable/collapsible directories
- **Filter** by substring (`/`)
- **Double-click** to open files in Kakoune
- **Git status colors** (modified, staged, untracked)
- **File operations**: create, delete, rename, copy
- **Mouse support**: click to select, double-click to open, scroll to navigate

## Installation

### With plug.kak

Add to your `kakrc`:

```kak
plug "westra126/kakfiletree" do %{
    cargo build --release
}

Then run `:plug-install` in Kakoune. The binary stays inside the plugin directory.

### Manual

```bash
git clone https://github.com/westra126/kakfiletree ~/Proyectos/kakfiletree
cd ~/Proyectos/kakfiletree
cargo build --release
# Binary at target/release/kakfiletree — stays inside the plugin dir
```

Then add to your `kakrc`:

```kak
source "~/Proyectos/kakfiletree/rc/kakfiletree.kak"
```

## Usage

From Kakoune inside tmux, run `:kakfiletree` to open the tree in a left split.

| Key | Action |
|---|---|
| `j`/`k` / `↑↓` | Navigate |
| `Enter` / `l` | Toggle expand/collapse (dir) or open file |
| `h` / `←` | Collapse dir or go to parent |
| `Tab` | Toggle expand/collapse |
| `/` | Filter |
| `Esc` | Cancel filter |
| `n` | New file |
| `N` | New directory |
| `d` / `Del` | Delete (confirm with `y`) |
| `r` | Rename |
| `y` | Yank path |
| `p` | Copy/paste |
| `.` | Toggle hidden files |
| `R` | Refresh tree |
| `?` | Show keybindings |
| `q` | Quit |

## Options

```kak
set-option global kakfiletree_width "40"   # split width in columns
```

## Requirements

- [Rust](https://rustup.rs) toolchain
- [tmux](https://tmux.org)
- [Kakoune](https://kakoune.org)
