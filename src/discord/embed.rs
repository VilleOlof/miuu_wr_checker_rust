//! Creates Discord embeds for different messages

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::miu::{
    score::{RecapScore, Score},
    weekly_data::{Challenge, NameLang, Weekly},
};

fn get_default_footer() -> Footer {
    let version = env!("CARGO_PKG_VERSION");

    Footer {
        text: format!("MIUU:OB/{} By VilleOlof", version),
        url: String::from(GITHUBLINK),
        icon_url: String::from(DISCORDPFP),
    }
}

const GITHUBLINK: &str = "https://github.com/VilleOlof";
const DISCORDPFP: &str = "https://cdn.discordapp.com/attachments/365772775832420353/1144432467013533757/discord_pfp.webp";
const THUMBNAILURL: &str =
    "https://cdn.discordapp.com/emojis/592218899441909760.webp?size=96&quality=lossless";

/// Gets an embed for world record announcements
pub fn get_score_embed(new: &Score, prev: &Score, level_title: String) -> Embed {
    Embed {
        r#type: String::from("rich"),
        title: String::from("***New Ultra World Record!***"),
        description: format!(
            "Level: **{}**\nImprovement: -**{:.6}**",
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

/// Gets an embed for the weekly challenge announcement post
pub fn get_weekly_embed(weekly: &Weekly, previous_scores: &Vec<Score>) -> Embed {
    fn get_physics_mods(challenge: &Challenge) -> Vec<String> {
        challenge
            .levels
            .first()
            .expect("somehow failed to get levels for the current week")
            .physicsmod
            .clone()
            .into_iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
    }

    let curr_physics_mods = get_physics_mods(&weekly.score_buckets.current);
    let prev_physics_mods = get_physics_mods(&weekly.score_buckets.previous);

    let levels = weekly
        .score_buckets
        .current
        .levels
        .clone()
        .into_iter()
        .map(|l| l.name)
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

    let mut fields = vec![
        Field {
            name: String::from("Current Modifiers:"),
            value: curr_physics_mods.join("\n").to_string(),
            inline: true,
        },
        Field {
            name: String::from("Current Levels:"),
            value: levels.join("\n").to_string(),
            inline: true,
        },
        Field {
            name: String::from("Previous Challenge:"),
            value: format!("{}", weekly.score_buckets.previous.get_name(NameLang::En)),
            inline: false,
        },
        Field {
            name: String::from("Previous Modifiers:"),
            value: prev_physics_mods.join("\n").to_string(),
            inline: false,
        },
    ];
    fields.append(&mut prev_fields);

    Embed {
        r#type: String::from("rich"),
        title: String::from("***New Ultra Weekly Challenge Starts Now!***"),
        description: format!(
            "**Current Challenge:**\n{}",
            weekly.score_buckets.current.get_name(NameLang::En),
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
        fields,
    }
}

/// Gets an embed for the world record weekly recap post
pub fn get_weekly_recap_embed(
    scores: Vec<RecapScore>,
    dates: (DateTime<Utc>, DateTime<Utc>),
) -> Embed {
    let date_format = &"%Y-%m-%d";

    let fields: Vec<Field> = scores
        .clone()
        .into_iter()
        .map(|s| Field {
            name: s.level,
            value: s
                .scores
                .into_iter()
                .map(|sub| format!("- {}: **{}**", sub.username, sub.get_formatted_time()))
                .collect::<Vec<String>>()
                .join("\n")
                + &format!("\n*Improvement:* ***-{:.6}***", s.improvement),
            inline: false,
        })
        .collect();

    Embed {
        r#type: String::from("rich"),
        title: String::from("***New Weekly Ultra WR Recap!***"),
        description: format!(
            "*Date: {}  >  {}*\nTotal New World Records: **{}**\nTotal Improvement: **-{}**",
            dates.0.format(date_format),
            dates.1.format(date_format),
            scores
                .clone()
                .into_iter()
                .map(|s| s.scores.len())
                .sum::<usize>(),
            scores
            .clone()
            .into_iter()
            .map(|s| s.improvement)
            .sum::<f32>()
        ),
        color: 3447003,
        timestamp: dates.1,
        footer: get_default_footer(),
        thumbnail: Some(Thumbnail {
            // Diamond Medal Emoji
            url: String::from("https://cdn.discordapp.com/emojis/500104801691107328.webp?size=96&quality=lossless")
        }),
        image: None,
        fields,
    }
}

/// A Discord embed
///
/// Doesn't contain all fields according to discord docs
///
/// Just those that are needed
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

/// A discord embed footer
#[derive(Debug, Serialize, Clone)]
pub struct Footer {
    text: String,
    url: String,
    icon_url: String,
}

/// A discord embed image
#[derive(Debug, Serialize, Clone)]
pub struct Image {
    url: String,
    proxy_url: Option<String>,
    height: Option<i32>,
    width: Option<i32>,
}

/// A discord embed thumbnail
#[derive(Debug, Serialize, Clone)]
pub struct Thumbnail {
    url: String,
}

/// A discord embed field
#[derive(Debug, Serialize, Clone)]
pub struct Field {
    name: String,
    value: String,
    inline: bool,
}
