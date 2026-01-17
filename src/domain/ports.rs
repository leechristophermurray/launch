use crate::domain::model::{App, Macro, Window};

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

pub trait IDictionaryService {
    fn lookup(&self, term: &str) -> Option<String>;
}

pub trait ILLMService {
    fn query(&self, prompt: &str, context: Option<String>) -> Result<String, String>;
    fn list_models(&self) -> Result<Vec<String>, String>;
    fn pull_model(&self, model: &str, on_progress: Box<dyn Fn(f64) + Send>) -> Result<(), String>;
    fn delete_model(&self, model: &str) -> Result<(), String>;
    fn set_model(&self, model: &str);
}

pub trait IFileIndexer {
    fn search(&self, term: &str) -> Vec<String>;
    fn index_home(&self);
}

pub trait IMacroRepository {
    fn get(&self, name: &str) -> Option<Macro>;
    fn get_all(&self) -> Vec<Macro>;
    fn add(&self, mac: Macro) -> Result<(), String>;
    fn remove(&self, name: &str) -> Result<(), String>;
}

pub trait IWindowRepository {
    fn get_open_windows(&self) -> Vec<Window>;
    fn focus_window(&self, id: &str) -> Result<(), String>;
}

pub trait ITimeService {
    fn start_timer(&self, duration_secs: u64);
    fn start_pomodoro(&self);
    fn start_stopwatch(&self);
    // Returns (Label, is_active) - e.g. ("24:59", true) or ("", false)
    fn get_status(&self) -> (String, bool);
    fn stop(&self);
    fn toggle_pause(&self);
    fn restart(&self);
}
