use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, CssProvider, Entry, ListBox, 
    ListBoxRow, Label, Orientation, StyleContext, gdk
};
use std::sync::Arc;
use crate::application::use_cases::search_apps::SearchApps;
use crate::application::use_cases::execute_command::ExecuteCommand;
use crate::domain::model::App;

// UI Dependencies wrapper
#[derive(Clone)]
pub struct AppContext {
    pub search_apps: Arc<SearchApps>,
    pub execute_command: Arc<ExecuteCommand>,
}

pub fn build_ui(app: &Application, ctx: AppContext) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Launch")
        .default_width(600)
        .default_height(50) // Starts small
        .decorated(false) // Pill shape requirement
        .build();

    // CSS
    let provider = CssProvider::new();
    provider.load_from_data("
        window {
            background-color: transparent;
        }
        .pill-container {
            background-color: rgba(30, 30, 30, 0.95);
            border-radius: 30px;
            padding: 10px;
            box-shadow: 0 4px 15px rgba(0,0,0,0.5);
        }
        entry {
            background-color: transparent;
            border: none;
            font-size: 20px;
            color: white;
            padding: 10px;
        }
        list {
            background-color: transparent;
        }
        label {
            color: white;
        }
        .running-app {
            font-weight: bold;
            color: #aaddff;
        }
    ");
    StyleContext::add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let main_box = gtk4::Box::new(Orientation::Vertical, 0);
    main_box.add_css_class("pill-container");
    window.set_child(Some(&main_box));

    let entry = Entry::new();
    entry.set_placeholder_text(Some("Type to launch..."));
    main_box.append(&entry);

    let list_box = ListBox::new();
    list_box.set_visible(false); // Hidden initially
    main_box.append(&list_box);

    let ctx_clone = ctx.clone();
    let list_box_clone = list_box.clone();
    let window_clone = window.clone();

    // On text change -> search
    entry.connect_changed(move |e| {
        let query = e.text();
        
        // Remove all children
        while let Some(child) = list_box_clone.first_child() {
            list_box_clone.remove(&child);
        }

        if query.is_empty() {
            list_box_clone.set_visible(false);
            window_clone.set_default_height(50);
            return;
        }

        let results = ctx_clone.search_apps.execute(&query);
        
        if results.is_empty() {
             list_box_clone.set_visible(false);
        } else {
             list_box_clone.set_visible(true);
             for app in results.iter().take(5) { // Show top 5
                 let row = ListBoxRow::new();
                 let label = Label::new(Some(&app.name));
                 label.set_halign(gtk4::Align::Start);
                 label.set_margin_start(10);
                 label.set_margin_end(10);
                 label.set_margin_top(5);
                 label.set_margin_bottom(5);
                 
                 if app.is_running {
                    label.add_css_class("running-app");
                 }

                 row.set_child(Some(&label));
                 
                 // Store exec path in widget data? 
                 // Or just keep index. Simpler to assume top selection.
                 
                 list_box_clone.append(&row);
             }
        }
    });

    let ctx_clone_exec = ctx.clone();
    let list_box_exec = list_box.clone();
    let window_exec = window.clone();
    
    // On Enter -> Execute
    entry.connect_activate(move |e| {
        let query = e.text();
        // If list has selection, use that.
        // Else use first item. 
        // For simplicity, let's just re-run search or grab top item from logic.
        // Ideally we track the selected item in the listbox.
        
        let results = ctx_clone_exec.search_apps.execute(&query);
        if let Some(top_app) = results.first() {
             ctx_clone_exec.execute_command.execute(&top_app.exec_path);
             // Clear, close, or minimize
             e.set_text("");
             window_exec.minimize(); 
             // Or actually just hide? GTK4 window close usually kills app if it's the only one.
             // window_exec.close(); 
        }
    });
    
    // Key press for Escape -> Close
    let controller = gtk4::EventControllerKey::new();
    let win_clone_key = window.clone();
    controller.connect_key_pressed(move |_, key, _, _| {
         if key == gtk4::gdk::Key::Escape {
             win_clone_key.close();
             return gtk4::glib::Propagation::Stop;
         }
         gtk4::glib::Propagation::Proceed
    });
    window.add_controller(controller);

    window.present();
}
