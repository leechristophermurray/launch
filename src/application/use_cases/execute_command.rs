use crate::interface::ports::ICommandExecutor;
use crate::interface::ports::IMacroRepository;
use crate::interface::ports::ISystemPower;
use crate::application::use_cases::omnibar::Omnibar;
use crate::domain::model::MacroAction;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::process::Command;

pub struct ExecuteCommand {
    executor: Arc<dyn ICommandExecutor + Send + Sync>,
    macros: Arc<dyn IMacroRepository + Send + Sync>,
    omnibar: Arc<Omnibar>,
    system: Arc<dyn ISystemPower + Send + Sync>,
}

impl ExecuteCommand {
    pub fn new(
        executor: Arc<dyn ICommandExecutor + Send + Sync>,
        macros: Arc<dyn IMacroRepository + Send + Sync>,
        omnibar: Arc<Omnibar>,
        system: Arc<dyn ISystemPower + Send + Sync>,
    ) -> Self {
        Self { executor, macros, omnibar, system }
    }

    pub fn execute(&self, cmd: &str) {
        if let Some(macro_name) = cmd.strip_prefix("internal:macro:") {
             self.execute_macro(macro_name);
             return;
        }

        if let Some(sys_action) = cmd.strip_prefix("internal:system:") {
             if let Err(e) = self.system.execute(sys_action) {
                 println!("System action failed: {}", e);
             }
             return;
        }

        self.executor.execute(cmd);
    }
    
    fn execute_macro(&self, name: &str) {
        if let Some(mac) = self.macros.get(name) {
            println!("Executing Macro: {}", name);
            for action in mac.actions {
                match action {
                    MacroAction::LaunchApp(app_name) => {
                         // Search via Omnibar to resolve "Firefox" -> "firefox"
                         let results = self.omnibar.search(&app_name);
                         if let Some(top) = results.first() {
                             self.execute(&top.exec_path);
                         }
                    },
                    MacroAction::Command(cmd) => {
                         self.executor.execute(&cmd);
                    },
                    MacroAction::OpenUrl(url) => {
                         let cmd = format!("xdg-open \"{}\"", url);
                         self.executor.execute(&cmd);
                    },
                    MacroAction::TypeText(text) => {
                        // Using xdotool/wtype fallback
                        // Check for xdotool presence? Or just try?
                        let _ = Command::new("xdotool").arg("type").arg(&text).spawn();
                    },
                    MacroAction::Sleep(ms) => {
                        thread::sleep(Duration::from_millis(ms));
                    },
                    MacroAction::System(sys_action) => {
                        if let Err(e) = self.system.execute(&sys_action) {
                            println!("Macro System Action failed: {}", e);
                        }
                    }
                }
            }
        }
    }
}
