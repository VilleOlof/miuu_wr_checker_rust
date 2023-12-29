use std::{collections::HashMap, env::var};

use chrono::{DateTime, Utc};
use colored::Colorize;
use reqwest::{header::CONTENT_TYPE, Client};
use serde::Serialize;

use crate::score::Score;

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

        if let Ok(url) = get_webhook_url() {
            match client
                .post(url)
                .json(&request_data)
                .header(CONTENT_TYPE, "application/json")
                .send()
                .await
            {
                Err(err) => println!(
                    "{}: {}",
                    "Failed to send webhook to discord".red().bold(),
                    err
                ),
                _ => (),
            };
        }
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
        footer: Footer {
            text: String::from("WR Checker By VilleOlof"),
            url: String::from(GITHUBLINK),
            icon_url: String::from(DISCORDPFP),
        },
        thumbnail: Thumbnail {
            url: String::from(THUMBNAILURL),
        },
        fields: vec![
            Field {
                name: String::from("New:"),
                value: format!("{}\n{}\n{}\n", new.time, new.username, new.platform),
                inline: true,
            },
            Field {
                name: String::from("Old:"),
                value: format!("{}\n{}\n{}\n", prev.time, prev.username, prev.platform),
                inline: true,
            },
        ],
    }
}

pub fn get_webhook_url() -> Result<String, String> {
    match var("DISCORD_WEBHOOK") {
        Ok(str) => Ok(str),
        Err(err) => return Err(format!("Env Discord Webhook Error: {:?}", err).into()),
    }
}

#[derive(Debug, Serialize)]
struct WebhookRequest {
    embeds: Vec<Embed>,
}

#[derive(Debug, Serialize)]
struct Embed {
    r#type: String,
    title: String,
    description: String,
    color: u32,
    timestamp: DateTime<Utc>,
    footer: Footer,
    thumbnail: Thumbnail,
    fields: Vec<Field>,
}

#[derive(Debug, Serialize)]
struct Footer {
    text: String,
    url: String,
    icon_url: String,
}

#[derive(Debug, Serialize)]
struct Thumbnail {
    url: String,
}

#[derive(Debug, Serialize)]
struct Field {
    name: String,
    value: String,
    inline: bool,
}
