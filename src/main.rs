mod domain;
mod interface;
mod infrastructure;
mod application;

use gtk4::prelude::*;
use gtk4::Application;
use std::sync::Arc;

use crate::infrastructure::filesystem::desktop_entry_adapter::LinuxAppRepoAdapter;
use crate::infrastructure::filesystem::procfs_adapter::ProcFsMonitorAdapter;
use crate::infrastructure::filesystem::fs_adapter::LocalFileSystemAdapter;
use crate::infrastructure::system::command_executor_adapter::SystemCommandExecutorAdapter;
use crate::infrastructure::services::system_adapter::SystemAdapter;
use crate::infrastructure::services::window_adapter::SystemWindowAdapter;
use crate::interface::ports::{ISystemPower, IWindowRepository};
use crate::infrastructure::services::calculator_adapter::MevalCalculatorAdapter;
use crate::infrastructure::services::settings_store::SettingsStore;
use crate::infrastructure::services::json_shortcut_adapter::JsonShortcutAdapter;
use crate::infrastructure::services::json_macro_adapter::JsonMacroAdapter;
use crate::infrastructure::services::dictionary_adapter::SmartDictionaryAdapter;

use crate::application::use_cases::omnibar::Omnibar;
use crate::application::use_cases::execute_command::ExecuteCommand;
use crate::infrastructure::ui::app_window::{build_ui, AppContext};

fn main() {
    // 1. Instantiate Adapters
    let app_repo = Arc::new(LinuxAppRepoAdapter::new());
    let process_monitor = Arc::new(ProcFsMonitorAdapter::new());
    let command_executor = Arc::new(SystemCommandExecutorAdapter::new());
    let fs_adapter = Arc::new(LocalFileSystemAdapter::new());
    let power_adapter: Arc<dyn ISystemPower + Send + Sync> = Arc::new(SystemAdapter::new());
    let window_adapter: Arc<dyn IWindowRepository + Send + Sync> = Arc::new(SystemWindowAdapter::new());
    let calculator_adapter = Arc::new(MevalCalculatorAdapter::new());
    let dictionary_adapter = Arc::new(SmartDictionaryAdapter::new());
    
    // Persistence
    let settings_store = Arc::new(SettingsStore::new());
    let shortcut_adapter = Arc::new(JsonShortcutAdapter::new(settings_store.clone()));
    let macro_adapter = Arc::new(JsonMacroAdapter::new(settings_store.clone()));

    // 2. Instantiate Use Cases
    let omnibar = Arc::new(Omnibar::new(
        app_repo.clone(),
        process_monitor.clone(),
        fs_adapter,
        shortcut_adapter.clone(),
        macro_adapter.clone(),
        window_adapter.clone(),
        power_adapter.clone(),
        calculator_adapter,
        dictionary_adapter,
    ));
    let execute_command = Arc::new(ExecuteCommand::new(command_executor, macro_adapter, omnibar.clone(), power_adapter.clone(), window_adapter));

    // 3. Create Context
    let ctx = AppContext {
        omnibar,
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
