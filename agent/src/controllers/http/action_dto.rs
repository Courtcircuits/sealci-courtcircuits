use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ActionDto {
    pub id: u32,
    pub state: String,
    pub repo_url: String,
    pub image: String
}