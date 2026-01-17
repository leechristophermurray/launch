use crate::domain::model::{App, Macro, MacroAction, Window};
use crate::domain::ports::*;
use crate::application::use_cases::omnibar::Omnibar;
use std::collections::HashMap;
use std::sync::Arc;

// Mocks
pub struct MockAppRepo;
impl IAppRepository for MockAppRepo {
    fn find_apps(&self) -> Vec<App> { vec![] }
}

pub struct MockProcessMonitor;
impl IProcessMonitor for MockProcessMonitor {
    fn get_running_pids(&self) -> Vec<u32> { vec![] }
    fn update_app_status(&self, _apps: &mut [App]) {}
}

pub struct MockFS;
impl IFileSystem for MockFS {
    fn list_dir(&self, _path: &str) -> Vec<String> { vec![] }
    fn is_dir(&self, _path: &str) -> bool { false }
    fn exists(&self, _path: &str) -> bool { false }
}

pub struct MockShortcuts;
impl IShortcutRepository for MockShortcuts {
    fn get(&self, key: &str) -> Option<String> {
        if key == "term" { Some("gnome-terminal".to_string()) } else { None }
    }
    fn get_all(&self) -> HashMap<String, String> { HashMap::new() }
    fn add(&self, _key: String, _cmd: String) -> Result<(), String> { Ok(()) }
    fn remove(&self, _key: &str) -> Result<(), String> { Ok(()) }
}

pub struct MockPower;
impl ISystemPower for MockPower {
    fn execute(&self, _action: &str) -> Result<(), String> { Ok(()) }
}

pub struct MockWindowRepo;
impl IWindowRepository for MockWindowRepo {
    fn get_open_windows(&self) -> Vec<Window> {
            vec![
                Window { id: "0x1".to_string(), title: "Google Chrome".to_string(), app_name: "Chrome".to_string(), workspace: 0, screen: 0 },
                Window { id: "0x2".to_string(), title: "Terminal".to_string(), app_name: "Gnome-terminal".to_string(), workspace: 1, screen: 0 },
            ]
    }
    fn focus_window(&self, _id: &str) -> Result<(), String> { Ok(()) }
}

pub struct MockCalculator;
impl ICalculator for MockCalculator {
    fn calculate(&self, expression: &str) -> Option<String> {
        if expression.contains("1+1") { Some("2".to_string()) } else { None }
    }
}

pub struct MockDictionary;
impl IDictionaryService for MockDictionary {
    fn lookup(&self, term: &str) -> Option<String> {
        if term == "rust" { Some("Awesome language".to_string()) } else { None }
    }
}

pub struct MockLLM;
impl ILLMService for MockLLM {
    fn query(&self, _prompt: &str, _context: Option<String>) -> Result<String, String> { Ok("AI response".to_string()) }
    fn list_models(&self) -> Result<Vec<String>, String> { Ok(vec![]) }
    fn pull_model(&self, _model: &str, _on_progress: Box<dyn Fn(f64) + Send>) -> Result<(), String> { Ok(()) }
    fn delete_model(&self, _model: &str) -> Result<(), String> { Ok(()) }
    fn set_model(&self, _model: &str) {}
}

pub struct MockIndexer;
impl IFileIndexer for MockIndexer {
    fn search(&self, _term: &str) -> Vec<String> { vec![] }
    fn index_home(&self) {}
}

pub struct MockTimeService;
impl ITimeService for MockTimeService {
    fn start_timer(&self, _duration_secs: u64) {}
    fn start_pomodoro(&self) {}
    fn start_stopwatch(&self) {}
    fn get_status(&self) -> (String, bool) { ("".to_string(), false) }
    fn stop(&self) {}
    fn toggle_pause(&self) {}
    fn restart(&self) {}
}

pub struct MockMacro;
impl IMacroRepository for MockMacro {
    fn get(&self, name: &str) -> Option<Macro> {
        if name == "test" { 
            Some(Macro { name: "test".to_string(), actions: vec![MacroAction::Command("x echo hi".to_string())] }) 
        } else { None }
    }
    fn get_all(&self) -> Vec<Macro> { vec![] }
    fn add(&self, _mac: Macro) -> Result<(), String> { Ok(()) }
    fn remove(&self, _name: &str) -> Result<(), String> { Ok(()) }
}

pub fn create_omnibar() -> Omnibar {
    Omnibar::new(
        Arc::new(MockAppRepo),
        Arc::new(MockProcessMonitor),
        Arc::new(MockFS),
        Arc::new(MockShortcuts),
        Arc::new(MockMacro),
        Arc::new(MockWindowRepo),
        Arc::new(MockPower),
        Arc::new(MockCalculator),
        Arc::new(MockDictionary),
        Arc::new(MockLLM),
        Arc::new(MockIndexer),
        Arc::new(MockTimeService),
    )
}
