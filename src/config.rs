use std::sync::RwLock;

use config::Config;
use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
    pub static ref SETTINGS: RwLock<Settings> = RwLock::new(
        Config::builder()
            .add_source(config::File::with_name("config"))
            .build()
            .unwrap()
            .try_deserialize::<Settings>()
            .unwrap()
    );
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database_url: String,
    pub discord: Discord,
    pub parse: Parse,
}

#[derive(Debug, Deserialize)]
pub struct Discord {
    pub webhooks: Vec<String>,
    pub weekly_webhooks: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Parse {
    pub domain: String,
    pub appid: String,
    pub class_name: String,
    pub weekly: ParseWeekly,
}

#[derive(Debug, Deserialize)]
pub struct ParseWeekly {
    pub class_name: String,
    pub class_name_stats: String,
}
