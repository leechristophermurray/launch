use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, CssProvider, Entry, ListBox, 
    ListBoxRow, Label, Orientation, StyleContext, gdk
};
use std::sync::Arc;
use crate::application::use_cases::omnibar::Omnibar;
use crate::application::use_cases::execute_command::ExecuteCommand;
use crate::domain::model::App;
// UI Dependencies wrapper
#[derive(Clone)]
pub struct AppContext {
    pub omnibar: Arc<Omnibar>,
    pub execute_command: Arc<ExecuteCommand>,
}

pub fn build_ui(app: &Application, ctx: AppContext) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Launch")
        .default_width(600)
        .default_height(50) // Starts small
        .decorated(false) // Pill shape requirement
        .resizable(false)
        .build();

    // CSS
    let provider = CssProvider::new();
    provider.load_from_data("
        .launcher-window {
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
        .about-dialog {
            background-color: rgba(30, 30, 30, 0.95);
            color: white;
        }
        .about-dialog label {
            color: white;
        }
    ");
    StyleContext::add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let main_box = gtk4::Box::new(Orientation::Vertical, 0);
    main_box.add_css_class("pill-container");
    
    window.add_css_class("launcher-window");
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
        let query_text = e.text();
        let query = query_text.as_str();
        
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

        let results = ctx_clone.omnibar.search(query);
        
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
            
            // Clone the command to drop the borrow immediately
            let cmd_opt = cmds_exec.borrow().get(idx).cloned();
            
            if let Some(cmd) = cmd_opt {
                if cmd.starts_with("internal:") {
                    if cmd == "internal:quit" {
                        if let Some(app) = window_exec.application() { app.quit(); }
                    } else if cmd == "internal:about" {
                        show_about_dialog(&window_exec);
                    }
                } else {
                    ctx_clone_exec.execute_command.execute(&cmd);
                    e.set_text("");
                    window_exec.set_visible(false); // Hide on launch
                }
            }
        }
    });
    
    // Key Controller on Entry to handle navigation and shortcuts
    let controller = gtk4::EventControllerKey::new();
    controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    let list_box_key = list_box.clone();
    let cmds_key = current_cmds.clone();
    let ctx_key_exec = ctx.clone();
    let win_key = window.clone();
    let entry_key = entry.clone();

    controller.connect_key_pressed(move |_, key, _keycode, state| {
        if key == gtk4::gdk::Key::Escape {
             win_key.set_visible(false);
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

        // Left/Right for File Browser Navigation
        let current_text = entry_key.text().to_string();
        if current_text.starts_with("f ") {
            if key == gtk4::gdk::Key::Right {
                if let Some(row) = list_box_key.selected_row() {
                    let idx = row.index() as usize;
                    let path_opt = cmds_key.borrow().get(idx).cloned();
                    if let Some(path) = path_opt {
                        // Check if it's a directory (we exec nautilus "path")
                        if path.starts_with("nautilus \"") && path.ends_with("\"") {
                            let clean_path = &path[10..path.len()-1]; // Strip 'nautilus "' and '"'
                            entry_key.set_text(&format!("f {}/", clean_path));
                            entry_key.set_position(-1); // Move cursor to end
                            return gtk4::glib::Propagation::Stop;
                        }
                    }
                }
            } else if key == gtk4::gdk::Key::Left {
                // Go up one directory
                let path_part = current_text[2..].trim();
                let path = std::path::Path::new(path_part);
                if let Some(parent) = path.parent() {
                     // Don't go above /
                     if parent == std::path::Path::new("") {
                         entry_key.set_text("f /");
                     } else {
                         entry_key.set_text(&format!("f {}/", parent.display()));
                     }
                     entry_key.set_position(-1);
                     return gtk4::glib::Propagation::Stop;
                }
            }
        }

        // Ctrl + 1..9
        if state.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
             let key_val = key.to_unicode(); // char
             if let Some(c) = key_val {
                 if let Some(digit) = c.to_digit(10) {
                     if digit >= 1 && digit <= 9 {
                         let idx = (digit - 1) as usize;
                         
                         // Clone cmd to drop borrow, preventing panic on set_text
                         let cmd_opt = cmds_key.borrow().get(idx).cloned();
                         
                         if let Some(cmd) = cmd_opt {
                             if cmd.starts_with("internal:") {
                                if cmd == "internal:quit" {
                                    if let Some(app) = win_key.application() { app.quit(); }
                                } else if cmd == "internal:about" {
                                    show_about_dialog(&win_key);
                                }
                             } else {
                                 ctx_key_exec.execute_command.execute(&cmd);
                                 entry_key.set_text("");
                                 win_key.set_visible(false); // Hide on launch
                             }
                             return gtk4::glib::Propagation::Stop;
                         }
                     }
                 }
             }
        }

        gtk4::glib::Propagation::Proceed
    });
    entry.add_controller(controller);



    // Auto-exit on focus loss
    // Only close if the application doesn't have ANY active window (handling modals)
    let app_weak = window.application().expect("Window must have app").downgrade();
    window.connect_is_active_notify(move |win| {
        if !win.is_active() {
            if let Some(app) = app_weak.upgrade() {
                // If no window in our app is active, user clicked away -> Close
                if app.active_window().is_none() {
                    win.set_visible(false); 
                }
            }
        }
    });

    // Intercept Close Request (Alt+F4) to just hide
    window.connect_close_request(move |win| {
        win.set_visible(false);
        gtk4::glib::Propagation::Stop
    });

    window.present();
}

fn show_about_dialog(window: &ApplicationWindow) {
    let dialog = gtk4::AboutDialog::builder()
        .transient_for(window)
        .modal(true)
        .program_name("Launch")
        .version("0.1.0")
        .authors(vec!["Christopher L Murray".to_string()])
        .website("https://github.com/leechristophermurray")
        .license_type(gtk4::License::Custom)
        .license("Full license available at: https://github.com/leechristophermurray/launch/blob/master/LICENSE")
        .comments("A sleek, pill-shaped application launcher for Linux.")
        .logo_icon_name("system-search") 
        .build();
    
    // Apply styling
    dialog.add_css_class("about-dialog");
    
    dialog.present();
}
