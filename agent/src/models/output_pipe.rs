use tokio::sync::mpsc::UnboundedSender;
use tonic::Status;

use crate::proto::{ActionResponseStream, ActionResult};


/// An output pipe is used to stream the output of an action.
/// It is directly associated with an action and provides a way to send logs and results back to the client.
pub struct OutputPipe {
    action_id: u32,
    pipe: UnboundedSender<Result<ActionResponseStream, Status>>,
}

impl OutputPipe {
    pub fn new(action_id: u32, pipe: UnboundedSender<Result<ActionResponseStream, Status>>) -> Self {
        Self { action_id, pipe }
    }

    pub fn output_log(&self, log: String, completion: i32, exit_code: Option<i32>) {
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
