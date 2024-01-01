//! Functions and things related to pure Marble It Up! fetching

use crate::{
    embed,
    request::make_request,
    score::{RecapScore, Score},
    webhook::{self, WebhookRequest},
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use reqwest::Client;

/// Gets all world records for all the given levels
pub async fn get_wrs(client: &Client, levels: &Vec<String>) -> Result<Vec<Score>> {
    let mut level_futures = Vec::new();

    for level in levels {
        level_futures.push(fetch(&client, level));
    }

    try_join_all(level_futures).await
}

async fn fetch(client: &Client, level: &str) -> Result<Score> {
    let level_str = format!(r#"{{"mapID":"SP_{}"}}"#, level);
    let params = vec![
        ("limit", "1"),
        ("order", "time,-updatedAt"),
        ("where", &level_str),
    ];

    match make_request(client, params, None, None).await {
        Ok(mut score) => {
            if score.is_empty() {
                return Err(anyhow!("Empty Scores returned from: {}", level));
            }

            Ok(score.remove(0))
        }
        Err(err) => Err(anyhow!("[{}] Failed to fetch score, {}", level, err)),
    }
}

/// Sends out a weekly recap
///
/// Constructs an embed and sends it
pub async fn weekly_recap(
    client: &Client,
    scores: Vec<RecapScore>,
    dates: (DateTime<Utc>, DateTime<Utc>),
) {
    let embed = embed::get_weekly_recap_embed(scores, dates);
    let request = &WebhookRequest {
        embeds: vec![embed],
    };

    webhook::send_to_all_webhooks(client, request).await;
}
