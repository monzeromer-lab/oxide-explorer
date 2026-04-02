use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingConfig {
    pub vim_mode: bool,
    pub custom_bindings: HashMap<String, String>,
}

impl Default for KeybindingConfig {
    fn default() -> Self {
        Self {
            vim_mode: false,
            custom_bindings: HashMap::new(),
        }
    }
}

impl KeybindingConfig {
    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("oxide-explorer")
            .join("keybindings.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        match fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(contents) = toml::to_string_pretty(self) {
            let _ = fs::write(&path, contents);
        }
    }

    /// Returns the standard Vim keybindings for navigation and selection.
    pub fn vim_shortcuts() -> Vec<(&'static str, &'static str)> {
        vec![
            // Navigation
            ("h", "win.go-up"),           // parent directory (like vim's h = left)
            ("l", "win.activate-item"),    // enter directory / open file
            ("j", "win.select-next"),      // next item
            ("k", "win.select-prev"),      // previous item
            ("g", "win.go-first"),         // first item
            ("<Shift>g", "win.go-last"),   // last item
            // Actions
            ("d", "win.trash"),            // delete (to trash)
            ("y", "win.copy"),             // yank (copy)
            ("p", "win.paste"),            // paste
            ("r", "win.rename"),           // rename
            ("slash", "win.toggle-filter"),// / to search
            ("period", "win.toggle-hidden"),// . to toggle hidden
            ("t", "win.new-tab"),          // new tab
            ("x", "win.cut"),             // cut
            ("o", "win.open-terminal"),   // open terminal
            ("i", "win.properties"),      // info/properties
            ("<Shift>question", "win.show-shortcuts"), // ? for help
        ]
    }
}
