//! Holds all structs related to scores

use std::time::Duration;

use serde::Deserialize;

use chrono::{DateTime, Utc};

/// A Replay struct
#[derive(Deserialize, Debug, Clone)]
pub struct Replay {
    /// The replay type, often a file
    #[serde(rename = "__type")]
    pub r#type: String,
    /// The name of the replay
    pub name: String,
    /// The url for the replay
    ///
    /// Kinda odd one since its a local host, use the name and replay module instead of this
    pub url: String,
}

/// A Score struct
///
/// Common across normal leaderboards and weekly challenges leaderboards
#[derive(Deserialize, Debug, Clone)]
pub struct Score {
    /// The time of the score
    pub time: f32,

    /// The user id, is differently formatted depending on platform
    #[serde(rename = "userID")]
    pub user_id: String,

    /// The username of whoevers score
    pub username: String,

    /// The raw mapid, includes `SP_###`
    #[serde(rename = "mapID")]
    pub map_id: String,

    /// The id of the skin used
    #[serde(rename = "skinUsed")]
    pub skin_used: String,

    /// The replay version
    #[serde(rename = "replayVersion")]
    pub replay_version: u32,

    /// The platform the score was performed on
    pub platform: String,
    /// The replay struct, contains further replay information
    pub replay: Option<Replay>,

    /// Created at, in Utc
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    /// Updated at, in Utc
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,

    /// Parse internal object id
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
}

/// A special score wrapper for weekly WR recaps
#[derive(Debug, Clone)]
pub struct RecapScore {
    /// The level for the scores
    ///
    /// Should be fancy title named
    pub level: String,
    /// The total improvement, in positive float
    pub improvement: f32,
    /// All the scores for the given level
    pub scores: Vec<Score>,
}

/// A Parse response, holds error data or the actual results
#[derive(Deserialize, Debug)]
pub struct Results<T> {
    /// The actual results, always stored in a `Vec`, generic
    ///
    /// `None` if the response contains an error
    pub results: Option<Vec<T>>,
    /// Holds the error code
    ///
    ///`None` if the response was successful
    pub code: Option<u32>,
    /// Holds the error message
    ///
    ///`None` if the response was successful
    pub error: Option<String>,
}

impl Score {
    /// Returns a formatted time
    ///
    /// In the format of: `MM:SS:MS` only if the time is above a minute,
    ///
    /// otherwise it just returns the time as string
    pub fn get_formatted_time(&self) -> String {
        if self.time < 60.0 {
            return self.time.to_string();
        }

        let dur = Duration::from_secs_f64(self.time as f64);
        let minutes = (dur.as_secs() / 60) % 60;
        let seconds = dur.as_secs() % 60;
        let millis = {
            let num = self.time.floor();
            let dec = self.time - num;

            format!("{:.6}", dec)[2..].to_string()
        };

        format!("{:0>2}:{:0>2}.{}", minutes, seconds, millis)
    }
}
