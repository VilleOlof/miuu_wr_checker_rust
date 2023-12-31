use std::collections::HashMap;

use colored::Colorize;
use reqwest::{header::CONTENT_TYPE, Client};
use serde::{Deserialize, Serialize};

use crate::{
    config::SETTINGS,
    embed::{get_score_embed, Embed},
    score::Score,
};

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

async fn send_to_all_webhooks(client: &Client, embeds: &WebhookRequest) -> Vec<String> {
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

#[derive(Debug, Serialize)]
pub struct WebhookRequest {
    pub embeds: Vec<Embed>,
}

#[derive(Debug, Deserialize)]
struct WebhookResponse {
    id: String,
    //...
    // https://discord.com/developers/docs/resources/channel#message-object
}
