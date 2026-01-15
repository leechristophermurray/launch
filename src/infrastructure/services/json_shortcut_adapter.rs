use crate::interface::ports::IShortcutRepository;
use crate::infrastructure::services::settings_store::SettingsStore;
use std::sync::Arc;
use std::collections::HashMap;

pub struct JsonShortcutAdapter {
    store: Arc<SettingsStore>,
}

impl JsonShortcutAdapter {
    pub fn new(store: Arc<SettingsStore>) -> Self {
        Self { store }
    }
}

impl IShortcutRepository for JsonShortcutAdapter {
    fn get(&self, key: &str) -> Option<String> {
        self.store.get_shortcuts().get(key).cloned()
    }

    fn get_all(&self) -> HashMap<String, String> {
        self.store.get_shortcuts()
    }
    
    fn add(&self, key: String, cmd: String) -> Result<(), String> {
        self.store.add_shortcut(key, cmd)
    }
    
    fn remove(&self, key: &str) -> Result<(), String> {
        self.store.remove_shortcut(key)
    }
}
