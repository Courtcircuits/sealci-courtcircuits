use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize,Display)]
pub enum State {
    #[default]
    InProgress = 0,
    Completed = 1,
    Failed = 2,
}
