use std::{collections::HashMap, sync::Arc};

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
    actions: HashMap<u32, Action<Container>>,
}

impl ActionService {
    pub fn new(docker_client: Arc<Docker>) -> Self {
        let actions = HashMap::new();
        Self {
            docker_client,
            actions,
        }
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

    pub async fn delete(&mut self, action_id: u32) -> Result<(), Error> {
        let action = self
            .actions
            .remove(&action_id)
            .ok_or(Error::ActionNotFound)?;
        action.cleanup().await?;
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Action<Container>>, Error> {
        let mut actions: Vec<Action<Container>> = Vec::new();
        for action in self.actions.values() {
            actions.push(action.to_owned());
        }
        Ok(actions)
    }

    pub async fn get(&self, action_id: u32) -> Result<Action<Container>, Error> {
        self.actions
            .get(&action_id)
            .cloned()
            .ok_or(Error::ActionNotFound)
    }
}
