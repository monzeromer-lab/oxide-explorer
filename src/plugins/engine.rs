use mlua::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::api::PluginAction;

/// Manages loading and executing Lua plugins
pub struct PluginEngine {
    lua: Lua,
    plugins_dir: PathBuf,
    actions: Arc<std::sync::Mutex<Vec<PluginAction>>>,
}

impl PluginEngine {
    pub fn new() -> Self {
        let lua = Lua::new();
        let plugins_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("oxide-explorer")
            .join("plugins");

        let actions = Arc::new(std::sync::Mutex::new(Vec::new()));

        // Create plugins directory if it doesn't exist
        let _ = std::fs::create_dir_all(&plugins_dir);

        // Create example plugin if none exist
        let example_path = plugins_dir.join("example.lua");
        if !example_path.exists() {
            let _ = std::fs::write(&example_path, EXAMPLE_PLUGIN);
        }

        let mut engine = Self { lua, plugins_dir, actions };
        engine.setup_api();
        engine
    }

    fn setup_api(&mut self) {
        // Set up the `oxide` table
        let globals = self.lua.globals();
        let oxide_table = self.lua.create_table().unwrap();

        // oxide.register_action(name, label, callback)
        let actions_ref = self.actions.clone();
        let register_action = self.lua.create_function(move |lua, (name, label, icon, callback): (String, String, Option<String>, LuaFunction)| {
            // Store the callback as a registry key
            let key = lua.create_registry_value(callback)?;
            let mut actions = actions_ref.lock().unwrap();
            actions.push(PluginAction {
                name,
                label,
                icon,
                callback_key: Some(Arc::new(key)),
            });
            Ok(())
        }).unwrap();
        oxide_table.set("register_action", register_action).unwrap();

        // oxide.log(message)
        let log_fn = self.lua.create_function(|_, msg: String| {
            log::info!("[plugin] {msg}");
            Ok(())
        }).unwrap();
        oxide_table.set("log", log_fn).unwrap();

        // oxide.exec(command) — run a shell command
        let exec_fn = self.lua.create_function(|_, cmd: String| {
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .output()
                .map_err(|e| mlua::Error::external(e))?;
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(stdout)
        }).unwrap();
        oxide_table.set("exec", exec_fn).unwrap();

        // oxide.get_selection() will be set dynamically before calling actions
        // For now, placeholder
        let get_selection = self.lua.create_function(|_, ()| {
            Ok(Vec::<String>::new())
        }).unwrap();
        oxide_table.set("get_selection", get_selection).unwrap();

        // oxide.get_current_dir() — placeholder, set dynamically
        let get_cwd = self.lua.create_function(|_, ()| {
            Ok(String::new())
        }).unwrap();
        oxide_table.set("get_current_dir", get_cwd).unwrap();

        globals.set("oxide", oxide_table).unwrap();
    }

    /// Load all .lua files from the plugins directory
    pub fn load_plugins(&mut self) {
        let dir = self.plugins_dir.clone();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "lua") {
                    if let Err(e) = self.load_plugin(&path) {
                        log::error!("Failed to load plugin {:?}: {e}", path.file_name());
                    } else {
                        log::info!("Loaded plugin: {:?}", path.file_name());
                    }
                }
            }
        }
    }

    fn load_plugin(&mut self, path: &Path) -> Result<(), mlua::Error> {
        let code = std::fs::read_to_string(path)
            .map_err(|e| mlua::Error::external(e))?;
        self.lua.load(&code).set_name(path.to_string_lossy()).exec()?;
        Ok(())
    }

    /// Get all registered plugin actions
    pub fn get_actions(&self) -> Vec<PluginAction> {
        self.actions.lock().unwrap().clone()
    }

    /// Execute a plugin action by name, providing context
    pub fn execute_action(
        &self,
        action_name: &str,
        selected_files: &[PathBuf],
        current_dir: &Path,
    ) -> Result<(), mlua::Error> {
        // Update context functions
        let globals = self.lua.globals();
        let oxide: LuaTable = globals.get("oxide")?;

        // Update get_selection
        let files: Vec<String> = selected_files.iter().map(|p| p.to_string_lossy().into_owned()).collect();
        let files_clone = files.clone();
        let get_sel = self.lua.create_function(move |_, ()| {
            Ok(files_clone.clone())
        })?;
        oxide.set("get_selection", get_sel)?;

        // Update get_current_dir
        let cwd = current_dir.to_string_lossy().into_owned();
        let get_cwd = self.lua.create_function(move |_, ()| {
            Ok(cwd.clone())
        })?;
        oxide.set("get_current_dir", get_cwd)?;

        // Find and call the action
        let actions = self.actions.lock().unwrap();
        if let Some(action) = actions.iter().find(|a| a.name == action_name) {
            if let Some(ref key) = action.callback_key {
                let callback: LuaFunction = self.lua.registry_value(key)?;
                callback.call::<()>(())?;
            }
        }

        Ok(())
    }
}

const EXAMPLE_PLUGIN: &str = r#"-- Example Oxide Explorer Plugin
-- Place .lua files in ~/.config/oxide-explorer/plugins/

-- Register a custom action that appears in the context menu
oxide.register_action(
    "word_count",           -- unique name
    "Count Lines",          -- display label in menu
    "text-x-generic",      -- icon name (optional, can be nil)
    function()
        local files = oxide.get_selection()
        if #files == 0 then
            oxide.log("No files selected")
            return
        end
        local total = 0
        for _, file in ipairs(files) do
            local result = oxide.exec("wc -l < '" .. file .. "'")
            local count = tonumber(result) or 0
            total = total + count
            oxide.log(file .. ": " .. count .. " lines")
        end
        oxide.log("Total: " .. total .. " lines")
    end
)
"#;
