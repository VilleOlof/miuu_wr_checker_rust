//! Holds struct and the global static struct related to settings
//!
//! Read from the config.toml / config.json file

use std::sync::RwLock;

use config::Config;
use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
    /// Global `settings` across the entire program
    ///
    /// # Example
    ///
    /// ```rust
    /// use miuu_wr_checker_rust::config::SETTINGS;
    ///
    /// let db_url = &SETTINGS.read().unwrap().database_url;
    /// ```
    pub static ref SETTINGS: RwLock<Settings> = RwLock::new(
        Config::builder()
            .add_source(config::File::with_name("config"))
            .build()
            .unwrap()
            .try_deserialize::<Settings>()
            .unwrap()
    );
}

/// Contains all the settings from the config.toml file
#[derive(Debug, Deserialize)]
pub struct Settings {
    /// The filepath or URL to the sqlite database
    pub database_url: String,
    /// How long to wait between each main iteration
    pub loop_wait_seconds: u64,
    /// Sends an uptime request to kuma if filled in
    ///
    /// Mostly for my own personal uptime dashboard
    pub kuma_push_url: Option<String>,
    /// A struct that contains discord related settings
    pub discord: Discord,
    /// A struct that contains parse related settings
    pub parse: Parse,
}

/// Holds discord related settings
#[derive(Debug, Deserialize)]
pub struct Discord {
    /// A vec of discord webhook urls
    ///
    /// These urls are used for new world records
    /// and world record recap webhooks
    pub webhooks: Vec<String>,
    /// A vec of discord webhook urls
    ///
    /// These are used for weekly challenge announcement posts
    pub weekly_webhooks: Vec<String>,
}

/// Holds parse related settings
#[derive(Debug, Deserialize)]
pub struct Parse {
    /// The url base (Domain) for the miubackend
    pub domain: String,
    /// The appid used in request headers
    pub appid: String,
    /// The class name for normal MIU ingame leaderboards
    pub class_name: String,
    /// A struct that contains weekly challenge parse classes
    pub weekly: ParseWeekly,
}

/// Holds parse settings related to weekly challenges
#[derive(Debug, Deserialize)]
pub struct ParseWeekly {
    /// The class name for weekly challenge leaderboard class
    pub class_name: String,
    /// The class name for weekly challenge stats/data class
    pub class_name_stats: String,
}
