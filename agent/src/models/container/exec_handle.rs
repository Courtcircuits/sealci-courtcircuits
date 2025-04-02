use std::pin::Pin;

use bollard::container::LogOutput;
use futures_util::Stream;
use tokio::task::JoinHandle;

pub struct ExecResult {
    pub output: Pin<Box<dyn Stream<Item = Result<LogOutput, bollard::errors::Error>> + Send>>,
    pub exec_handle: JoinHandle<i64>,
}

