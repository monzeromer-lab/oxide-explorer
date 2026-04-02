use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_view: ViewMode,
    pub show_hidden_files: bool,
    pub confirm_trash: bool,
    pub icon_size: i32,
    pub sort_by: SortColumn,
    pub sort_ascending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    Icon,
    Details,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortColumn {
    Name,
    Size,
    Date,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_view: ViewMode::Icon,
            show_hidden_files: false,
            confirm_trash: true,
            icon_size: 48,
            sort_by: SortColumn::Name,
            sort_ascending: true,
        }
    }
}

impl Settings {
    fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("oxide-explorer");
        config_dir.join("config.toml")
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
}
