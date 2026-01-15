mod domain;
mod interface;
mod infrastructure;
mod application;

use gtk4::prelude::*;
use gtk4::Application;
use std::sync::Arc;

use crate::infrastructure::filesystem::desktop_entry_adapter::LinuxAppRepoAdapter;
use crate::infrastructure::filesystem::procfs_adapter::ProcFsMonitorAdapter;
use crate::infrastructure::system::command_executor_adapter::SystemCommandExecutorAdapter;
use crate::application::use_cases::search_apps::SearchApps;
use crate::application::use_cases::execute_command::ExecuteCommand;
use crate::infrastructure::ui::app_window::{build_ui, AppContext};

fn main() {
    // 1. Instantiate Adapters
    let app_repo = Arc::new(LinuxAppRepoAdapter::new());
    let process_monitor = Arc::new(ProcFsMonitorAdapter::new());
    let command_executor = Arc::new(SystemCommandExecutorAdapter::new());

    // 2. Instantiate Use Cases
    let search_apps = Arc::new(SearchApps::new(app_repo, process_monitor));
    let execute_command = Arc::new(ExecuteCommand::new(command_executor));

    // 3. Create Context
    let ctx = AppContext {
        search_apps,
        execute_command,
    };

    // 4. Initialize GTK Application
    let app = Application::builder()
        .application_id("com.launch.launcher")
        .build();

    app.connect_activate(move |app| {
        // Check if ANY window exists, not just the currently focused/active one.
        if let Some(window) = app.windows().first() {
            window.present();
        } else {
            build_ui(app, ctx.clone());
        }
    });

    app.run();
}
