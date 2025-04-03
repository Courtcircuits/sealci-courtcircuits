use super::{container::Container, error::Error};
use crate::models::container::exec_handle::ExecResult;
use std::sync::Arc;

pub struct Step {
    /// This is the command that will be executed in the container
    pub command: String,

    /// This is the directory in which the command will be executed
    execute_in: Option<String>,

    /// Container
    container: Arc<Container>,
}

impl Step {
    pub fn new(command: String, execute_in: Option<String>, container: Arc<Container>) -> Self {
        
        Self {
            command,
            execute_in,
            container,
        }
    }

    /// Execute the command in the container
    pub async fn execute(&self) -> Result<ExecResult, Error> {
        self.container
            .exec(self.command.clone(), self.execute_in.clone())
            .await
    }
}
