use std::sync::Arc;

use bollard::Docker;
use tokio::sync::mpsc::UnboundedSender;
use tonic::Status;

use crate::{
    models::{
        action::Action,
        container::{Container, ContainerOperations},
        error::Error,
    },
    proto::ActionResponseStream,
};

pub struct ActionService {
    docker_client: Arc<Docker>,
}

impl ActionService {
    pub fn new(docker_client: Arc<Docker>) -> Self {
        Self { docker_client }
    }

    pub async fn create(
        &self,
        image: String,
        commands: Vec<String>,
        log_input: UnboundedSender<Result<ActionResponseStream, Status>>,
        repo_url: String,
        action_id: u32,
    ) -> Result<Action<Container>, Error> {
        let container = Container::new(image, self.docker_client.clone());
        container.start().await?;
        let action = Action::new(action_id, container, commands, log_input, repo_url);
        action.setup_repository().await?;
        Ok(action)
    }
}
