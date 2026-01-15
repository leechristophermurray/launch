use crate::interface::ports::ICommandExecutor;
use crate::interface::ports::IMacroRepository;
use crate::application::use_cases::omnibar::Omnibar;
use std::sync::Arc;

pub struct ExecuteCommand {
    executor: Arc<dyn ICommandExecutor + Send + Sync>,
    macros: Arc<dyn IMacroRepository + Send + Sync>,
    omnibar: Arc<Omnibar>,
}

impl ExecuteCommand {
    pub fn new(
        executor: Arc<dyn ICommandExecutor + Send + Sync>,
        macros: Arc<dyn IMacroRepository + Send + Sync>,
        omnibar: Arc<Omnibar>,
    ) -> Self {
        Self { executor, macros, omnibar }
    }

    pub fn execute(&self, cmd: &str) {
        if let Some(macro_name) = cmd.strip_prefix("internal:macro:") {
             self.execute_macro(macro_name);
             return;
        }

        self.executor.execute(cmd);
    }
    
    fn execute_macro(&self, name: &str) {
        if let Some(mac) = self.macros.get(name) {
            println!("Executing Macro: {}", name);
            for action in mac.actions {
                // Try to resolve the action as an Omnibar query first
                let results = self.omnibar.search(&action);
                
                if let Some(top_result) = results.first() {
                    let cmd_to_run = &top_result.exec_path;
                    
                    // Prevent infinite recursion if a macro calls itself (basic check)
                    // "internal:macro:name"
                    if cmd_to_run == &format!("internal:macro:{}", name) {
                        println!("Skipping recursive macro call to self: {}", name);
                        continue;
                    }

                    // Recursively execute
                    self.execute(cmd_to_run);
                } else {
                    // Fallback: If no Omnibar result (unlikely if 'x ' works, but possible),
                    // execute raw.
                    self.executor.execute(&action);
                }
            }
        }
    }
}
