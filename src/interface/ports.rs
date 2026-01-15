use crate::domain::model::App;

pub trait IAppRepository {
    fn find_apps(&self) -> Vec<App>;
}

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

pub trait IShortcutRepository {
    fn get(&self, key: &str) -> Option<String>;
    fn get_all(&self) -> std::collections::HashMap<String, String>;
}
