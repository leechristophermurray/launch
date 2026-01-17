use gtk4::prelude::*;
use gtk4::{Application, ListBox};
use launch::*;
use launch::test_utils::*;
use std::sync::Arc;

#[test]
fn test_ui_search_flow() {
    // Initialize GTK
    gtk4::init().unwrap();

    // 1. Setup AppContext with Mocks
    let omnibar = Arc::new(create_omnibar());
    
    // We need a dummy executor since we won't execute for this test
    let command_executor = Arc::new(SystemCommandExecutorAdapter::new());
    let settings_store = Arc::new(SettingsStore::new());
    let macro_repo = Arc::new(MockMacro);
    let power = Arc::new(MockPower);
    let win_repo = Arc::new(MockWindowRepo);
    
    let execute_command = Arc::new(ExecuteCommand::new(
         command_executor, 
         macro_repo, 
         omnibar.clone(),
         power,
         win_repo,
         Arc::new(MockTimeService),
    ));

    let ctx = AppContext {
        omnibar,
        execute_command,
        settings: settings_store,
    };

    // 2. Initialize Application
    let app = Application::builder()
        .application_id("com.launch.test")
        .build();
    
    let _hold = app.hold();
    app.register(gtk4::gio::Cancellable::NONE).expect("Failed to register app");

    // 3. Build UI
    let ctx_clone = ctx.clone();
    app.connect_activate(move |app| {
        build_ui(app, ctx_clone.clone());
    });
    
    app.activate();
    
    // Force event processing
    let main_context = glib::MainContext::default();
    while main_context.iteration(false) {}
    
    // 4. Get Window and Widgets
    // Wait for window
    let mut window = None;
    for _ in 0..10 {
         let windows = app.windows();
         if let Some(w) = windows.first() {
             window = Some(w.clone());
             break;
         }
         while main_context.iteration(false) {}
    }
    
    let window = window.expect("Window not created");
    // Main box
    let main_box = window.child().expect("Main box missing")
        .downcast::<gtk4::Box>().expect("Not a box");
    
    // Entry is inside a horizontal box (first child of main_box)
    let entry_box = main_box.first_child().expect("Entry Box missing")
        .downcast::<gtk4::Box>().expect("Not a box");

    // Entry is the first child of entry_box
    let entry = entry_box.first_child().expect("Entry missing")
        .downcast::<gtk4::Entry>().expect("Not an entry");
    
    // ListBox is the second child of MAIN BOX
    let list_box = entry_box.next_sibling().expect("ListBox missing")
        .downcast::<gtk4::ListBox>().expect("Not a ListBox");

    // 5. Verify Initial State (Hidden ListBox)
    assert!(!list_box.is_visible());
    
    // 6. Simulate Typing "d rust" (Dictionary lookup)
    entry.set_text("d rust");
    
    // Pump loop
    for _ in 0..20 {
        while main_context.iteration(false) {}
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    
    // 7. Verify Results
    assert!(list_box.is_visible());
    
    // Check we have results
    assert!(list_box.row_at_index(0).is_some());
    
    // Cleanup
    window.close();
    while main_context.iteration(false) {}
}
