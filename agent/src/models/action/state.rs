use tokio::sync::
    watch::{self, Receiver, Sender}
;
use tokio_stream::wrappers::WatchStream;

use crate::models::error::Error;

#[derive(Clone)]
pub enum State {
    InProgress = 0,
    Completed = 1,
    Failed = 2,
}

#[derive(Clone)]
pub struct ActionState {
    sender: Sender<State>,
    receiver: Receiver<State>,
}

impl ActionState {
    pub fn new() -> Self {
        let (sender, receiver) = watch::channel(State::InProgress);
        ActionState { sender, receiver }
    }
    pub fn set(&mut self, new_state: State) -> Result<(), Error> {
        self.sender
            .send(new_state)
            .map_err(|_| Error::ActionStateError)?;
        Ok(())
    }

    pub fn get(&self) -> State {
        self.receiver.borrow().clone()
    }

    pub fn subscribe(&self) -> WatchStream<State> {
        WatchStream::new(self.receiver.clone())
    }
}
