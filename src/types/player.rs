use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
//use sqlx::types::chrono;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub player_uuid: String,
    pub name: String,
    //pub joined: chrono::DataTime<chrono::Utc>,
    pub joined: DateTime<Utc>,
    pub playtime: i32,
}