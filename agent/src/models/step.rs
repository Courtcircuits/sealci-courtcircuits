use super::{container::ContainerOperations, error::Error};
use crate::models::container::exec_handle::ExecResult;
use std::sync::Arc;

#[derive(Clone)]
pub struct Step<T: ContainerOperations> {
    /// This is the command that will be executed in the container
    pub command: String,

    /// This is the directory in which the command will be executed
    execute_in: Option<String>,

    /// Container
    container: Arc<T>,
}

impl<T: ContainerOperations> Step<T> {
    pub fn new(command: String, execute_in: Option<String>, container: Arc<T>) -> Self {
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
#[cfg(test)]
mod tests {
    use crate::models::container::mock::MockContainer;

    use super::*;
    use std::sync::Mutex;

    #[tokio::test]
    async fn test_step_execute_passes_command_to_container() {
        // Setup
        let container = Arc::new(MockContainer {
            exec_calls: Mutex::new(Vec::new()),
            should_fail: false,
        });

        let command = "echo 'test'".to_string();
        let workdir = Some("/tmp".to_string());

        let step = Step::new(command.clone(), workdir.clone(), container.clone());

        // Execute
        let _ = step.execute().await;

        // Verify
        let calls = container.exec_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, command);
        assert_eq!(calls[0].1, workdir);
    }

    #[tokio::test]
    async fn test_step_execute_handles_error_from_container() {
        // Setup
        let container = Arc::new(MockContainer {
            exec_calls: Mutex::new(Vec::new()),
            should_fail: true, // Configure to return an error
        });

        let step = Step::new("any command".to_string(), None, container);

        // Execute & Verify
        let result = step.execute().await;
        assert!(result.is_err());

        // Check it's the right type of error
        match result {
            Err(Error::ContainerExecError(_)) => (), // Expected
            _ => panic!("Unexpected result"),
        }
    }
}
