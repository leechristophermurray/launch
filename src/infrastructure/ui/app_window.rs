use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, CssProvider, Entry, ListBox, 
    ListBoxRow, Label, Orientation, StyleContext, gdk
};
use std::sync::Arc;
use crate::application::use_cases::search_apps::SearchApps;
use crate::application::use_cases::execute_command::ExecuteCommand;

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

    let current_cmds = std::rc::Rc::new(std::cell::RefCell::new(Vec::<String>::new()));

    let ctx_clone = ctx.clone();
    let list_box_clone = list_box.clone();
    let window_clone = window.clone();
    let cmds_clone = current_cmds.clone();

    // On text change -> search
    entry.connect_changed(move |e| {
        let query = e.text();
        
        // Remove all children
        while let Some(child) = list_box_clone.first_child() {
            list_box_clone.remove(&child);
        }
        
        cmds_clone.borrow_mut().clear();

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
             let mut cmds = cmds_clone.borrow_mut();
             
             for app in results.iter().take(9) { // Show top 9 to match Ctrl+1-9
                 let row = ListBoxRow::new();
                 let row_box = gtk4::Box::new(Orientation::Horizontal, 10);
                 
                 // Icon
                 if let Some(icon_str) = &app.icon {
                    let image = gtk4::Image::new();
                    // Check if it's a path or icon name
                    if std::path::Path::new(icon_str).exists() {
                        image.set_from_file(Some(icon_str));
                    } else {
                        image.set_icon_name(Some(icon_str));
                    }
                    image.set_pixel_size(24);
                    row_box.append(&image);
                 }

                 let label = Label::new(Some(&app.name));
                 label.set_halign(gtk4::Align::Start);
                 label.set_margin_top(5);
                 label.set_margin_bottom(5);
                 
                 if app.is_running {
                    label.add_css_class("running-app");
                 }

                 row_box.append(&label);
                 row.set_child(Some(&row_box));
                 
                 list_box_clone.append(&row);
                 cmds.push(app.exec_path.clone());
             }
             
             // Select first row strictly
             if let Some(row) = list_box_clone.row_at_index(0) {
                 list_box_clone.select_row(Some(&row));
             }
        }
    });

    let ctx_clone_exec = ctx.clone();
    let list_box_exec = list_box.clone();
    let window_exec = window.clone();
    let cmds_exec = current_cmds.clone();
    
    // On Enter -> Execute Selected
    entry.connect_activate(move |e| {
        if let Some(row) = list_box_exec.selected_row() {
            let idx = row.index() as usize;
            let cmds = cmds_exec.borrow();
            if let Some(cmd) = cmds.get(idx) {
                ctx_clone_exec.execute_command.execute(cmd);
                e.set_text("");
                window_exec.minimize();
            }
        }
    });
    
    // Key Controller on Entry to handle navigation and shortcuts
    let controller = gtk4::EventControllerKey::new();
    let list_box_key = list_box.clone();
    let cmds_key = current_cmds.clone();
    let ctx_key_exec = ctx.clone();
    let win_key = window.clone();
    let entry_key = entry.clone();

    controller.connect_key_pressed(move |_, key, _keycode, state| {
        if key == gtk4::gdk::Key::Escape {
             win_key.close();
             return gtk4::glib::Propagation::Stop;
        }
        
        if key == gtk4::gdk::Key::Up {
             if let Some(row) = list_box_key.selected_row() {
                 let cur_idx = row.index();
                 if cur_idx > 0 {
                     if let Some(prev) = list_box_key.row_at_index(cur_idx - 1) {
                         list_box_key.select_row(Some(&prev));
                     }
                 }
             }
             return gtk4::glib::Propagation::Stop;
        }
        
        if key == gtk4::gdk::Key::Down {
             let cur_idx = list_box_key.selected_row().map(|r| r.index()).unwrap_or(-1);
             // We don't verify max size easily here without querying children count, 
             // but `row_at_index` returns None if out of bounds.
             if let Some(next) = list_box_key.row_at_index(cur_idx + 1) {
                 list_box_key.select_row(Some(&next));
             }
             return gtk4::glib::Propagation::Stop;
        }

        // Ctrl + 1..9
        if state.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
             let key_val = key.to_unicode(); // char
             if let Some(c) = key_val {
                 if let Some(digit) = c.to_digit(10) {
                     if digit >= 1 && digit <= 9 {
                         let idx = (digit - 1) as usize;
                         let cmds = cmds_key.borrow();
                         if let Some(cmd) = cmds.get(idx) {
                             ctx_key_exec.execute_command.execute(cmd);
                             entry_key.set_text("");
                             win_key.minimize();
                             return gtk4::glib::Propagation::Stop;
                         }
                     }
                 }
             }
        }

        gtk4::glib::Propagation::Proceed
    });
    entry.add_controller(controller);

    window.present();
}
