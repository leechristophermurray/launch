use crate::domain::model::App;
use crate::interface::ports::{IAppRepository, IProcessMonitor, IFileSystem, ISystemPower, ICalculator, IShortcutRepository, IMacroRepository};
use std::sync::Arc;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

pub struct Omnibar {
    app_repo: Arc<dyn IAppRepository + Send + Sync>,
    process_monitor: Arc<dyn IProcessMonitor + Send + Sync>,
    fs: Arc<dyn IFileSystem + Send + Sync>,
    pub shortcuts: Arc<dyn IShortcutRepository + Send + Sync>,
    pub macros: Arc<dyn IMacroRepository + Send + Sync>,
    power: Arc<dyn ISystemPower + Send + Sync>,
    calculator: Arc<dyn ICalculator + Send + Sync>,
}

impl Omnibar {
    pub fn new(
        app_repo: Arc<dyn IAppRepository + Send + Sync>,
        process_monitor: Arc<dyn IProcessMonitor + Send + Sync>,
        fs: Arc<dyn IFileSystem + Send + Sync>,
        shortcuts: Arc<dyn IShortcutRepository + Send + Sync>,
        macros: Arc<dyn IMacroRepository + Send + Sync>,
        power: Arc<dyn ISystemPower + Send + Sync>,
        calculator: Arc<dyn ICalculator + Send + Sync>,
    ) -> Self {
        Self {
            app_repo,
            process_monitor,
            fs,
            shortcuts,
            macros,
            power,
            calculator,
        }
    }

    pub fn search(&self, query: &str) -> Vec<App> {
        if query.is_empty() {
            // Default: Show running apps or top apps
            let mut apps = self.app_repo.find_apps();
            self.process_monitor.update_app_status(&mut apps);
            apps.sort_by(|a, b| {
                b.is_running.cmp(&a.is_running).then_with(|| a.name.cmp(&b.name))
            });
            return apps;
        }

        if let Some(term_cmd) = query.strip_prefix("x ") {
             let cmd = term_cmd.trim();
             if !cmd.is_empty() {
                 let wrapped_cmd = format!(
                     "if command -v gnome-terminal >/dev/null 2>&1; then gnome-terminal -- {}; elif command -v ptyxis >/dev/null 2>&1; then ptyxis --standalone -- {}; else x-terminal-emulator -e {}; fi", 
                     cmd, cmd, cmd
                 );
                 return vec![App {
                     name: format!("Execute: {}", cmd),
                     exec_path: wrapped_cmd,
                     icon: Some("utilities-terminal".to_string()),
                     is_running: false,
                 }];
             }
             return vec![];
        }

        if let Some(fs_query) = query.strip_prefix("f ") {
            let path_input = fs_query.trim();
            // Basic ~ expansion
            let search_path = if path_input.starts_with("~") {
                if let Ok(home) = std::env::var("HOME") {
                    path_input.replacen("~", &home, 1)
                } else {
                    path_input.to_string()
                }
            } else if path_input.is_empty() {
                std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
            } else {
                path_input.to_string()
            };

            let path = std::path::Path::new(&search_path);
            let (dir, prefix) = if search_path.ends_with('/') || self.fs.is_dir(&search_path) {
                if self.fs.is_dir(&search_path) {
                    (search_path.as_str(), "")
                } else {
                     // Dir doesn't exist yet but ends with /, fallback or empty?
                     // Logic from before: treat parent
                     if self.fs.exists(&search_path) {
                          (search_path.as_str(), "")
                     } else {
                         // parent
                         if let Some(parent) = path.parent() {
                             (parent.to_str().unwrap_or("/"), path.file_name().and_then(|s| s.to_str()).unwrap_or(""))
                         } else {
                             ("/", "")
                         }
                     }
                }
            } else {
                 if let Some(parent) = path.parent() {
                     (parent.to_str().unwrap_or("/"), path.file_name().and_then(|s| s.to_str()).unwrap_or(""))
                 } else {
                     ("/", "")
                 }
            };

            let entries = self.fs.list_dir(dir);
            let mut results = vec![];
            for name in entries {
                if name.starts_with('.') { continue; }
                if !prefix.is_empty() && !name.to_lowercase().starts_with(&prefix.to_lowercase()) { continue; }
                
                let full_path = format!("{}/{}", dir.trim_end_matches('/'), name);
                let is_dir = self.fs.is_dir(&full_path);
                let display_name = if is_dir { format!("{}/", name) } else { name.clone() };
                let icon = if is_dir { "folder" } else { "text-x-generic" };
                let exec = if is_dir {
                    format!("nautilus \"{}\"", full_path)
                } else {
                    format!("xdg-open \"{}\"", full_path)
                };

                results.push(App {
                    name: display_name,
                    exec_path: exec,
                    icon: Some(icon.to_string()),
                    is_running: false,
                });
            }
             // Sort: Directories first, then alphabetical
            results.sort_by(|a, b| {
                let a_is_dir = a.name.ends_with('/');
                let b_is_dir = b.name.ends_with('/');
                if a_is_dir && !b_is_dir { std::cmp::Ordering::Less }
                else if !a_is_dir && b_is_dir { std::cmp::Ordering::Greater }
                else { a.name.cmp(&b.name) }
            });
            return results.into_iter().take(20).collect();
        }

        if let Some(sc_query) = query.strip_prefix("ss ") {
             let key = sc_query.trim();
             if let Some(cmd) = self.shortcuts.get(key) {
                 return vec![App {
                     name: format!("Shortcut: {}", key),
                     exec_path: cmd,
                     icon: Some("emblem-symbolic-link".to_string()),
                     is_running: false,
                 }];
             }
             // Show all shortcuts matching?
             let all = self.shortcuts.get_all();
             return all.into_iter()
                 .filter(|(k, _)| k.contains(key))
                 .map(|(k, v)| App {
                     name: format!("Shortcut: {}", k),
                     exec_path: v,
                     icon: Some("emblem-symbolic-link".to_string()),
                     is_running: false
                 })
                 .collect();
        }
        
        if let Some(calc_query) = query.strip_prefix("c ") {
             let expr = calc_query.trim();
             if let Some(result) = self.calculator.calculate(expr) {
                  return vec![App {
                      name: format!("= {}", result),
                      exec_path: format!("echo \"{}\" | xclip -selection clipboard", result), // Copy to clipboard
                      icon: Some("accessories-calculator".to_string()),
                      is_running: false,
                  }];
             }
             return vec![];
        }

        if let Some(m_query) = query.strip_prefix("m ") {
             let name = m_query.trim();
             if let Some(_mac) = self.macros.get(name) {
                 return vec![App {
                     name: format!("Macro: {}", name),
                     // Encode the macro execution as a special internal command
                     // Or just return it. The executor needs to know how to handle it.
                     // Let's use internal:macro:name
                     exec_path: format!("internal:macro:{}", name),
                     icon: Some("system-run".to_string()),
                     is_running: false,
                 }];
             }
             // Show all macros matching
             let all = self.macros.get_all();
             return all.into_iter()
                 .filter(|m| m.name.contains(name))
                 .map(|m| App {
                     name: format!("Macro: {}", m.name),
                     exec_path: format!("internal:macro:{}", m.name),
                     icon: Some("system-run".to_string()),
                     is_running: false
                 })
                 .collect();
        }

        if let Some(sys_query) = query.strip_prefix("! ") {
            let action = sys_query.trim();
            // predefined actions
            let actions = vec![
                "suspend", "reboot", "poweroff", "lock", "hibernate",
                "mute", "mute_mic", "mute_all",
                "toggle_night_light", "toggle_dark_mode", "toggle_dnd"
            ];
            return actions.into_iter()
                .filter(|a| a.starts_with(action))
                .map(|a| App {
                    name: format!("System: {}", a),
                    exec_path: format!("internal:system:{}", a),
                    icon: Some("system-shutdown".to_string()),
                    is_running: false
                }).collect();
        }
        
        if let Some(l_query) = query.strip_prefix("l ") {
             // internal settings
             let items = vec![
                App { 
                    name: "About Launch".to_string(), 
                    exec_path: "internal:about".to_string(), 
                    icon: Some("help-about".to_string()), 
                    is_running: false 
                },
                App { 
                    name: "Quit".to_string(), 
                    exec_path: "internal:quit".to_string(), 
                    icon: Some("application-exit".to_string()), 
                    is_running: false 
                },
                App { 
                    name: "Settings".to_string(), 
                    exec_path: "internal:settings".to_string(), 
                    icon: Some("preferences-system".to_string()), 
                    is_running: false 
                },
            ];
            return items.into_iter()
                .filter(|app| app.name.to_lowercase().contains(&l_query.trim().to_lowercase()))
                .collect();
        }

        // App Search (Default)
        let mut apps = self.app_repo.find_apps();
        self.process_monitor.update_app_status(&mut apps);
        
        let matcher = SkimMatcherV2::default();
        let mut scored_apps: Vec<(i64, App)> = apps.into_iter().filter_map(|app| {
            let score = matcher.fuzzy_match(&app.name, query)?;
            Some((score, app))
        }).collect();


        scored_apps.sort_by(|a, b| b.0.cmp(&a.0));
        scored_apps.into_iter().map(|(_, app)| app).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::MacroAction;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mocks
    struct MockAppRepo;
    impl IAppRepository for MockAppRepo {
        fn find_apps(&self) -> Vec<App> { vec![] }
    }

    struct MockProcessMonitor;
    impl IProcessMonitor for MockProcessMonitor {
        fn get_running_pids(&self) -> Vec<u32> { vec![] }
        fn update_app_status(&self, _apps: &mut [App]) {}
    }

    struct MockFS;
    impl IFileSystem for MockFS {
        fn list_dir(&self, _path: &str) -> Vec<String> { vec![] }
        fn is_dir(&self, _path: &str) -> bool { false }
        fn exists(&self, _path: &str) -> bool { false }
    }

    struct MockShortcuts;
    impl IShortcutRepository for MockShortcuts {
        fn get(&self, key: &str) -> Option<String> {
            if key == "term" { Some("gnome-terminal".to_string()) } else { None }
        }
        fn get_all(&self) -> HashMap<String, String> { HashMap::new() }
    }

    struct MockPower;
    impl ISystemPower for MockPower {
        fn execute(&self, _action: &str) -> Result<(), String> { Ok(()) }
    }

    struct MockCalculator;
    impl ICalculator for MockCalculator {
        fn calculate(&self, expression: &str) -> Option<String> {
            if expression.contains("1+1") { Some("2".to_string()) } else { None }
        }
    }

    struct MockMacro;
    impl IMacroRepository for MockMacro {
        fn get(&self, name: &str) -> Option<Macro> {
            if name == "test" { 
                Some(Macro { name: "test".to_string(), actions: vec![MacroAction::Command("x echo hi".to_string())] }) 
            } else { None }
        }
        fn get_all(&self) -> Vec<Macro> { vec![] }
        fn add(&self, _mac: Macro) -> Result<(), String> { Ok(()) }
        fn remove(&self, _name: &str) -> Result<(), String> { Ok(()) }
    }

    fn create_omnibar() -> Omnibar {
        Omnibar::new(
            Arc::new(MockAppRepo),
            Arc::new(MockProcessMonitor),
            Arc::new(MockFS),
            Arc::new(MockShortcuts),
            Arc::new(MockMacro),
            Arc::new(MockPower),
            Arc::new(MockCalculator),
        )
    }

    #[test]
    fn test_routes_terminal() {
        let omnibar = create_omnibar();
        let results = omnibar.search("x echo hello");
        assert_eq!(results.len(), 1);
        assert!(results[0].name.contains("Execute: echo hello"));
    }

    #[test]
    fn test_routes_shortcut() {
        let omnibar = create_omnibar();
        let results = omnibar.search("ss term");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].exec_path, "gnome-terminal");
    }

    #[test]
    fn test_routes_macro() {
        let omnibar = create_omnibar();
        let results = omnibar.search("m test");
        assert_eq!(results.len(), 1);
        assert!(results[0].name.contains("Macro: test"));
        assert_eq!(results[0].exec_path, "internal:macro:test");
    }

    #[test]
    fn test_routes_calculator() {
        let omnibar = create_omnibar();
        let results = omnibar.search("c 1+1");
        assert_eq!(results.len(), 1);
        assert!(results[0].name.contains("= 2"));
    }

    #[test]
    fn test_routes_system() {
        let omnibar = create_omnibar();
        let results = omnibar.search("! reboot");
        assert!(results.len() >= 1);
        assert!(results[0].name.contains("System: reboot"));
    }
}
