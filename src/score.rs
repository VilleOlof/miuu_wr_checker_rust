use std::time::Duration;

use serde::Deserialize;

use chrono::{DateTime, Utc};

#[derive(Deserialize, Debug, Clone)]
pub struct Replay {
    #[serde(rename = "__type")]
    pub r#type: String,

    pub name: String,
    pub url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Score {
    pub time: f32,

    #[serde(rename = "userID")]
    pub user_id: String,

    pub username: String,

    #[serde(rename = "mapID")]
    pub map_id: String,

    #[serde(rename = "skinUsed")]
    pub skin_used: String,

    #[serde(rename = "replayVersion")]
    pub replay_version: u32,

    pub platform: String,
    pub replay: Option<Replay>,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,

    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Results {
    pub results: Option<Vec<Score>>,
    pub code: Option<u32>,
    pub error: Option<String>,
}

impl Score {
    pub fn get_formatted_time(&self) -> String {
        if self.time < 60.0 {
            return self.time.to_string();
        }

        let dur = Duration::from_secs_f64(self.time as f64);
        let minutes = (dur.as_secs() / 60) % 60;
        let seconds = dur.as_secs() % 60;
        let millis = {
            let num = self.time.floor();
            self.time - num
        };

        format!("{:0>2}:{:0>2}.{:0>2}", minutes, seconds, millis)
    }
}
