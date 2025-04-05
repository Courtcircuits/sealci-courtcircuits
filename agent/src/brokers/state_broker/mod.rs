use std::sync::Arc;

use crate::models::action::state::State;

use super::Channel;

#[derive(Debug, Clone)]
pub struct StateEvent {
    pub action_id: u32,
    pub state: State,
}

pub struct StateBroker {
    pub state_channel: Arc<Channel<StateEvent>>,
}

impl StateBroker {
    pub fn new() -> Self {
        Self {
            state_channel: Arc::new(Channel::new()),
        }
    }
}
