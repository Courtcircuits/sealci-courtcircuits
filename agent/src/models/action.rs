use std::sync::Arc;

use tokio::{sync::mpsc::UnboundedSender, task};
use tokio_stream::StreamExt;
use tonic::Status;

use crate::proto::ActionResponseStream;

use super::{
    container::Container,
    error::Error::{self, StepOutputError},
    output_pipe::OutputPipe,
    step::Step,
};

pub struct Action {
    id: u32,
    container: Arc<Container>,
    steps: Vec<Step>,
    pipe: Arc<OutputPipe>,
    repository_url: String,
}

impl Action {
    pub fn new(
        id: u32,
        container: Container,
        commands: Vec<String>,
        stdout: UnboundedSender<Result<ActionResponseStream, Status>>,
        repository_url: String,
    ) -> Self {
        let pipe = Arc::new(OutputPipe::new(id, stdout));
        let container = Arc::new(container);
        let steps: Vec<Step> = commands
            .iter()
            .map(|c| Step::new(c.into(), None, container.clone()))
            .collect();
        Self {
            id,
            container,
            steps,
            repository_url,
            pipe,
        }
    }

    pub async fn execute(&self) -> Result<(), Error> {
        for step in &self.steps {
            let mut exec_result = step.execute().await?;
            let pipe = self.pipe.clone();
            task::spawn(async move {
                while let Some(log) = exec_result.output.next().await {
                    let log_output = match log {
                        Ok(log_output) => log_output.to_string(),
                        Err(e) => return Err(Status::aborted(format!("Execution error: {}", e))),
                    };
                    pipe.output_log(log_output, 2, None);
                }
                Ok(())
            });
            let exit_status = exec_result.exec_handle.await;
            let pipe = self.pipe.clone();

            if let Ok(exit_code) = exit_status {
                if exit_code != 0 {
                    pipe.output_log("Action failed".to_string(), 3, Some(exit_code));
                    self.cleanup().await;
                    return Err(StepOutputError(exit_code));
                }
            }
        }
        Ok(())
    }

    pub async fn cleanup(&self) -> Result<(), Error> {
        self.container.remove().await
    }
}
