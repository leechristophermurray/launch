use crate::domain::model::{App, Macro};

pub trait IAppRepository {
    fn find_apps(&self) -> Vec<App>;
}

use std::collections::HashMap;

pub trait IProcessMonitor {
    fn get_running_pids(&self) -> Vec<u32>;
    fn update_app_status(&self, apps: &mut [App]);
}

pub trait ICommandExecutor {
    fn execute(&self, cmd: &str);
}

pub trait IFileSystem {
    fn list_dir(&self, path: &str) -> Vec<String>; // Returns raw names
    fn is_dir(&self, path: &str) -> bool;
    fn exists(&self, path: &str) -> bool;
}

pub trait ISystemPower {
    fn execute(&self, action: &str) -> Result<(), String>;
}

pub trait ICalculator {
    fn calculate(&self, expression: &str) -> Option<String>;
}


// ... other ports ...

pub trait IShortcutRepository {
    fn get(&self, key: &str) -> Option<String>;
    fn get_all(&self) -> HashMap<String, String>;
    fn add(&self, key: String, cmd: String) -> Result<(), String>;
    fn remove(&self, key: &str) -> Result<(), String>;
}

pub trait IMacroRepository {
    fn get(&self, name: &str) -> Option<Macro>;
    fn get_all(&self) -> Vec<Macro>;
    fn add(&self, mac: Macro) -> Result<(), String>;
    fn remove(&self, name: &str) -> Result<(), String>;
}
