use crate::interface::ports::ICommandExecutor;
use std::process::Command;

pub struct SystemCommandExecutorAdapter;

impl SystemCommandExecutorAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl ICommandExecutor for SystemCommandExecutorAdapter {
    fn execute(&self, cmd: &str) {
        // Run detached command
        // cmd is likely "binary arg1 arg2", we should parse it or just use shell.
        // Using shell is easier for handling args but has security implications.
        // For a launcher, using shell (sh -c) is often acceptable or expected 
        // to handle env vars and expansion, but let's try direct execution first if possible,
        // or fallback to sh -c.
        
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if let Some((bin, args)) = parts.split_first() {
            let _ = Command::new(bin)
                .args(args)
                .spawn();
        }
    }
}
