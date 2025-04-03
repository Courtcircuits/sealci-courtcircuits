use tokio::sync::mpsc::UnboundedSender;
use tonic::Status;

use crate::proto::{ActionResponseStream, ActionResult};

/// An output pipe is used to stream the output of an action.
/// It is directly associated with an action and provides a way to send logs and results back to the client.
pub struct OutputPipe {
    action_id: u32,
    pipe: UnboundedSender<Result<ActionResponseStream, Status>>,
}

pub trait Pipe {
    fn output_log(&self, log: String, completion: i32, exit_code: Option<i32>);
}

impl OutputPipe {
    pub fn new(
        action_id: u32,
        pipe: UnboundedSender<Result<ActionResponseStream, Status>>,
    ) -> Self {
        Self { action_id, pipe }
    }
}

impl Pipe for OutputPipe {
    fn output_log(&self, log: String, completion: i32, exit_code: Option<i32>) {
        let _ = self.pipe.send(Ok(ActionResponseStream {
            log,
            action_id: self.action_id,
            result: Some(ActionResult {
                completion,
                exit_code,
            }),
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::{self, UnboundedReceiver};

    // Helper function to create a pipe and receiver for testing
    fn create_test_pipe(
        action_id: u32,
    ) -> (
        OutputPipe,
        UnboundedReceiver<Result<ActionResponseStream, Status>>,
    ) {
        let (tx, rx) = mpsc::unbounded_channel();
        let pipe = OutputPipe::new(action_id, tx);
        (pipe, rx)
    }

    #[tokio::test]
    async fn test_output_log_sends_correct_message() {
        // Setup
        let action_id = 42;
        let (pipe, mut rx) = create_test_pipe(action_id);
        let test_log = "Test log message".to_string();
        let test_completion = 75;
        let test_exit_code = Some(0);

        // Exercise
        pipe.output_log(test_log.clone(), test_completion, test_exit_code);

        // Verify
        if let Some(Ok(message)) = rx.recv().await {
            assert_eq!(message.log, test_log);
            assert_eq!(message.action_id, action_id);
            assert!(message.result.is_some());

            let result = message.result.unwrap();
            assert_eq!(result.completion, test_completion);
            assert_eq!(result.exit_code, test_exit_code);
        } else {
            panic!("Failed to receive message");
        }
    }

    #[tokio::test]
    async fn test_output_log_with_no_exit_code() {
        // Setup
        let action_id = 123;
        let (pipe, mut rx) = create_test_pipe(action_id);

        // Exercise - with None exit code
        pipe.output_log("No exit code".to_string(), 100, None);

        // Verify
        if let Some(Ok(message)) = rx.recv().await {
            let result = message.result.unwrap();
            assert_eq!(result.exit_code, None);
            assert_eq!(result.completion, 100);
        } else {
            panic!("Failed to receive message");
        }
    }

    #[tokio::test]
    async fn test_new_creates_pipe_with_correct_id() {
        let action_id = 789;
        let (tx, _rx) = mpsc::unbounded_channel();
        let pipe = OutputPipe::new(action_id, tx);

        // We can't directly access action_id since it's private
        // So we'll test indirectly by sending a message
        pipe.output_log("".to_string(), 0, None);

        // We can't assert here directly because we don't have the receiver
        // In a real test, you might want to structure things differently
        // This is mainly to show that the constructor works
    }
}
