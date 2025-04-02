use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;

use super::{container::Container, error::Error, step::Step};

pub struct Action {
    id: String,
    container: Arc<Container>,
    steps: Vec<Step>,
    stdout: Arc<UnboundedSender<String>>,
    repository_url: String,
}

impl Action {
    pub fn new(
        id: String,
        container: Container,
        commands: Vec<String>,
        stdout: Arc<UnboundedSender<String>>,
        repository_url: String,
    ) -> Self {
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
            stdout,
        }
    }

    pub async fn execute(&self) -> Result<(), Error> {
        for step in &self.steps {
            let exec_result = step.execute().await?;
            let exit_status = exec_result.exec_handle.await;
            // TO DO: Handle logs
            if let Ok(exit_code) = exit_status {
                if exit_code != 0 {
                    break;
                }
            }
        }
        Ok(())
    }

    pub async fn cleanup(&self) -> Result<(), Error> {
        self.container.remove().await
    }
}
