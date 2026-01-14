use crate::interface::ports::ICommandExecutor;
use std::sync::Arc;

pub struct ExecuteCommand {
    executor: Arc<dyn ICommandExecutor + Send + Sync>,
}

impl ExecuteCommand {
    pub fn new(executor: Arc<dyn ICommandExecutor + Send + Sync>) -> Self {
        Self { executor }
    }

    pub fn execute(&self, cmd: &str) {
        self.executor.execute(cmd);
    }
}
