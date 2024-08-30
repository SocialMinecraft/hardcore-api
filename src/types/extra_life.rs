use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtraLife {
    pub id: i64,
    pub player_uuid: String,
    pub stamp: DateTime<Utc>,
    pub reason: String,
    pub playtime: i32,
}