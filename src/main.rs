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
        .flags(gtk4::gio::ApplicationFlags::HANDLES_COMMAND_LINE) // Enable CLI handling
        .build();

    // Keep app alive even when windows are hidden
    let hold_guard = std::rc::Rc::new(std::cell::RefCell::new(None));

    let hold_guard_activate = hold_guard.clone();
    let ctx_activate = ctx.clone();
    app.connect_activate(move |app| {
        // Ensure we hold the app to prevent exit when window hides
        if hold_guard_activate.borrow().is_none() {
             *hold_guard_activate.borrow_mut() = Some(app.hold());
        }

        if let Some(window) = app.windows().first() {
            window.present();
        } else {
            build_ui(app, ctx_activate.clone());
        }
    });

    let hold_guard_cmd = hold_guard.clone();
    let ctx_cmd = ctx.clone();
    app.connect_command_line(move |app, cmdline| {
        // Ensure hold
        if hold_guard_cmd.borrow().is_none() {
             *hold_guard_cmd.borrow_mut() = Some(app.hold());
        }

        let args = cmdline.arguments();
        let toggle = args.iter().any(|arg| arg.to_str() == Some("toggle"));

        if toggle {
            if let Some(window) = app.windows().first() {
                if window.is_visible() && window.is_active() {
                    window.set_visible(false);
                } else {
                    window.present();
                }
            } else {
                build_ui(app, ctx_cmd.clone());
            }
        } else {
            // Default behavior: Open/Show
            app.activate();
        }
        
        0 // Exit code
    });

    app.run();
}
