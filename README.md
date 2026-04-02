# Oxide Explorer

A blazing-fast, power-user-centric file manager built with **Rust**, **GTK4**, and **libadwaita**. Combines the visual elegance of GNOME Files with the advanced productivity features of Directory Opus and Total Commander.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.94%2B-orange.svg)
![GTK4](https://img.shields.io/badge/GTK-4.12%2B-green.svg)

## Features

### Core File Management
- **Icon & Details views** with sortable columns (Name, Size, Date)
- **Breadcrumb navigation** with editable path bar (Ctrl+L)
- **Back/Forward/Up** navigation with full history
- **Copy, Cut, Paste, Rename, Delete** with async I/O
- **Move to Trash** with undo via toast notification
- **Drag and drop** file moving
- **File properties** dialog (size, permissions, owner, dates)

### Power User Features
- **Tabbed browsing** (Ctrl+T / Ctrl+W) with independent per-tab state
- **Dual-pane mode** (F3) — Commander-style split view with cross-pane copy/move (F6)
- **Embedded terminal** (F4) — VTE terminal synced to current directory
- **Miller columns** (F5) — macOS Finder-style cascading column navigation
- **Instant filter** (Ctrl+F) — type-ahead filtering
- **Vim-style keybindings** — optional h/j/k/l navigation (enable in Preferences)
- **Fully customizable shortcuts** — override any keybinding in Preferences

### Advanced Tooling
- **Advanced search** (Ctrl+Shift+F) — regex, wildcards, file content search
- **Batch renamer** — find/replace, prefix/suffix, numbering, extension change with live preview
- **Preview pane** (Space) — quick look for text, images, and file info
- **Archive management** — extract and compress zip, tar.gz, tar.xz, 7z, rar
- **File tags & colors** — assign color dots and text tags via xattr

### Network & Extensibility
- **Network drives** — connect to SMB, FTP, SFTP, WebDAV servers via GIO
- **Lua plugin system** — write custom context menu actions in Lua
- **Cloud sync status** — detect Nextcloud, Google Drive, Dropbox sync state
- **GIO content type detection** — proper system theme icons for all file types

### Sidebar
- **Places** — Home, Documents, Downloads, Desktop, Pictures, Music, Videos
- **File System** root entry
- **Mounted devices** — auto-detected via GIO VolumeMonitor
- **Bookmarks** (Ctrl+D) — persistent custom folder bookmarks
- **Trash** entry

### UI/UX
- **libadwaita** adaptive design with custom CSS styling
- **Toast notifications** for all operations
- **Loading spinner** for async directory loads
- **Empty state** view for empty directories
- **Error state** with helpful messages
- **Disk space indicator** in status bar
- **Selection counter** in status bar
- **User guide** (F1) with full feature documentation
- **About dialog** with release notes
- **Preferences** window with settings persistence

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| Ctrl+T | New tab |
| Ctrl+W | Close tab |
| Ctrl+C / Ctrl+X / Ctrl+V | Copy / Cut / Paste |
| Ctrl+Shift+C | Copy file path |
| Delete | Move to trash |
| F2 | Rename |
| Ctrl+A | Select all |
| Ctrl+H | Toggle hidden files |
| Ctrl+L | Edit location path |
| Ctrl+F | Filter files |
| Ctrl+D | Bookmark folder |
| Ctrl+Shift+F | Advanced search |
| Space | Toggle preview pane |
| F1 | User guide |
| F3 | Dual pane mode |
| F4 | Embedded terminal |
| F5 | Miller columns |
| F6 / Shift+F6 | Copy / Move to other pane |
| Tab | Switch pane focus |
| Alt+Left / Backspace | Go back |
| Alt+Right | Go forward |
| Alt+Up | Parent directory |
| Ctrl++ / Ctrl+- / Ctrl+0 | Zoom in / out / reset |
| Alt+Enter | File properties |
| Ctrl+Alt+T | Open external terminal |
| Ctrl+, | Preferences |

## Building from Source

### Prerequisites

```bash
# Ubuntu / Debian
sudo apt install libgtk-4-dev libadwaita-1-dev libvte-2.91-gtk4-dev

# Fedora
sudo dnf install gtk4-devel libadwaita-devel vte291-gtk4-devel

# Arch Linux
sudo pacman -S gtk4 libadwaita vte4
```

### Build & Run

```bash
# Clone
git clone https://github.com/oxide-explorer/oxide-explorer.git
cd oxide-explorer

# Build (includes terminal by default)
cargo build --release

# Run
cargo run --release

# Build without terminal (if VTE is not available)
cargo build --release --no-default-features
```

### Install from .deb (Debian/Ubuntu)

```bash
sudo dpkg -i oxide-explorer_0.4.0_amd64.deb
```

## Plugin System

Oxide Explorer supports Lua plugins. Place `.lua` files in `~/.config/oxide-explorer/plugins/`.

### Example Plugin

```lua
-- Count lines in selected files
oxide.register_action(
    "line_count",        -- unique name
    "Count Lines",       -- menu label
    nil,                 -- icon (optional)
    function()
        local files = oxide.get_selection()
        for _, file in ipairs(files) do
            local result = oxide.exec("wc -l < '" .. file .. "'")
            oxide.log(file .. ": " .. result .. " lines")
        end
    end
)
```

### Plugin API

| Function | Description |
|---|---|
| `oxide.register_action(name, label, icon, callback)` | Add action to context menu |
| `oxide.get_selection()` | Get list of selected file paths |
| `oxide.get_current_dir()` | Get current directory path |
| `oxide.exec(command)` | Run shell command, return stdout |
| `oxide.log(message)` | Log to application console |

## Configuration

Settings are stored in `~/.config/oxide-explorer/`:

| File | Purpose |
|---|---|
| `config.toml` | General settings (view mode, icon size, sort, etc.) |
| `keybindings.toml` | Custom keyboard shortcuts and vim mode toggle |
| `bookmarks.txt` | Sidebar bookmarks |
| `recent_connections.txt` | Network connection history |
| `plugins/` | Lua plugin directory |

## Tech Stack

- **Rust** — memory safety, performance, safe concurrency
- **GTK4** — hardware-accelerated rendering, virtualized list views
- **libadwaita** — modern GNOME UI patterns, adaptive layouts
- **Tokio** — async file I/O on background threads
- **GIO** — file monitoring, trash, network mounts, content type detection
- **VTE** — embedded terminal emulator
- **mlua** — Lua 5.4 plugin runtime (vendored)

## License

MIT License - Copyright (c) 2024-2026 Monzer Omer <monzer.a.omer@gmail.com>

See [LICENSE](LICENSE) for details.
