use super::{container::ContainerOperations, error::Error::ExecError, step::Step};
use super::{
    error::Error::{self, StepOutputError},
    output_pipe::OutputPipe,
};
use crate::{models::output_pipe::Pipe, proto::ActionResponseStream};
use state::{ActionState, State};
use std::sync::Arc;
use tokio::{sync::mpsc::UnboundedSender, task};
use tokio_stream::StreamExt;
use tonic::Status;
use tracing::{debug, error};
mod state;

#[derive(Clone)]
pub struct Action<T: ContainerOperations> {
    pub id: u32,
    pub container: Arc<T>,
    steps: Vec<Step<T>>,
    pipe: Arc<OutputPipe>,
    pub repository_url: String,
    pub state: ActionState,
}

impl<T: ContainerOperations> Action<T> {
    pub fn new(
        id: u32,
        container: T,
        commands: Vec<String>,
        stdout: UnboundedSender<Result<ActionResponseStream, Status>>,
        repository_url: String,
    ) -> Self {
        let pipe = Arc::new(OutputPipe::new(id, stdout));
        let container = Arc::new(container);
        let steps: Vec<Step<T>> = commands
            .iter()
            .map(|c| Step::new(c.into(), Some(format!("/{}", id)), container.clone()))
            .collect();
        let state = ActionState::new();

        Self {
            id,
            container,
            steps,
            repository_url,
            pipe,
            state,
        }
    }

    pub async fn execute(&mut self) -> Result<(), Error> {
        for step in &self.steps {
            // Execute the step in the folder where we cloned the repository
            // When cloning we use the action id as a name for the folder
            let mut exec_result = step.execute().await?;
            let command = step.command.clone();
            debug!("Executing command {} for action {}", command, self.id);
            self.pipe.clone().output_log(command, 2, None);
            let pipe = self.pipe.clone();
            let id = self.id.clone();
            task::spawn(async move {
                while let Some(log) = exec_result.output.next().await {
                    match log {
                        Ok(log_output) => {
                            debug!("Command output: {} ", log_output);
                            pipe.output_log(log_output.to_string(), 2, None);
                        }
                        Err(e) => {
                            error!("Action {} failed: {}", id, e);
                            return Err(Status::aborted(format!("Execution error: {}", e)));
                        }
                    }
                }
                Ok(())
            });
            let exit_status = exec_result.exec_handle.await;

            if let Ok(exit_code) = exit_status {
                if exit_code != 0 {
                    self.cleanup().await?;
                    self.state.set(State::Completed)?;
                    self.pipe
                        .output_log("Action failed".to_string(), 3, Some(exit_code));
                    return Err(StepOutputError(exit_code));
                }
            }
        }
        self.cleanup().await?;
        self.state.set(State::Completed)?;
        Ok(())
    }

    pub async fn setup_repository(&self) -> Result<(), Error> {
        // Cloning the repository in a folder that takes as name the id of the action
        let setup_command = format!("git clone --depth 1 {} {}", self.repository_url, self.id);
        let exec_result = self.container.exec(setup_command, None).await?;
        exec_result.exec_handle.await.map_err(ExecError)?;
        Ok(())
    }

    pub async fn cleanup(&self) -> Result<(), Error> {
        self.container.remove().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::container::mock::MockContainer;
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_action_setup_repository_calls_exec_with_git_clone() {
        // Arrange
        let (tx, _rx) = mpsc::unbounded_channel();
        let mock_container = MockContainer {
            exec_calls: Mutex::new(Vec::new()),
            should_fail: false,
        };

        let action_id = 42;
        let repo_url = "https://github.com/user/repo.git".to_string();
        let action = Action::new(
            action_id,
            mock_container,
            vec!["echo 'test'".to_string()],
            tx,
            repo_url.clone(),
        );

        // Act
        let result = action.setup_repository().await;

        // Assert
        assert!(result.is_ok());

        let container = Arc::new(&action.container);
        let calls = container.exec_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);

        // Verify the git clone command has correct format
        let expected_cmd = format!("git clone --depth 1 {} {}", repo_url, action_id);
        assert_eq!(calls[0].0, expected_cmd);
        assert_eq!(calls[0].1, None); // No working directory for clone
    }

    #[tokio::test]
    async fn test_action_execute_runs_all_steps() {
        // Arrange
        let (tx, _rx) = mpsc::unbounded_channel();
        let mock_container = MockContainer {
            exec_calls: Mutex::new(Vec::new()),
            should_fail: false,
        };

        let commands = vec![
            "echo 'step 1'".to_string(),
            "echo 'step 2'".to_string(),
            "echo 'step 3'".to_string(),
        ];

        let mut action = Action::new(
            1,
            mock_container,
            commands.clone(),
            tx,
            "https://example.com/repo.git".to_string(),
        );

        // Act
        let result = action.execute().await;

        // Assert
        assert!(result.is_ok());

        let container = Arc::new(&action.container);
        let calls = container.exec_calls.lock().unwrap();

        // Check that all steps were executed
        assert_eq!(calls.len(), commands.len());

        // Verify each step was called with correct working directory
        for (i, command) in commands.iter().enumerate() {
            assert_eq!(&calls[i].0, command);
            assert_eq!(calls[i].1, Some("/1".to_string()));
        }
    }

    #[tokio::test]
    async fn test_action_execute_handles_step_failure() {
        // Arrange - Setup a mock that will fail on execution
        let (tx, _rx) = mpsc::unbounded_channel();
        let mock_container = MockContainer {
            exec_calls: Mutex::new(Vec::new()),
            should_fail: true,
        };

        let mut action = Action::new(
            1,
            mock_container,
            vec!["echo 'will fail'".to_string()],
            tx,
            "https://example.com/repo.git".to_string(),
        );

        // Act
        let result = action.execute().await;

        // Assert
        assert!(result.is_err());
        match result {
            Err(Error::ContainerExecError(_)) => (), // Expected error
            _ => panic!("Unexpected result"),
        }
    }

    #[tokio::test]
    async fn test_action_cleanup_removes_container() {
        // Arrange
        let (tx, _rx) = mpsc::unbounded_channel();
        let mock_container = MockContainer {
            exec_calls: Mutex::new(Vec::new()),
            should_fail: false,
        };

        let action = Action::new(
            1,
            mock_container,
            vec![],
            tx,
            "https://example.com/repo.git".to_string(),
        );

        // Act
        let result = action.cleanup().await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_action_new_initializes_with_correct_values() {
        // Arrange
        let (tx, _rx) = mpsc::unbounded_channel();
        let mock_container = MockContainer {
            exec_calls: Mutex::new(Vec::new()),
            should_fail: false,
        };

        let action_id = 99;
        let commands = vec!["cmd1".to_string(), "cmd2".to_string()];
        let repo_url = "https://example.com/test.git".to_string();

        // Act
        let action = Action::new(
            action_id,
            mock_container,
            commands.clone(),
            tx,
            repo_url.clone(),
        );

        // Assert
        assert_eq!(action.id, action_id);
        assert_eq!(action.repository_url, repo_url);
        assert_eq!(action.steps.len(), commands.len());
    }
}
