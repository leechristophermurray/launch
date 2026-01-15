use crate::interface::ports::IWindowRepository;
use crate::domain::model::Window;
use std::process::Command;

pub struct SystemWindowAdapter;

impl SystemWindowAdapter {
    pub fn new() -> Self {
        Self
    }
    
    fn run_wmctrl_list() -> Vec<Window> {
        // Output format: 0x04e00003  0 dpkg-configure-a-service  Title
        // -x flag adds class: 0x04e00003  0 gnome-terminal-server.Gnome-terminal  clmxone  Terminal
        let output = match Command::new("wmctrl").arg("-l").arg("-x").output() {
            Ok(o) => o,
            Err(_) => return vec![], // wmctrl missing
        };
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut windows = vec![];
        
        for line in stdout.lines() {
            // parsing logic
            // columns: items: <id> <desktop> <class> <machine> <title...>
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 { continue; }
            
            let id = parts[0].to_string();
            let desktop_id = parts[1].parse::<i32>().unwrap_or(-1);
            let class = parts[2].to_string();
            // part 3 is machine
            let title = parts[4..].join(" ");
            
            // Cleanup class (e.g. gnome-terminal-server.Gnome-terminal -> Gnome-terminal)
            let app_name = if let Some(idx) = class.find('.') {
                class[idx+1..].to_string()
            } else {
                class
            };
            
            windows.push(Window {
                id,
                title,
                app_name,
                workspace: desktop_id,
                screen: -1,
            });
        }
        windows
    }

    // Attempts to get window list using gdbus for Gnome Shell.
    // Returns Some(Vec<Window>) on success, None on failure (e.g., gdbus not found, parsing error).
    fn run_gdbus_list() -> Option<Vec<Window>> {
        let output = Command::new("gdbus")
            .arg("call")
            .arg("--session")
            .arg("--dest")
            .arg("org.gnome.Shell")
            .arg("--object-path")
            .arg("/org/gnome/Shell/Introspect")
            .arg("--method")
            .arg("org.gnome.Shell.Introspect.GetWindows")
            .output()
            .ok()?; // Return None if gdbus command fails or cannot be run

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // gdbus output is typically like: "([{'id': <id>, 'title': <title>, ...}])"
        // We need to extract the JSON array part.
        // This is a simplified parsing assuming a specific structure.
        let json_str = stdout.trim();
        if json_str.starts_with("([") && json_str.ends_with("])") {
            let inner_json = &json_str[2..json_str.len() - 2];
            // Replace single quotes with double quotes for valid JSON
            let valid_json = inner_json.replace('\'', "\"");

            // Split into individual JSON objects and parse
            let mut windows = vec![];
            // This is a very naive split and parse. A proper JSON library (like serde_json)
            // would be much more robust here.
            for obj_str in valid_json.split("}, {") {
                let mut obj_str_cleaned = obj_str.to_string();
                if !obj_str_cleaned.starts_with('{') {
                    obj_str_cleaned.insert(0, '{');
                }
                if !obj_str_cleaned.ends_with('}') {
                    obj_str_cleaned.push('}');
                }

                // Attempt to parse key-value pairs
                let mut id = String::new();
                let mut title = String::new();
                let mut app_name = String::new();
                let mut workspace = -1;
                let mut screen = -1;

                for pair in obj_str_cleaned.trim_matches('{').trim_matches('}').split(',') {
                    let parts: Vec<&str> = pair.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim().trim_matches('"');
                        let value = parts[1].trim().trim_matches('"');

                        match key {
                            "id" => id = value.to_string(),
                            "title" => title = value.to_string(),
                            "app_name" => app_name = value.to_string(),
                            "workspace" => workspace = value.parse::<i32>().unwrap_or(-1),
                            "screen" => screen = value.parse::<i32>().unwrap_or(-1),
                            _ => {}
                        }
                    }
                }

                if !id.is_empty() {
                    windows.push(Window {
                        id,
                        title,
                        app_name,
                        workspace,
                        screen,
                    });
                }
            }
            Some(windows)
        } else {
            None // Gdbus output format not as expected
        }
    }
    
    fn run_wmctrl_focus(id: &str) -> Result<(), String> {
        let status = Command::new("wmctrl")
            .arg("-i")
            .arg("-a")
            .arg(id)
            .status()
            .map_err(|e| e.to_string())?;
            
        if status.success() {
            Ok(())
        } else {
            Err("wmctrl failed".to_string())
        }
    }
}

impl IWindowRepository for SystemWindowAdapter {
    fn get_open_windows(&self) -> Vec<Window> {
        // Attempt to use gdbus first for Gnome Shell
        if let Some(windows) = Self::run_gdbus_list() {
            if !windows.is_empty() {
                return windows;
            }
        }

        // Fallback to wmctrl if gdbus fails or returns no windows
        Self::run_wmctrl_list()
    }

    fn focus_window(&self, id: &str) -> Result<(), String> {
         Self::run_wmctrl_focus(id)
    }
}
