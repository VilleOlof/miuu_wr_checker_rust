use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{
    score::Score,
    weekly_data::{NameLang, Weekly},
};

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

pub fn get_score_embed(new: &Score, prev: &Score, level_title: String) -> Embed {
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
        image: None,
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

pub fn get_weekly_embed(weekly: &Weekly, previous_scores: &Vec<Score>) -> Embed {
    let curr_physics_mods = weekly
        .score_buckets
        .current
        .levels
        .first()
        .expect("somehow failed to get levels for the current week")
        .physicsmod
        .clone()
        .into_iter()
        .map(|p| p.to_string())
        .collect::<Vec<String>>();

    let mut prev_fields: Vec<Field> = vec![];
    for (i, score) in previous_scores.into_iter().enumerate() {
        let level = weekly.score_buckets.previous.levels[i].name.clone();

        prev_fields.push(Field {
            name: level,
            value: format!(
                "*{}: {} - {}*",
                score.platform,
                score.username,
                score.get_formatted_time()
            ),
            inline: true,
        });
    }

    Embed {
        r#type: String::from("rich"),
        title: String::from("***New Ultra Weekly Challenge Starts Now!***"),
        description: format!(
            "**Challenge: {}**\n{} \n \n**Previous Challenge Winners**\n{}",
            weekly.score_buckets.current.get_name(NameLang::En),
            curr_physics_mods.join("\n"),
            weekly.score_buckets.previous.get_name(NameLang::En)
        ),
        color: 5763719,
        timestamp: Utc::now(),
        footer: get_default_footer(),
        thumbnail: None,
        image: Some(Image {
            // Image taken from the OG challenge embed thing
            url: String::from("http://blueteak.io/img/portfolio/MIU_ChallengeSmall.png"),
            proxy_url: None,
            width: None,
            height: None,
        }),
        fields: prev_fields,
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Embed {
    r#type: String,
    title: String,
    description: String,
    color: u32,
    timestamp: DateTime<Utc>,
    footer: Footer,
    thumbnail: Option<Thumbnail>,
    image: Option<Image>,
    fields: Vec<Field>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Footer {
    text: String,
    url: String,
    icon_url: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Image {
    url: String,
    proxy_url: Option<String>,
    height: Option<i32>,
    width: Option<i32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Thumbnail {
    url: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Field {
    name: String,
    value: String,
    inline: bool,
}
