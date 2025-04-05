use tokio::sync::watch::{self, Receiver, Sender};
use tokio_stream::wrappers::WatchStream;

use crate::models::error::Error;

pub mod action_broker;
pub mod state_broker;

pub struct Channel<T: Sync + Send + Clone> {
    sender: Sender<Option<T>>,
    receiver: Receiver<Option<T>>,
}

impl<T: Sync + Send + Clone> Channel<T> {
    pub fn new() -> Self {
        let (sender, receiver) = watch::channel(None);
        Self { sender, receiver }
    }
}

pub trait Broker<T> {
    fn send_event(&self, event: T) -> Result<(), Error>;
    fn subscribe(&self) -> WatchStream<Option<T>>;
}

impl<T: Sync + Send + Clone + 'static> Broker<T> for Channel<T> {
    fn send_event(&self, event: T) -> Result<(), Error> {
        self.sender
            .send(Some(event))
            .map_err(|e| Error::ChannelError(e.to_string()))
    }

    fn subscribe(&self) -> WatchStream<Option<T>> {
        WatchStream::new(self.receiver.clone())
    }
}
