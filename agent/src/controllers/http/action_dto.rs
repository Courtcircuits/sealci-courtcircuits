use serde::{Deserialize, Serialize};

use crate::models::action::state::State;

#[derive(Serialize, Deserialize)]
pub struct ActionDto {
    pub id: u32,
    pub state: State,
    pub repo_url: String,
    pub image: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateActionRequest {
    pub image: String,
    pub commands: Vec<String>,
    pub repo_url: String,
    pub action_id: u32,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteActionRequest {
    pub id: u32,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteActionResponse {
    pub id: u32,
}
