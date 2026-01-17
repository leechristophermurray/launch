use crate::domain::ports::IMacroRepository;
use crate::infrastructure::services::settings_store::SettingsStore;
use crate::domain::model::Macro;
use std::sync::Arc;

pub struct JsonMacroAdapter {
    store: Arc<SettingsStore>,
}

impl JsonMacroAdapter {
    pub fn new(store: Arc<SettingsStore>) -> Self {
        Self { store }
    }
}

impl IMacroRepository for JsonMacroAdapter {
    fn get(&self, name: &str) -> Option<Macro> {
        self.store.get_macros().into_iter().find(|m| m.name == name)
    }

    fn get_all(&self) -> Vec<Macro> {
        self.store.get_macros()
    }
    
    fn add(&self, mac: Macro) -> Result<(), String> {
        self.store.add_macro(mac)
    }
    
    fn remove(&self, name: &str) -> Result<(), String> {
        self.store.remove_macro(name)
    }
}
