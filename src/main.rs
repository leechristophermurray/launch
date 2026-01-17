use gtk4::prelude::*;
use gtk4::Application;
use std::sync::Arc;

use launch::*;


fn main() {
    // 1. Instantiate Adapters
    let app_repo_inner = Arc::new(LinuxAppRepoAdapter::new());
    let process_monitor_inner = Arc::new(ProcFsMonitorAdapter::new());
    
    // Cached Service
    let app_cache = Arc::new(AppCacheService::new(app_repo_inner, process_monitor_inner));
    
    // For consistency, we treat app_cache as both Repo and Monitor for Omnibar
    let app_repo = app_cache.clone();
    let process_monitor = app_cache.clone();

    let command_executor = Arc::new(SystemCommandExecutorAdapter::new());
    let fs_adapter = Arc::new(LocalFileSystemAdapter::new());
    let power_adapter: Arc<dyn ISystemPower + Send + Sync> = Arc::new(SystemAdapter::new());
    let window_adapter: Arc<dyn IWindowRepository + Send + Sync> = Arc::new(SystemWindowAdapter::new());
    let calculator_adapter = Arc::new(MevalCalculatorAdapter::new());
    let dictionary_adapter = Arc::new(SmartDictionaryAdapter::new());
    let llm_adapter = Arc::new(OllamaAdapter::new("llama3")); // Default model
    let file_indexer = Arc::new(FileIndexerAdapter::new());
    file_indexer.index_home();
    let time_adapter: Arc<dyn ITimeService + Send + Sync> = Arc::new(TimeAdapter::new());
    
    // Persistence
    let settings_store = Arc::new(SettingsStore::new());
    let shortcut_adapter = Arc::new(JsonShortcutAdapter::new(settings_store.clone()));
    let macro_adapter = Arc::new(JsonMacroAdapter::new(settings_store.clone()));

    // 2. Instantiate Use Cases
    let omnibar = Arc::new(Omnibar::new(
        app_repo,
        process_monitor,
        fs_adapter,
        shortcut_adapter.clone(),
        macro_adapter.clone(),
        window_adapter.clone(),
        power_adapter.clone(),
        calculator_adapter,
        dictionary_adapter,
        llm_adapter,
        file_indexer,
        time_adapter.clone(),
    ));
    let execute_command = Arc::new(ExecuteCommand::new(command_executor, macro_adapter, omnibar.clone(), power_adapter.clone(), window_adapter, time_adapter));

    // 3. Create Context
    let ctx = AppContext {
        omnibar,
        execute_command,
        settings: settings_store.clone(),
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
