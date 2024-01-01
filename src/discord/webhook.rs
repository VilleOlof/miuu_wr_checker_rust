//! Used to send webhooks to Discords API

use std::collections::HashMap;

use colored::Colorize;
use reqwest::{header::CONTENT_TYPE, Client};
use serde::{Deserialize, Serialize};

use crate::{
    config::SETTINGS,
    discord::embed::{get_score_embed, get_weekly_embed, Embed},
    miu::{score::Score, weekly_data::Weekly},
};

/// Sends World Record announcement message(s)
///
/// Given the tuple of scores, (`Vec<(new, previous)`)
pub async fn send_webhooks(
    client: &Client,
    scores: Vec<(Score, Score)>,
    name_conversion: &HashMap<String, String>,
) {
    for chunk in scores.chunks(10) {
        let mut request_data: WebhookRequest = WebhookRequest { embeds: vec![] };

        for (new, prev) in chunk {
            let level_title = if let Some(name) = name_conversion.get(&new.map_id[3..]) {
                name
            } else {
                println!("{}: {}", "Failed to convert level name".red(), new.map_id);
                &new.map_id
            }
            .to_owned();

            request_data
                .embeds
                .push(get_score_embed(new, prev, level_title))
        }

        send_to_all_webhooks(client, &request_data).await;
    }
}

/// Sends an embed to all webhooks in `SETTINGS.discord.webhooks`
pub async fn send_to_all_webhooks(client: &Client, embeds: &WebhookRequest) -> Vec<String> {
    let mut ids: Vec<String> = vec![];

    for url in &SETTINGS.read().unwrap().discord.webhooks {
        let response: WebhookResponse = match client
            .post(url.to_owned() + "?wait=true")
            .json(embeds)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
        {
            Err(err) => {
                println!(
                    "{}: {}, {}",
                    "Failed to send webhook to discord".red().bold(),
                    url,
                    err
                );
                continue;
            }
            Ok(res) => res
                .json::<WebhookResponse>()
                .await
                .expect("Failed to get webhook response"),
        };

        ids.push(response.id);
    }

    ids
}

/// Sends a weekly announcement embed to all webhooks in `SETTINGS.discord.weekly_webhooks`
pub async fn send_weekly_embed(client: &Client, weekly: &Weekly, previous_scores: &Vec<Score>) {
    let embed = get_weekly_embed(&weekly, &previous_scores);
    let request_struct = WebhookRequest {
        embeds: vec![embed],
    };

    for url in &SETTINGS.read().unwrap().discord.weekly_webhooks {
        match client
            .post(url.clone())
            .json(&request_struct)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
        {
            Ok(_) => (),
            Err(_) => println!("Failed to send challenge webhook"),
        };
    }
}

/// Webhook request, does not contain all Discord documented fields
#[derive(Debug, Serialize)]
pub struct WebhookRequest {
    /// A vec of embeds to send, limit of 10 at a time
    pub embeds: Vec<Embed>,
}

#[derive(Debug, Deserialize)]
struct WebhookResponse {
    id: String,
    //...
    // https://discord.com/developers/docs/resources/channel#message-object
}
