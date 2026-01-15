use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, CssProvider, Entry, ListBox, 
    ListBoxRow, Label, Orientation, StyleContext, gdk, Notebook,
    TextView, TextBuffer, Button, ScrolledWindow, Window, MessageDialog, ResponseType, DialogFlags
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
                    } else if cmd == "internal:settings" {
                        show_settings_dialog(&window_exec, &ctx_clone_exec);
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

fn show_error_dialog(parent: &gtk4::Window, message: &str) {
    let dialog = MessageDialog::new(
        Some(parent),
        DialogFlags::MODAL,
        gtk4::MessageType::Error,
        gtk4::ButtonsType::Ok,
        message
    );
    dialog.connect_response(|d, _| d.close());
    dialog.present();
}

fn show_about_dialog(window: &ApplicationWindow) {
    let dialog = gtk4::AboutDialog::builder()
        .transient_for(window)
        .modal(true)
        .program_name("Launch")
        .version("0.3.141592")
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

fn manage_shortcut_dialog(
    parent: &gtk4::Window, 
    ctx: &AppContext, 
    existing: Option<(String, String)>,
    on_success: impl Fn() + 'static
) {
    let is_edit = existing.is_some();
    let title = if is_edit { "Edit Shortcut" } else { "Add Shortcut" };

    let dialog = Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(title)
        .default_width(300)
        .default_height(150)
        .build();
    
    let vbox = gtk4::Box::new(Orientation::Vertical, 10);
    vbox.set_margin_top(10);
    vbox.set_margin_bottom(10);
    vbox.set_margin_start(10);
    vbox.set_margin_end(10);
    
    let key_entry = Entry::new();
    key_entry.set_placeholder_text(Some("Key (e.g. 'term')"));
    
    let cmd_entry = Entry::new();
    cmd_entry.set_placeholder_text(Some("Command (e.g. 'gnome-terminal')"));
    
    if let Some((k, c)) = &existing {
        key_entry.set_text(k);
        cmd_entry.set_text(c);
    }

    let btn_box = gtk4::Box::new(Orientation::Horizontal, 10);
    let save_btn = Button::with_label("Save");
    let cancel_btn = Button::with_label("Cancel");
    
    btn_box.append(&save_btn);
    btn_box.append(&cancel_btn);
    
    vbox.append(&Label::new(Some("Shortcut Key:")));
    vbox.append(&key_entry);
    vbox.append(&Label::new(Some("Command:")));
    vbox.append(&cmd_entry);
    vbox.append(&btn_box);
    
    dialog.set_child(Some(&vbox));
    
    let dialog_weak = dialog.downgrade();
    cancel_btn.connect_clicked(move |_| {
        if let Some(d) = dialog_weak.upgrade() { d.close(); }
    });
    
    let ctx_clone = ctx.clone();
    let dialog_weak_save = dialog.downgrade();
    let key_entry_clone = key_entry.clone();
    let cmd_entry_clone = cmd_entry.clone();
    let existing_unwrap = existing.clone();

    save_btn.connect_clicked(move |_| {
        let key = key_entry_clone.text().to_string();
        let cmd = cmd_entry_clone.text().to_string();
        
        let old_key = existing_unwrap.as_ref().map(|(k, _)| k.clone());

        if !key.is_empty() && !cmd.is_empty() {
             // If editing and key changed, remove old one first
             if let Some(old) = &old_key {
                 if old != &key {
                     if let Err(e) = ctx_clone.omnibar.shortcuts.remove(old) {
                         println!("Error removing old shortcut during rename: {}", e);
                         // Continue? Or fail? Probably safer to fail or warn.
                     }
                 }
             }

             // Add new/updated to repo
             if let Err(e) = ctx_clone.omnibar.shortcuts.add(key.clone(), cmd) {
                 println!("Error adding shortcut: {}", e);
                 if let Some(d) = dialog_weak_save.upgrade() {
                     show_error_dialog(&d, &format!("Failed to save shortcut: {}", e));
                 }
                 // If failed and we removed old key, we might be in bad state.
                 // Ideally this should be transactional, but for now we accept this risk.
             } else {
                 on_success();
                 if let Some(d) = dialog_weak_save.upgrade() { d.close(); }
             }
        }
    });


    dialog.present();
}

use crate::domain::model::Macro;
fn manage_macro_dialog(
    parent: &gtk4::Window,
    ctx: &AppContext,
    existing: Option<Macro>,
    on_success: impl Fn() + 'static
) {
    let is_edit = existing.is_some();
    let title = if is_edit { "Edit Macro" } else { "Add Macro" };

    let dialog = Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(title)
        .default_width(400)
        .default_height(300)
        .build();
    
    let vbox = gtk4::Box::new(Orientation::Vertical, 10);
    vbox.set_margin_top(10);
    vbox.set_margin_bottom(10);
    vbox.set_margin_start(10);
    vbox.set_margin_end(10);
    
    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("Macro Name (e.g. 'dev-setup')"));
    
    let buffer = TextBuffer::new(None);
    
    if let Some(mac) = &existing {
        name_entry.set_text(&mac.name);
        buffer.set_text(&mac.actions.join("\n"));
    }

    let text_view = TextView::with_buffer(&buffer);
    text_view.set_vexpand(true);
    let sc = ScrolledWindow::builder()
        .child(&text_view)
        .min_content_height(150)
        .build();
    
    let btn_box = gtk4::Box::new(Orientation::Horizontal, 10);
    let save_btn = Button::with_label("Save");
    let cancel_btn = Button::with_label("Cancel");
    
    btn_box.append(&save_btn);
    btn_box.append(&cancel_btn);

    vbox.append(&Label::new(Some("Macro Name:")));
    vbox.append(&name_entry);
    vbox.append(&Label::new(Some("Actions (One command per line):")));
    vbox.append(&sc);
    vbox.append(&btn_box);
    
    dialog.set_child(Some(&vbox));

    let dialog_weak = dialog.downgrade();
    cancel_btn.connect_clicked(move |_| {
        if let Some(d) = dialog_weak.upgrade() { d.close(); }
    });
    
    let ctx_clone = ctx.clone();
    let dialog_weak_save = dialog.downgrade();
    let name_entry_clone = name_entry.clone();
    let existing_unwrap = existing.clone();
    
    save_btn.connect_clicked(move |_| {
        let name = name_entry_clone.text().to_string();
        let (start, end) = buffer.bounds();
        let text = buffer.text(&start, &end, false).to_string();
        
        let actions: Vec<String> = text.lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
            
        let old_name = existing_unwrap.as_ref().map(|m| m.name.clone());

        if !name.is_empty() && !actions.is_empty() {
             if let Some(old) = &old_name {
                 if old != &name {
                     if let Err(e) = ctx_clone.omnibar.macros.remove(old) {
                         println!("Error removing old macro during rename: {}", e);
                     }
                 }
             }

             let new_macro = Macro { name: name.clone(), actions: actions };
             if let Err(e) = ctx_clone.omnibar.macros.add(new_macro) {
                 println!("Error adding macro: {}", e);
                 if let Some(d) = dialog_weak_save.upgrade() {
                     show_error_dialog(&d, &format!("Failed to save macro: {}", e));
                 }
             } else {
                 on_success();
                 if let Some(d) = dialog_weak_save.upgrade() { d.close(); }
             }
        }
    });

    dialog.present();
}

fn show_settings_dialog(window: &ApplicationWindow, ctx: &AppContext) {
    let dialog = gtk4::Window::builder()
        .transient_for(window)
        .modal(true)
        .title("Settings")
        .default_width(600)
        .default_height(500)
        .build();
    
    let notebook = Notebook::new();
    
    // TAB 1: SHORTCUTS
    let shortcuts_box = gtk4::Box::new(Orientation::Vertical, 10);
    shortcuts_box.set_margin_top(10);
    shortcuts_box.set_margin_bottom(10);
    shortcuts_box.set_margin_start(10);
    shortcuts_box.set_margin_end(10);
    
    let sc_list = ListBox::new();
    sc_list.set_selection_mode(gtk4::SelectionMode::Single);
    sc_list.add_css_class("boxed-list"); 
    sc_list.set_vexpand(true);
    
    let sc_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .min_content_height(300)
        .child(&sc_list)
        .build();
    shortcuts_box.append(&sc_scroll);
    
    // Logic to populate shortcuts
    let ctx_refresh_sc = ctx.clone();
    let sc_list_refresh = sc_list.clone();
    let refresh_shortcuts = std::rc::Rc::new(move || {
        // Clear
        while let Some(child) = sc_list_refresh.first_child() {
            sc_list_refresh.remove(&child);
        }
        // Populate
        let shortcuts = ctx_refresh_sc.omnibar.shortcuts.get_all();
        // Sort keys
        let mut sorted_keys: Vec<_> = shortcuts.keys().cloned().collect();
        sorted_keys.sort();
        
        for key in sorted_keys {
             if let Some(cmd) = shortcuts.get(&key) {
                let row = ListBoxRow::new();
                let hbox = gtk4::Box::new(Orientation::Horizontal, 10);
                hbox.set_margin_start(10);
                hbox.set_margin_end(10);
                hbox.set_margin_top(5);
                hbox.set_margin_bottom(5);
                
                let key_lbl = Label::new(Some(&key));
                key_lbl.add_css_class("heading");
                key_lbl.set_hexpand(true);
                key_lbl.set_halign(gtk4::Align::Start);
                
                let cmd_lbl = Label::new(Some(cmd));
                cmd_lbl.add_css_class("dim-label");
                
                hbox.append(&key_lbl);
                hbox.append(&cmd_lbl);
                
                // Store Key in row data via name? 
                // We need to identify the row for deletion. 
                // ListBoxRow doesn't hold data easily without subclassing or using set_widget_name (hacky but works)
                row.set_widget_name(&key);
                
                row.set_child(Some(&hbox));
                sc_list_refresh.append(&row);
             }
        }
    });

    // Initial load
    refresh_shortcuts();


    let sc_actions = gtk4::Box::new(Orientation::Horizontal, 10);
    let add_sc_btn = gtk4::Button::with_label("Add");
    let edit_sc_btn = gtk4::Button::with_label("Edit");
    let del_sc_btn = gtk4::Button::with_label("Delete");
    sc_actions.append(&add_sc_btn);
    sc_actions.append(&edit_sc_btn);
    sc_actions.append(&del_sc_btn);
    shortcuts_box.append(&sc_actions);
    
    let ctx_clone_add_sc = ctx.clone();
    let dialog_weak_sc = dialog.downgrade();
    let refresh_sc_clone_add = refresh_shortcuts.clone();
    
    add_sc_btn.connect_clicked(move |_| {
        if let Some(parent) = dialog_weak_sc.upgrade() {
            let refresh = refresh_sc_clone_add.clone();
            manage_shortcut_dialog(&parent, &ctx_clone_add_sc, None, move || {
                refresh();
            });
        }
    });
    
    let sc_list_edit = sc_list.clone();
    let ctx_clone_edit_sc = ctx.clone();
    let refresh_sc_clone_edit = refresh_shortcuts.clone();
    let dialog_weak_edit_sc = dialog.downgrade();

    edit_sc_btn.connect_clicked(move |_| {
         if let Some(row) = sc_list_edit.selected_row() {
            let key = row.widget_name().to_string();
            if !key.is_empty() {
                // Fetch details
                if let Some(cmd) = ctx_clone_edit_sc.omnibar.shortcuts.get(&key) {
                    if let Some(parent) = dialog_weak_edit_sc.upgrade() {
                         let refresh = refresh_sc_clone_edit.clone();
                         manage_shortcut_dialog(&parent, &ctx_clone_edit_sc, Some((key, cmd)), move || {
                             refresh();
                         });
                    }
                }
            }
         }
    });

    let sc_list_del = sc_list.clone();
    let ctx_clone_del_sc = ctx.clone();
    let refresh_sc_clone_del = refresh_shortcuts.clone();
    let dialog_weak_del_sc = dialog.downgrade();
    
    del_sc_btn.connect_clicked(move |_| {
        if let Some(row) = sc_list_del.selected_row() {
            let key = row.widget_name().to_string();
            if !key.is_empty() {
                if let Err(e) = ctx_clone_del_sc.omnibar.shortcuts.remove(&key) {
                    println!("Error removing shortcut: {}", e);
                    if let Some(d) = dialog_weak_del_sc.upgrade() {
                         show_error_dialog(&d, &format!("Failed to remove shortcut: {}", e));
                    }
                } else {
                    refresh_sc_clone_del();
                }
            }
        }
    });

    notebook.append_page(&shortcuts_box, Some(&Label::new(Some("Shortcuts"))));

    // TAB 2: MACROS
    let macros_box = gtk4::Box::new(Orientation::Vertical, 10);
    macros_box.set_margin_top(10);
    macros_box.set_margin_bottom(10);
    macros_box.set_margin_start(10);
    macros_box.set_margin_end(10);
    
    let mac_list = ListBox::new();
    mac_list.set_selection_mode(gtk4::SelectionMode::Single);
    mac_list.add_css_class("boxed-list"); 
    mac_list.set_vexpand(true);

    let mac_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .min_content_height(300)
        .child(&mac_list)
        .build();
    macros_box.append(&mac_scroll);
    
    // Logic to populate macros
    let ctx_refresh_mac = ctx.clone();
    let mac_list_refresh = mac_list.clone();
    let refresh_macros = std::rc::Rc::new(move || {
        while let Some(child) = mac_list_refresh.first_child() {
            mac_list_refresh.remove(&child);
        }
        let macros = ctx_refresh_mac.omnibar.macros.get_all();
        // Sort? macros is Vec, already order from storage probably.
        for mac in macros {
            let row = ListBoxRow::new();
            let hbox = gtk4::Box::new(Orientation::Horizontal, 10);
            hbox.set_margin_start(10);
            hbox.set_margin_end(10);
            hbox.set_margin_top(5);
            hbox.set_margin_bottom(5);
            
            let name_label = Label::new(Some(&mac.name));
            name_label.add_css_class("heading");
            name_label.set_hexpand(true);
            name_label.set_halign(gtk4::Align::Start);
            
            let count_label = Label::new(Some(&format!("{} actions", mac.actions.len())));
            count_label.add_css_class("dim-label");
            
            hbox.append(&name_label);
            hbox.append(&count_label);
            
            row.set_widget_name(&mac.name);
            row.set_child(Some(&hbox));
            mac_list_refresh.append(&row);
        }
    });

    refresh_macros();
    

    let mac_actions = gtk4::Box::new(Orientation::Horizontal, 10);
    let add_mac_btn = gtk4::Button::with_label("Add");
    let edit_mac_btn = gtk4::Button::with_label("Edit");
    let del_mac_btn = gtk4::Button::with_label("Delete");
    mac_actions.append(&add_mac_btn);
    mac_actions.append(&edit_mac_btn);
    mac_actions.append(&del_mac_btn);
    macros_box.append(&mac_actions);

    let ctx_clone_add_mac = ctx.clone();
    let dialog_weak_mac = dialog.downgrade();
    let refresh_mac_clone_add = refresh_macros.clone();

    add_mac_btn.connect_clicked(move |_| {
         if let Some(parent) = dialog_weak_mac.upgrade() {
            let refresh = refresh_mac_clone_add.clone();
            manage_macro_dialog(&parent, &ctx_clone_add_mac, None, move || {
                refresh();
            });
         }
    });

    let mac_list_edit = mac_list.clone();
    let ctx_clone_edit_mac = ctx.clone();
    let refresh_mac_clone_edit = refresh_macros.clone();
    let dialog_weak_edit_mac = dialog.downgrade();

    edit_mac_btn.connect_clicked(move |_| {
         if let Some(row) = mac_list_edit.selected_row() {
            let name = row.widget_name().to_string();
            if !name.is_empty() {
                 if let Some(mac) = ctx_clone_edit_mac.omnibar.macros.get(&name) {
                     if let Some(parent) = dialog_weak_edit_mac.upgrade() {
                         let refresh = refresh_mac_clone_edit.clone();
                         manage_macro_dialog(&parent, &ctx_clone_edit_mac, Some(mac), move || {
                              refresh();
                         });
                     }
                 }
            }
         }
    });

    let mac_list_del = mac_list.clone();
    let ctx_clone_del_mac = ctx.clone();
    let refresh_mac_clone_del = refresh_macros.clone();
    let dialog_weak_del_mac = dialog.downgrade();

    del_mac_btn.connect_clicked(move |_| {
         if let Some(row) = mac_list_del.selected_row() {
            let name = row.widget_name().to_string();
            if !name.is_empty() {
                if let Err(e) = ctx_clone_del_mac.omnibar.macros.remove(&name) {
                     println!("Error removing macro: {}", e);
                     if let Some(d) = dialog_weak_del_mac.upgrade() {
                          show_error_dialog(&d, &format!("Failed to remove macro: {}", e));
                     }
                } else {
                     refresh_mac_clone_del();
                }
            }
         }
    });

    notebook.append_page(&macros_box, Some(&Label::new(Some("Macros"))));

    dialog.set_child(Some(&notebook));
    dialog.present();
}
