use std::cmp::Ordering;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use chrono::serde::ts_seconds;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Death {
    pub id: i64,
    pub player_uuid: String,
    #[serde(with = "ts_seconds")]
    pub stamp: DateTime<Utc>,
    pub playtime: i32,
    pub reason: String,
}

// it is possible for two events to have the exact same stamp... which would break some rules.

impl PartialOrd for Death {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.stamp.partial_cmp(&other.stamp) // should consider playtime
    }
}

impl Ord for Death {
    fn cmp(&self, other: &Self) -> Ordering {
        self.stamp.cmp(&other.stamp)
    }
}
