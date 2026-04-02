use mlua::prelude::*;
use std::sync::Arc;

/// A plugin-registered action that appears in the context menu
#[derive(Clone)]
pub struct PluginAction {
    pub name: String,
    pub label: String,
    #[allow(dead_code)]
    pub icon: Option<String>,
    /// Registry key is not Clone, so wrap in Arc for sharing
    pub callback_key: Option<Arc<LuaRegistryKey>>,
}
