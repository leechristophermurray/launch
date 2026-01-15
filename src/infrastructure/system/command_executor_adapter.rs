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
        let _ = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .spawn();
    }
}
