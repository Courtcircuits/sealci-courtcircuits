use crate::models::{action::Action, container::Container};

pub enum Event {
    Creation,
    Deletion
}

pub struct ActionEvent {
    pub event: Event,
    pub id: String,
    pub action: Option<Action<Container>>
}
