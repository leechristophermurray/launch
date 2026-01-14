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
