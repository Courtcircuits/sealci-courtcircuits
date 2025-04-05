use std::{pin::Pin, sync::Mutex};

use bollard::container::LogOutput;
use futures_util::{stream, Stream};

use crate::models::error::Error;

use super::{exec_handle::ExecResult, ContainerOperations};

// A simpler mock implementation of ContainerOperations
pub struct MockContainer {
    // Track what was passed to exec
    pub exec_calls: Mutex<Vec<(String, Option<String>)>>,
    // Configure if exec should return success or error
    pub should_fail: bool,
}

impl ContainerOperations for MockContainer {
    async fn start(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn exec(&self, command: String, workdir: Option<String>) -> Result<ExecResult, Error> {
        // Record the call
        self.exec_calls.lock().unwrap().push((command, workdir));

        if self.should_fail {
            return Err(Error::ContainerExecError(bollard::errors::Error::from(
                std::io::Error::new(std::io::ErrorKind::Other, "Mock exec error"),
            )));
        }

        // Create a simple empty stream for the output
        let empty_stream = Box::pin(stream::empty::<Result<LogOutput, bollard::errors::Error>>())
            as Pin<Box<dyn Stream<Item = Result<LogOutput, bollard::errors::Error>> + Send>>;

        // Create a task that just returns 0
        let handle = tokio::task::spawn(async { 0 });

        Ok(ExecResult {
            output: empty_stream,
            exec_handle: handle,
        })
    }

    async fn remove(&self) -> Result<(), Error> {
        Ok(())
    }
}
