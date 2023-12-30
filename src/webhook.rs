use std::collections::HashMap;

use chrono::{DateTime, Utc};
use colored::Colorize;
use reqwest::{header::CONTENT_TYPE, Client};
use serde::{Deserialize, Serialize};

use crate::{config::SETTINGS, score::Score};

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

            request_data.embeds.push(get_embed(new, prev, level_title))
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

fn get_default_footer() -> Footer {
    let version = env!("CARGO_PKG_VERSION");

    Footer {
        text: format!("WR Checker/{} By VilleOlof", version),
        url: String::from(GITHUBLINK),
        icon_url: String::from(DISCORDPFP),
    }
}

const GITHUBLINK: &str = "https://github.com/VilleOlof";
const DISCORDPFP: &str = "https://cdn.discordapp.com/attachments/365772775832420353/1144432467013533757/discord_pfp.webp";
const THUMBNAILURL: &str =
    "https://cdn.discordapp.com/emojis/592218899441909760.webp?size=96&quality=lossless";

fn get_embed(new: &Score, prev: &Score, level_title: String) -> Embed {
    Embed {
        r#type: String::from("rich"),
        title: String::from("New Ultra World Record!"),
        description: format!(
            "Level: **{}**\nImprovement: -**{}**",
            level_title,
            prev.time - new.time
        ),
        color: 15844367,
        timestamp: new.updated_at,
        footer: get_default_footer(),
        thumbnail: Some(Thumbnail {
            url: String::from(THUMBNAILURL),
        }),
        fields: vec![
            Field {
                name: String::from("New:"),
                value: format!(
                    "{}\n{}\n{}\n",
                    new.get_formatted_time(),
                    new.username,
                    new.platform
                ),
                inline: true,
            },
            Field {
                name: String::from("Old:"),
                value: format!(
                    "{}\n{}\n{}\n",
                    prev.get_formatted_time(),
                    prev.username,
                    prev.platform
                ),
                inline: true,
            },
        ],
    }
}

#[derive(Debug, Serialize)]
struct WebhookRequest {
    embeds: Vec<Embed>,
}

#[derive(Debug, Deserialize)]
struct WebhookResponse {
    id: String,
    //...
    // https://discord.com/developers/docs/resources/channel#message-object
}

#[derive(Debug, Serialize, Clone)]
struct Embed {
    r#type: String,
    title: String,
    description: String,
    color: u32,
    timestamp: DateTime<Utc>,
    footer: Footer,
    thumbnail: Option<Thumbnail>,
    fields: Vec<Field>,
}

#[derive(Debug, Serialize, Clone)]
struct Footer {
    text: String,
    url: String,
    icon_url: String,
}

#[derive(Debug, Serialize, Clone)]
struct Thumbnail {
    url: String,
}

#[derive(Debug, Serialize, Clone)]
struct Field {
    name: String,
    value: String,
    inline: bool,
}
