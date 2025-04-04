use tokio::sync::watch::{self, Receiver, Sender};
use tokio_stream::wrappers::WatchStream;

use crate::models::error::Error;

pub mod action_borker;

pub struct Channel<T: Default + Sync + Send + Clone> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T: Default + Sync + Send + Clone> Channel<T> {
    pub fn new() -> Self {
        let (sender, receiver) = watch::channel(T::default());
        Self { sender, receiver }
    }
}

pub trait Broker<T> {
    fn send_event(&self, event: T) -> Result<(), Error>;
    fn subscribe(&self) -> WatchStream<T>;
}

impl<T: Default + Sync + Send + Clone + 'static> Broker<T> for Channel<T> {
    fn send_event(&self, event: T) -> Result<(), Error> {
        self.sender
            .send(event)
            .map_err(|e| Error::ChannelError(e.to_string()))
    }

    fn subscribe(&self) -> WatchStream<T> {
        WatchStream::new(self.receiver.clone())
    }
}
