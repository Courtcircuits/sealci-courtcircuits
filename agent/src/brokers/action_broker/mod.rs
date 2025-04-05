use std::sync::Arc;

use crate::models::{action::Action, container::Container};

use super::Channel;

#[derive(Clone)]
pub struct ActionBroker {
    pub create_action_channel: Arc<Channel<Action<Container>>>,
    pub delete_action_channel: Arc<Channel<u32>>,
}

impl ActionBroker {
    pub fn new() -> Self {
        let create_action_channel = Arc::new(Channel::new());
        let delete_action_channel = Arc::new(Channel::new());
        Self {
            create_action_channel,
            delete_action_channel,
        }
    }
}
