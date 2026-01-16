use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::domain::model::{Macro, MacroAction};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AppSettings {
    pub shortcuts: HashMap<String, String>,
    pub macros: Vec<MacroSerde>,
    #[serde(default = "default_model")]
    pub ai_model: String,
}

fn default_model() -> String {
    "llama3".to_string()
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MacroSerde {
    pub name: String,
    pub actions: Vec<MacroAction>,
}

impl From<MacroSerde> for Macro {
    fn from(m: MacroSerde) -> Self {
        Macro { name: m.name, actions: m.actions }
    }
}
impl From<Macro> for MacroSerde {
    fn from(m: Macro) -> Self {
        MacroSerde { name: m.name, actions: m.actions }
    }
}

pub struct SettingsStore {
    path: PathBuf,
    cache: Arc<Mutex<AppSettings>>,
}

impl SettingsStore {
    pub fn new() -> Self {
        let path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("launch")
            .join("settings.json");
            
        let store = Self {
            path: path.clone(),
            cache: Arc::new(Mutex::new(AppSettings::default())),
        };
        store.load();
        store
    }

    fn load(&self) {
        if let Ok(content) = fs::read_to_string(&self.path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                *self.cache.lock().unwrap() = settings;
                return;
            }
        }
        // If fail or not exist, ensure default
        // Optionally create dir
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let data = self.cache.lock().unwrap();
        let json = serde_json::to_string_pretty(&*data).map_err(|e| e.to_string())?;
        fs::write(&self.path, json).map_err(|e| e.to_string())
    }

    pub fn get_shortcuts(&self) -> HashMap<String, String> {
        self.cache.lock().unwrap().shortcuts.clone()
    }

    pub fn add_shortcut(&self, key: String, cmd: String) -> Result<(), String> {
        self.cache.lock().unwrap().shortcuts.insert(key, cmd);
        self.save()
    }
    
    pub fn remove_shortcut(&self, key: &str) -> Result<(), String> {
        self.cache.lock().unwrap().shortcuts.remove(key);
        self.save()
    }

    pub fn get_macros(&self) -> Vec<Macro> {
        self.cache.lock().unwrap().macros.iter().map(|m| m.clone().into()).collect()
    }

    pub fn add_macro(&self, r#macro: Macro) -> Result<(), String> {
        let mut data = self.cache.lock().unwrap();
        // Remove existing if any
        data.macros.retain(|m| m.name != r#macro.name);
        data.macros.push(r#macro.into());
        drop(data);
        self.save()
    }
     
    pub fn remove_macro(&self, name: &str) -> Result<(), String> {
        let mut data = self.cache.lock().unwrap();
        data.macros.retain(|m| m.name != name);
        drop(data);
        self.save()
    }

    pub fn get_ai_model(&self) -> String {
        self.cache.lock().unwrap().ai_model.clone()
    }

    pub fn set_ai_model(&self, model: String) -> Result<(), String> {
        self.cache.lock().unwrap().ai_model = model;
        self.save()
    }
}
