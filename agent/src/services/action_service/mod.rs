use std::{collections::HashMap, sync::Arc};

use bollard::Docker;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::WatchStream;
use tonic::Status;

use crate::{
    brokers::{
        action_broker::ActionBroker,
        state_broker::{StateBroker, StateEvent},
        Broker,
    },
    models::{
        action::Action,
        container::{Container, ContainerOperations},
        error::Error,
    },
    proto::ActionResponseStream,
};

#[derive(Clone)]
pub struct ActionService {
    docker_client: Arc<Docker>,
    actions: HashMap<u32, Arc<Action<Container>>>,
    action_broker: ActionBroker,
    state_broker: Arc<StateBroker>,
}

impl ActionService {
    pub fn new(docker_client: Arc<Docker>, state_broker: Arc<StateBroker>) -> Self {
        let actions = HashMap::new();
        let action_broker = ActionBroker::new();
        Self {
            docker_client,
            actions,
            action_broker,
            state_broker,
        }
    }

    pub async fn create(
        &mut self,
        image: String,
        commands: Vec<String>,
        log_input: UnboundedSender<Result<ActionResponseStream, Status>>,
        repo_url: String,
        action_id: u32,
    ) -> Result<Action<Container>, Error> {
        let container = Container::new(image, self.docker_client.clone());
        container.start().await?;
        let action = Action::new(
            action_id,
            container,
            commands,
            log_input,
            repo_url,
            self.state_broker.clone(),
        );
        let action_arc = Arc::new(action.clone());
        action.setup_repository().await?;
        self.actions.insert(action.id, action_arc.clone());
        self.action_broker
            .create_action_channel
            .send_event(action.clone())?;
        Ok(action)
    }

    pub async fn delete(&mut self, action_id: u32) -> Result<(), Error> {
        let action = self
            .actions
            .remove(&action_id)
            .ok_or(Error::ActionNotFound)?;
        action.cleanup().await?;
        self.action_broker
            .delete_action_channel
            .send_event(action_id)?;
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Arc<Action<Container>>>, Error> {
        let mut actions: Vec<Arc<Action<Container>>> = Vec::new();
        for action in self.actions.values() {
            actions.push(action.to_owned());
        }
        Ok(actions)
    }

    pub async fn get(&self, action_id: u32) -> Result<Arc<Action<Container>>, Error> {
        self.actions
            .get(&action_id)
            .cloned()
            .ok_or(Error::ActionNotFound)
    }

    pub fn creation_stream(&self) -> WatchStream<Option<Action<Container>>> {
        self.action_broker.create_action_channel.subscribe()
    }

    pub fn state_stream(&self) -> WatchStream<Option<StateEvent>> {
        self.state_broker.state_channel.subscribe()
    }
}
