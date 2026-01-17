use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, CssProvider, Entry, ListBox, 
    ListBoxRow, Label, Orientation, StyleContext, gdk, Notebook,
    TextView, TextBuffer, Button, ScrolledWindow, Window, MessageDialog, DialogFlags,
    ComboBoxText, FlowBox, SelectionMode
};
use std::sync::Arc;
use crate::application::use_cases::omnibar::Omnibar;
use crate::application::use_cases::execute_command::ExecuteCommand;
use crate::infrastructure::services::settings_store::SettingsStore;

// UI Dependencies wrapper
#[derive(Clone)]
pub struct AppContext {
    pub omnibar: Arc<Omnibar>,
    pub execute_command: Arc<ExecuteCommand>,
    pub settings: Arc<SettingsStore>,
}

pub fn build_ui(app: &Application, ctx: AppContext) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Launch")
        .default_width(630)  // 600 + 15*2 for margin
        .default_height(80)  // 50 + 15*2 for margin
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
            background-color: rgba(9, 9, 9, 0.85);
            border-radius: 30px;
            padding: 10px;
            margin: 15px;
            box-shadow: 0 4px 15px rgba(0,0,0,0.5);
            /* Attempt backdrop blur if supported by GTK/Compositor extensions */
            backdrop-filter: blur(40px) saturate(180%);
            -webkit-backdrop-filter: blur(40px) saturate(180%);
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
        .running-dot {
            background-color: #aaddff;
            border-radius: 5px;
            min-width: 8px;
            min-height: 8px;
            margin-right: 8px;
            margin-bottom: 5px; /* Center with text vertically approx */
            margin-top: 5px;
        }
        .about-dialog {
            background-color: rgba(30, 30, 30, 0.95);
            color: white;
        }
        .about-dialog label {
            color: white;
        }
        .section-title {
            font-weight: bold;
            font-size: 14px;
            margin-top: 10px;
            margin-bottom: 5px;
            color: #888888;
        }
        .grid-item {
            padding: 10px;
            background-color: rgba(255, 255, 255, 0.05);
            border-radius: 8px;
            margin: 2px;
        }
        .grid-item:hover {
            background-color: rgba(255, 255, 255, 0.15);
        }
        .time-status {
            color: #F8cfa5; /* Pastel Orange/Coral */
            font-weight: bold;
            font-size: 16px;
            margin-right: 20px;
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

    let entry_box = gtk4::Box::new(Orientation::Horizontal, 0);
    main_box.append(&entry_box);

    let entry = Entry::new();
    entry.set_hexpand(true);
    entry.set_placeholder_text(Some("Type to launch..."));
    entry_box.append(&entry);

    let time_label = Label::new(None);
    time_label.add_css_class("time-status");
    entry_box.append(&time_label);

    let list_box = ListBox::new();
    list_box.set_visible(false); // Hidden initially
    main_box.append(&list_box);

    // Overview Grid (Zero State)
    let overview_scroll = ScrolledWindow::builder()
        .min_content_height(400)
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .visible(false)
        .build();
    let overview_box = gtk4::Box::new(Orientation::Vertical, 10);
    overview_box.set_margin_start(15);
    overview_box.set_margin_end(15);
    overview_box.set_margin_bottom(15);
    overview_scroll.set_child(Some(&overview_box));
    main_box.append(&overview_scroll);

    let current_cmds = std::rc::Rc::new(std::cell::RefCell::new(Vec::<String>::new()));

    let ctx_clone = ctx.clone();
    let list_box_clone = list_box.clone();
    let window_clone = window.clone();
    let cmds_clone = current_cmds.clone();
    let overview_scroll_clone = overview_scroll.clone();

    // On text change -> search
    entry.connect_changed(move |e| {
        let query_text = e.text();
        let query = query_text.as_str();
        
        // Remove all children
        while let Some(child) = list_box_clone.first_child() {
            list_box_clone.remove(&child);
        }
        
        cmds_clone.borrow_mut().clear();
        overview_scroll_clone.set_visible(false); // Hide overview on type

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
                 label.set_wrap(true);
                 label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
                 label.set_max_width_chars(50); // Prevent limitless expansion
                 label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
                 label.set_lines(5); // Show up to 5 lines of definition
                 
                 if app.is_running {
                    label.add_css_class("running-app");
                    
                    let dot = gtk4::Box::new(Orientation::Horizontal, 0);
                    dot.add_css_class("running-dot");
                    dot.set_valign(gtk4::Align::Center);
                    row_box.append(&dot);
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
                    } else if let Some(prompt) = cmd.strip_prefix("internal:ai:") {
                         // 1. Clear list to show we are doing something, distinct from search results
                         while let Some(row) = list_box_exec.row_at_index(0) {
                             list_box_exec.remove(&row);
                         }
                         
                         // 2. Add Thinking row
                         let row = ListBoxRow::new();
                         let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
                         box_.set_margin_top(12);
                         box_.set_margin_bottom(12);
                         box_.set_margin_start(12);
                         box_.set_margin_end(12);
                         
                         let spinner = gtk4::Spinner::new();
                         spinner.start();
                         box_.append(&spinner);
                         
                         let label = Label::new(Some("Thinking..."));
                         box_.append(&label);
                         
                         row.set_child(Some(&box_));
                         row.set_activatable(false);
                         list_box_exec.append(&row);
                         
                         let ctx_ai_exec = ctx_clone_exec.clone();
                         let prompt_str = prompt.to_string();
                         let list_box_weak = list_box_exec.downgrade();
                         
                         let (sender, receiver) = std::sync::mpsc::channel();
                         std::thread::spawn(move || {
                             let response = match ctx_ai_exec.omnibar.query_ai(&prompt_str) {
                                 Ok(r) => r,
                                 Err(e) => format!("Error: {}", e),
                             };
                             let _ = sender.send(response);
                         });

                         glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                             if let Ok(response) = receiver.try_recv() {
                                 if let Some(lb) = list_box_weak.upgrade() {
                                     // Clear "Thinking" (or everything to be safe)
                                     while let Some(r) = lb.row_at_index(0) {
                                         lb.remove(&r);
                                     }
                                     
                                     // Add Result
                                     let row = ListBoxRow::new();
                                     let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
                                     box_.set_margin_top(12);
                                     box_.set_margin_bottom(12);
                                     box_.set_margin_start(12);
                                     box_.set_margin_end(12);
                                     // Top align for long text
                                     box_.set_valign(gtk4::Align::Start);
                                     
                                     let icon = gtk4::Image::from_icon_name("dialog-information");
                                     icon.set_pixel_size(24);
                                     icon.set_valign(gtk4::Align::Start);
                                     box_.append(&icon);
                                     
                                     let label = Label::new(Some(&response));
                                     label.set_wrap(true);
                                     label.set_wrap_mode(gtk4::pango::WrapMode::Word);
                                     label.set_xalign(0.0);
                                     label.set_valign(gtk4::Align::Start);
                                     label.set_hexpand(true);
                                     // Allow selecting text if necessary, but ListBoxRow steals clicks usually. 
                                     // label.set_selectable(true); 
                                     
                                     box_.append(&label);
                                     row.set_child(Some(&box_));
                                     row.set_activatable(false); 
                                     
                                     lb.append(&row);
                                 }
                                 return glib::ControlFlow::Break;
                             }
                             glib::ControlFlow::Continue
                         });
                    } else {
                        // Delegate other internal commands (time, window, system, macro) to executor
                        ctx_clone_exec.execute_command.execute(&cmd);
                        e.set_text("");
                        window_exec.set_visible(false);
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
    let entry_key = entry.clone();
    let overview_box_key = overview_box.clone();
    let overview_scroll_key = overview_scroll.clone();
    let ctx_key_exec = ctx.clone();
    let win_key = window.clone();

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
             // Logic: If list visible, navigate list. If list hidden and text empty, Show Overview.
             if list_box_key.is_visible() {
                 let cur_idx = list_box_key.selected_row().map(|r| r.index()).unwrap_or(-1);
                 if let Some(next) = list_box_key.row_at_index(cur_idx + 1) {
                     list_box_key.select_row(Some(&next));
                 }
             } else if entry_key.text().is_empty() {
                 // Show Overview Grid
                show_overview_grid(&overview_box_key, &ctx_key_exec, &win_key, &entry_key);
                 overview_scroll_key.set_visible(true);
                 
                 // Focus first item with small delay to ensure widgets are realized
                 let overview_box_focus = overview_box_key.clone();
                 gtk4::glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
                     // overview_box children are Label, ScrolledWindow, Label, ScrolledWindow...
                     let mut next_child = overview_box_focus.first_child();
                     while let Some(child) = next_child {
                         next_child = child.next_sibling();
                         if let Ok(scroll) = child.downcast::<ScrolledWindow>() {
                             if let Some(child_widget) = scroll.child() {
                                 // Try to find the inner box
                                 let target_box = if let Ok(b) = child_widget.clone().downcast::<gtk4::Box>() {
                                      b 
                                 } else if let Ok(vp) = child_widget.downcast::<gtk4::Viewport>() {
                                      if let Some(vp_child) = vp.child() {
                                          if let Ok(b) = vp_child.downcast::<gtk4::Box>() { b } else { continue; }
                                      } else { continue; }
                                 } else {
                                     continue;
                                 };

                                 // Focus first button
                                 if let Some(btn) = target_box.first_child() {
                                     btn.grab_focus();
                                     break; // Found and focused
                                 }
                             }
                         }
                     }
                     gtk4::glib::ControlFlow::Break
                 });
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

    // Time Status Poller
    let time_label_clone = time_label.clone();
    let ctx_poller = ctx.clone();
    gtk4::glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
        let (text, active) = ctx_poller.omnibar.time.get_status();
        if active {
            time_label_clone.set_text(&text);
            time_label_clone.set_visible(true);
        } else {
            time_label_clone.set_visible(false);
        }
        gtk4::glib::ControlFlow::Continue
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
        .version("0.5.0")
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

use crate::domain::model::{Macro, MacroAction};

fn manage_action_dialog(
    parent: &Window,
    existing: Option<MacroAction>,
    on_success: impl Fn(MacroAction) + 'static
) {
    let title = if existing.is_some() { "Edit Action" } else { "Add Action" };
    let dialog = Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(title)
        .default_width(300)
        .default_height(200)
        .build();

    let vbox = gtk4::Box::new(Orientation::Vertical, 10);
    vbox.set_margin_top(10);
    vbox.set_margin_bottom(10);
    vbox.set_margin_start(10);
    vbox.set_margin_end(10);

    let type_combo = ComboBoxText::new();
    type_combo.append(Some("Command"), "Command");
    type_combo.append(Some("LaunchApp"), "Launch App");
    type_combo.append(Some("OpenUrl"), "Open URL");
    type_combo.append(Some("TypeText"), "Type Text");
    type_combo.append(Some("Sleep"), "Sleep (ms)");
    type_combo.append(Some("System"), "System Action");
    type_combo.set_active_id(Some("Command"));

    let value_entry = Entry::new();
    value_entry.set_placeholder_text(Some("Value"));

    if let Some(action) = existing {
        match &action {
            MacroAction::Command(v) => { type_combo.set_active_id(Some("Command")); value_entry.set_text(v); },
            MacroAction::LaunchApp(v) => { type_combo.set_active_id(Some("LaunchApp")); value_entry.set_text(v); },
            MacroAction::OpenUrl(v) => { type_combo.set_active_id(Some("OpenUrl")); value_entry.set_text(v); },
            MacroAction::TypeText(v) => { type_combo.set_active_id(Some("TypeText")); value_entry.set_text(v); },
            MacroAction::Sleep(v) => { type_combo.set_active_id(Some("Sleep")); value_entry.set_text(&v.to_string()); },
            MacroAction::System(v) => { type_combo.set_active_id(Some("System")); value_entry.set_text(v); },
        }
    }

    let btn_box = gtk4::Box::new(Orientation::Horizontal, 10);
    let save_btn = Button::with_label("Save");
    let cancel_btn = Button::with_label("Cancel");
    btn_box.append(&save_btn);
    btn_box.append(&cancel_btn);

    vbox.append(&Label::new(Some("Action Type:")));
    vbox.append(&type_combo);
    vbox.append(&Label::new(Some("Value:")));
    vbox.append(&value_entry);
    vbox.append(&btn_box);

    dialog.set_child(Some(&vbox));

    let dialog_weak = dialog.downgrade();
    cancel_btn.connect_clicked(move |_| {
        if let Some(d) = dialog_weak.upgrade() { d.close(); }
    });

    let dialog_weak_save = dialog.downgrade();
    let type_combo_clone = type_combo.clone();
    let value_entry_clone = value_entry.clone();

    save_btn.connect_clicked(move |_| {
        if let Some(type_id) = type_combo_clone.active_id() {
            let val = value_entry_clone.text().to_string();
            let action = match type_id.as_str() {
                "Command" => Some(MacroAction::Command(val)),
                "LaunchApp" => Some(MacroAction::LaunchApp(val)),
                "OpenUrl" => Some(MacroAction::OpenUrl(val)),
                "TypeText" => Some(MacroAction::TypeText(val)),
                "Sleep" => val.parse::<u64>().ok().map(MacroAction::Sleep),
                "System" => Some(MacroAction::System(val)),
                _ => None,
            };

            if let Some(act) = action {
                on_success(act);
                if let Some(d) = dialog_weak_save.upgrade() { d.close(); }
            } else {
                 if let Some(d) = dialog_weak_save.upgrade() {
                     show_error_dialog(&d, "Invalid Value for Type (e.g. Sleep requires number)");
                 }
            }
        }
    });

    dialog.present();
}

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
        .default_width(500)
        .default_height(400)
        .build();
    
    let vbox = gtk4::Box::new(Orientation::Vertical, 10);
    vbox.set_margin_top(10);
    vbox.set_margin_bottom(10);
    vbox.set_margin_start(10);
    vbox.set_margin_end(10);
    
    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("Macro Name"));
    
    // Actions List
    let actions_list = ListBox::new();
    actions_list.set_selection_mode(gtk4::SelectionMode::Single);
    actions_list.add_css_class("boxed-list");
    let actions_scroll = ScrolledWindow::builder()
        .child(&actions_list)
        .min_content_height(200)
        .vexpand(true)
        .build();

    // Store actions in a Rc<RefCell<Vec<MacroAction>>>
    let current_actions = std::rc::Rc::new(std::cell::RefCell::new(Vec::<MacroAction>::new()));
    
    if let Some(mac) = &existing {
        name_entry.set_text(&mac.name);
        *current_actions.borrow_mut() = mac.actions.clone();
    }

    let refresh_actions_list = {
        let list = actions_list.clone();
        let acts = current_actions.clone();
        std::rc::Rc::new(move || {
            while let Some(child) = list.first_child() {
                list.remove(&child);
            }
            for (i, action) in acts.borrow().iter().enumerate() {
                let row = ListBoxRow::new();
                let label_text = match action {
                    MacroAction::Command(v) => format!("Command: {}", v),
                    MacroAction::LaunchApp(v) => format!("Launch: {}", v),
                    MacroAction::OpenUrl(v) => format!("Open URL: {}", v),
                    MacroAction::TypeText(v) => format!("Type: {}", v),
                    MacroAction::Sleep(v) => format!("Sleep: {}ms", v),
                    MacroAction::System(v) => format!("System: {}", v),
                };
                let label = Label::new(Some(&label_text));
                label.set_halign(gtk4::Align::Start);
                label.set_margin_start(10);
                row.set_child(Some(&label));
                // We could store index, but row index works
                list.append(&row);
            }
        })
    };

    refresh_actions_list();

    // Action Buttons
    let act_btn_box = gtk4::Box::new(Orientation::Horizontal, 5);
    let add_act_btn = Button::with_label("Add Action");
    let del_act_btn = Button::with_label("Remove Selected");
    // Move up/down? Maybe later.
    act_btn_box.append(&add_act_btn);
    act_btn_box.append(&del_act_btn);

    // Main Buttons
    let btn_box = gtk4::Box::new(Orientation::Horizontal, 10);
    let save_btn = Button::with_label("Save Macro");
    let cancel_btn = Button::with_label("Cancel");
    btn_box.append(&save_btn);
    btn_box.append(&cancel_btn);

    vbox.append(&Label::new(Some("Macro Name:")));
    vbox.append(&name_entry);
    vbox.append(&Label::new(Some("Actions:")));
    vbox.append(&actions_scroll);
    vbox.append(&act_btn_box);
    vbox.append(&btn_box);
    
    dialog.set_child(Some(&vbox));

    // Handlers
    let dialog_weak = dialog.downgrade();
    let dialog_weak_add = dialog.downgrade();
    
    cancel_btn.connect_clicked(move |_| {
        if let Some(d) = dialog_weak.upgrade() { d.close(); }
    });

    let acts_add = current_actions.clone();
    let refresh_add = refresh_actions_list.clone();
    add_act_btn.connect_clicked(move |_| {
        if let Some(parent) = dialog_weak_add.upgrade() {
            let acts = acts_add.clone();
            let refresh = refresh_add.clone();
            manage_action_dialog(&parent, None, move |new_action| {
                acts.borrow_mut().push(new_action);
                refresh();
            });
        }
    });

    let acts_del = current_actions.clone();
    let refresh_del = refresh_actions_list.clone();
    let list_del = actions_list.clone();
    del_act_btn.connect_clicked(move |_| {
        if let Some(row) = list_del.selected_row() {
            let idx = row.index() as usize;
            if idx < acts_del.borrow().len() {
                acts_del.borrow_mut().remove(idx);
                refresh_del();
            }
        }
    });

    let ctx_clone = ctx.clone();
    let dialog_weak_save = dialog.downgrade();
    let name_entry_clone = name_entry.clone();
    let acts_save = current_actions.clone();
    let existing_unwrap = existing.clone();

    save_btn.connect_clicked(move |_| {
        let name = name_entry_clone.text().to_string();
        let actions = acts_save.borrow().clone();
        
        let old_name = existing_unwrap.as_ref().map(|m| m.name.clone());

        if !name.is_empty() && !actions.is_empty() {
             if let Some(old) = &old_name {
                 if old != &name {
                     let _ = ctx_clone.omnibar.macros.remove(old);
                 }
             }

             let new_macro = Macro { name: name.clone(), actions };
             if let Err(e) = ctx_clone.omnibar.macros.add(new_macro) {
                 if let Some(d) = dialog_weak_save.upgrade() {
                     show_error_dialog(&d, &format!("Failed to save: {}", e));
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

    // TAB 3: AI
    let ai_box = gtk4::Box::new(Orientation::Vertical, 10);
    ai_box.set_margin_top(10);
    ai_box.set_margin_bottom(10);
    ai_box.set_margin_start(10);
    ai_box.set_margin_end(10);

    let model_label = Label::new(Some("AI Model (Ollama)"));
    model_label.set_halign(gtk4::Align::Start);
    model_label.add_css_class("heading");

    let model_combo = ComboBoxText::new();
    model_combo.append(Some("llama3"), "Llama 3 (Meta)");
    model_combo.append(Some("mistral"), "Mistral");
    model_combo.append(Some("gemma"), "Gemma (Google)");
    model_combo.append(Some("phi"), "Phi (Microsoft)");
    model_combo.append(Some("tinyllama"), "TinyLlama (1.1GB - Fast)");
    
    // Set current
    let current_model = ctx.settings.get_ai_model();
    if !model_combo.set_active_id(Some(&current_model)) {
        model_combo.set_active_id(Some("llama3"));
    }

    let warning_label = Label::new(Some("⚠ Switching models will download the new model and delete the old one."));
    warning_label.set_wrap(true);
    warning_label.add_css_class("dim-label");

    let save_ai_btn = Button::with_label("Save & Apply");

    let progress_bar = gtk4::ProgressBar::new();
    progress_bar.set_visible(false);
    progress_bar.set_show_text(true);

    ai_box.append(&model_label);
    ai_box.append(&model_combo);
    ai_box.append(&warning_label);
    ai_box.append(&progress_bar);
    ai_box.append(&save_ai_btn);
    
    let ctx_ai = ctx.clone();
    let combo_ai = model_combo.clone();
    let dialog_weak_ai = dialog.downgrade();
    let pb_weak = progress_bar.downgrade();
    
    save_ai_btn.connect_clicked(move |btn| {
        if let Some(new_model) = combo_ai.active_id() {
             let new_model_str = new_model.to_string();
             let old_model_str = ctx_ai.settings.get_ai_model();
             
             if new_model_str != old_model_str {
                 // Disable button
                 btn.set_sensitive(false);
                 btn.set_label("Downloading Model... Please Wait");
                 
                 let ctx_bg = ctx_ai.clone();
                 let btn_weak = btn.downgrade();
                 let dialog_weak_bg = dialog_weak_ai.clone();
                 let pb_weak_bg = pb_weak.clone();
                 
                 let (sender, receiver) = std::sync::mpsc::channel();
                 let (progress_tx, progress_rx) = std::sync::mpsc::channel();
                 
                 if let Some(pb) = pb_weak.upgrade() {
                     pb.set_visible(true);
                     pb.set_fraction(0.0);
                 }

                 std::thread::spawn(move || {
                     println!("Pulling model: {}", new_model_str);
                     // 1. Pull new with progress
                     let _ = ctx_bg.omnibar.llm.pull_model(&new_model_str, Box::new(move |p| {
                         let _ = progress_tx.send(p);
                     }));
                     
                     println!("Deleting old model: {}", old_model_str);
                     // 2. Delete old
                     if !old_model_str.is_empty() {
                         let _ = ctx_bg.omnibar.llm.delete_model(&old_model_str);
                     }
                     
                     // 3. Update settings
                     let _ = ctx_bg.settings.set_ai_model(new_model_str.clone());
                     
                     // 4. Update Adapter state so queries use it immediately
                     ctx_bg.omnibar.llm.set_model(&new_model_str);
                     
                     let _ = sender.send(());
                 });
                 
                 let pb_weak_poll = pb_weak.clone();
                 glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                     // Check progress
                     while let Ok(p) = progress_rx.try_recv() {
                         if let Some(pb) = pb_weak_poll.upgrade() {
                             pb.set_fraction(p);
                             pb.set_text(Some(&format!("{:.0}%", p * 100.0)));
                         }
                     }

                     if let Ok(_) = receiver.try_recv() {
                         if let Some(d) = dialog_weak_bg.upgrade() {
                             d.close();
                         }
                         return glib::ControlFlow::Break;
                     }
                     glib::ControlFlow::Continue
                 });
             } else {
                 if let Some(d) = dialog_weak_ai.upgrade() { d.close(); }
             }
        }
    });

    ai_box.append(&warning_label);
    ai_box.append(&save_ai_btn);

    notebook.append_page(&ai_box, Some(&Label::new(Some("AI"))));

    dialog.set_child(Some(&notebook));
    dialog.present();
}

fn add_section_items(row_box: &gtk4::Box, items: Vec<crate::domain::model::App>, ctx: &AppContext, window: &ApplicationWindow) {
    for item in items {
        let btn = Button::new();
        btn.add_css_class("grid-item");
        btn.set_width_request(100);
        btn.set_height_request(130);
        
        let vbox = gtk4::Box::new(Orientation::Vertical, 5);
        if let Some(icon_name) = &item.icon {
            let img = gtk4::Image::new();
            if std::path::Path::new(icon_name).exists() {
                img.set_from_file(Some(icon_name));
            } else {
                img.set_icon_name(Some(icon_name));
            }
            img.set_pixel_size(48); // Slightly larger icon for grid
            vbox.append(&img);
        }
        
        let label = Label::new(Some(&item.name));
        label.set_wrap(true);
        label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
        label.set_max_width_chars(12); // Approximate chars per line for 100px
        label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        label.set_lines(3);
        label.set_justify(gtk4::Justification::Center);
        label.set_valign(gtk4::Align::Start);
        vbox.append(&label);
        
        btn.set_child(Some(&vbox));
        
        let exec_cmd = item.exec_path.clone();
        let ctx_clone = ctx.clone();
        let win_clone = window.clone();
        
        btn.connect_clicked(move |_| {
            if exec_cmd.starts_with("internal:") {
                 // Simplified handling for now, ideally match full logic
                 if exec_cmd == "internal:quit" {
                     if let Some(app) = win_clone.application() { app.quit(); }
                 } else if exec_cmd == "internal:settings" {
                     show_settings_dialog(&win_clone, &ctx_clone);
                 }
            } else {
                ctx_clone.execute_command.execute(&exec_cmd);
                win_clone.set_visible(false);
            }
        });
        
        row_box.append(&btn);
    }
}

fn show_overview_grid(
    container: &gtk4::Box, 
    ctx: &AppContext, 
    window: &ApplicationWindow, 
    entry: &Entry
) {
    // Clear existing
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    let data = ctx.omnibar.get_overview();

    // Helper to create sections
    // We need to pass entry to the closure to capture it for the first section
    let entry_weak = entry.downgrade();
    
    let create_section = move |title: &str, items: Vec<crate::domain::model::App>| {
        if items.is_empty() { return; }
        
        let label = Label::new(Some(title));
        label.add_css_class("section-title");
        label.set_halign(gtk4::Align::Start);
        container.append(&label);

        // Horizontal Scroll
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Automatic)
            .vscrollbar_policy(gtk4::PolicyType::Never)
            .min_content_height(100)
            .build();

        let row_box = gtk4::Box::new(Orientation::Horizontal, 10);
        row_box.set_margin_bottom(10);
        
        // Add items
        add_section_items(&row_box, items, ctx, window);
        
        scroll.set_child(Some(&row_box));
        
        // Navigation Controller
        let controller = gtk4::EventControllerKey::new();
        let entry_weak_inner = entry_weak.clone();

        controller.connect_key_pressed(move |ctrl, key, _, _| {
            let widget = ctrl.widget().expect("Controller should be attached to widget");
            // widget is row_box
            let row_box = widget.downcast::<gtk4::Box>().unwrap();
            
            // NOTE: Left/Right is handled natively by container focus if items are buttons?
            // Actually GTK Box navigation uses Tab usually. 
            // We might need to handle Left/Right if default doesn't work well.
            // But let's verify Up/Down first.

            if key == gtk4::gdk::Key::Up {
                // Focus: 
                // 1. Find ScrolledWindow parent of this box
                // 2. Find Label sibling before ScrolledWindow
                // 3. Find ScrolledWindow before Label?
                
                if let Some(scroll_parent) = row_box.parent() { // ScrolledWindow
                     if let Some(prev_lbl) = scroll_parent.prev_sibling() { // Label
                         if let Some(prev_scroll_widget) = prev_lbl.prev_sibling() { // ScrolledWindow
                             if let Ok(prev_scroll) = prev_scroll_widget.downcast::<ScrolledWindow>() {
                                 if let Some(child_widget) = prev_scroll.child() {
                                     // child_widget is the viewport or the box? 
                                     // Usually viewport if using adjustments, but here we set_child box directly
                                     // Let's assume it's the Box (or Viewport -> Box). 
                                     // If ScrolledWindow wraps it in a Viewport automatically...
                                     
                                     // The built child is direct if we use set_child (GTK4).
                                     // Let's try casting to Box.
                                     
                                     let target_box = if let Ok(b) = child_widget.clone().downcast::<gtk4::Box>() {
                                         b 
                                     } else if let Ok(vp) = child_widget.downcast::<gtk4::Viewport>() {
                                          if let Some(vp_child) = vp.child() {
                                              if let Ok(b) = vp_child.downcast::<gtk4::Box>() { b } else { return gtk4::glib::Propagation::Proceed; }
                                          } else { return gtk4::glib::Propagation::Proceed; }
                                     } else {
                                         return gtk4::glib::Propagation::Proceed;
                                     };

                                     // Focus first child of target box
                                     if let Some(first) = target_box.first_child() {
                                         first.grab_focus();
                                         return gtk4::glib::Propagation::Stop;
                                     }
                                 }
                             }
                         }
                     }
                     // Top of list? -> Focus Entry
                     if let Some(entry) = entry_weak_inner.upgrade() {
                         entry.grab_focus();
                         entry.set_position(-1);
                         return gtk4::glib::Propagation::Stop;
                     }
                }
            } else if key == gtk4::gdk::Key::Down {
                if let Some(scroll_parent) = row_box.parent() { 
                     if let Some(next_lbl) = scroll_parent.next_sibling() { // Label
                         if let Some(next_scroll_widget) = next_lbl.next_sibling() { // ScrolledWindow
                             // Focus it
                             if let Ok(next_scroll) = next_scroll_widget.downcast::<ScrolledWindow>() {
                                 if let Some(child_widget) = next_scroll.child() {
                                     let target_box = if let Ok(b) = child_widget.clone().downcast::<gtk4::Box>() {
                                         b 
                                     } else if let Ok(vp) = child_widget.downcast::<gtk4::Viewport>() {
                                          if let Some(vp_child) = vp.child() {
                                              if let Ok(b) = vp_child.downcast::<gtk4::Box>() { b } else { return gtk4::glib::Propagation::Proceed; }
                                          } else { return gtk4::glib::Propagation::Proceed; }
                                     } else {
                                         return gtk4::glib::Propagation::Proceed;
                                     };

                                     if let Some(first) = target_box.first_child() {
                                         first.grab_focus();
                                         return gtk4::glib::Propagation::Stop;
                                     }
                                 }
                             }
                         }
                     }
                }
            }
            
            gtk4::glib::Propagation::Proceed
        });

        row_box.add_controller(controller);
        container.append(&scroll);
    };

    create_section("Applications", data.apps);
    create_section("Folders", data.folders);
    create_section("Shortcuts", data.shortcuts);
    create_section("Macros", data.macros);
    create_section("AI", data.ai);
    create_section("Settings", data.settings);
    create_section("System", data.system);
}

