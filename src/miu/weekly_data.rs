//! Fetch and orders data related to weekly challenges

use std::{collections::HashMap, str::FromStr};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use serde::Deserialize;
use serde_with::{serde_as, EnumMap};

use crate::{config::SETTINGS, miu::score::Results, request::raw_request};

/// An entire weekly challenge
#[derive(Debug, Deserialize, Clone)]
pub struct Weekly {
    /// The internal parse objectid
    #[serde(rename = "objectId")]
    pub object_id: String,

    /// The level id, this ones a bit off but its often `CHALLENGE_DATA` for the current week
    #[serde(rename = "LevelID")]
    pub level_id: String,

    /// Created at, in UTC
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    /// Updated at, in UTC
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,

    /// The scorebucket, contains levels, names, start/end dates and physics mods
    #[serde(rename = "ScoreBuckets")]
    pub score_buckets: ScoreBucket,
}

impl FromStr for Weekly {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mid = match serde_json::from_str::<Results<MidWeekly>>(s) {
            Ok(mid) => match mid.results {
                Some(results) => {
                    if results.is_empty() {
                        return Err(format!("Results is empty when mid weekly parsing"));
                    }

                    results.first().unwrap().clone()
                }
                None => return Err(format!("Results is none when mid weekly parsing")),
            },
            Err(err) => return Err(format!("Failed to parse mid weekly: {}", err)),
        };

        let score_buckets = match serde_json::from_str::<ScoreBucket>(&mid.score_buckets) {
            Ok(s_bucket) => s_bucket,
            Err(err) => return Err(format!("Failed to parse score bucket: {}", err)),
        };

        Ok(Weekly {
            object_id: mid.object_id,
            level_id: mid.level_id,
            created_at: mid.created_at,
            updated_at: mid.updated_at,
            score_buckets,
        })
    }
}

impl Weekly {
    /// Returns an entire Weekly challenge
    ///
    /// Fetches and parses the data from the server
    pub async fn fetch(client: &Client) -> Result<Weekly> {
        let url = match Url::parse_with_params(
            &format!(
                "https://{}/parse/classes/{}",
                &SETTINGS.read().unwrap().parse.domain,
                &SETTINGS.read().unwrap().parse.weekly.class_name_stats
            ),
            &[("where", r#"{"LevelID": "CHALLENGE_DATA"}"#)],
        ) {
            Ok(url) => url,
            Err(err) => return Err(anyhow!("Failed to build weekly data url: {}", err)),
        };

        let resp = match raw_request(client, url).await {
            Ok(resp) => resp,
            Err(err) => return Err(anyhow!("Failed to fetch weekly data: {}", err)),
        };

        if let Ok(text) = resp.text().await {
            return match Weekly::from_str(&text) {
                Ok(weekly) => Ok(weekly),
                Err(err) => Err(anyhow!("Failed to get weekly from str: {}", err)),
            };
        }

        Err(anyhow!("Failed to return weekly data"))
    }
}

/// This is because ScoreBuckets is a json string, inside a json response
/// Only for internal deserialization
#[derive(Debug, Deserialize, Clone)]
struct MidWeekly {
    #[serde(rename = "objectId")]
    object_id: String,

    #[serde(rename = "LevelID")]
    level_id: String,

    #[serde(rename = "createdAt")]
    created_at: DateTime<Utc>,

    #[serde(rename = "updatedAt")]
    updated_at: DateTime<Utc>,

    #[serde(rename = "ScoreBuckets")]
    score_buckets: String,
}

/// Holds the current and previous challenge
///
/// And some internal sheet ids and stuff
#[derive(Debug, Deserialize, Clone)]
pub struct ScoreBucket {
    /// The current challenge
    pub current: Challenge,
    /// The previous challenge
    pub previous: Challenge,

    /// The sheet id of the challenge
    #[serde(rename = "sheetID")]
    pub sheet_id: i32,

    /// the cur id?
    #[serde(rename = "curID")]
    pub cur_id: i32,

    /// i honestly dont know what this is help, look at the raw response or something idk
    pub level: String,
}

/// A challenge, contains levels, name translation, start and end dates
#[derive(Debug, Deserialize, Clone)]
pub struct Challenge {
    /// The chapter set
    #[serde(rename = "chapterSet")]
    pub chapter_set: String,

    /// The challenge id
    #[serde(rename = "challengeID")]
    pub challenge_id: String,

    /// The levels, contains physics mods and level titles
    pub levels: Vec<ChallengeLevel>,
    /// Name translation, where key is the language and value is the corresponding name
    ///
    /// Use `Challenge::get_name(&self, lang: NameLang)` instead to get the translated name
    pub name: HashMap<String, String>,

    /// The start date of the challenge, in Utc
    #[serde(rename = "startDate")]
    pub start_date: DateTime<Utc>,

    /// The end date of the challenge, in Utc
    #[serde(rename = "endDate")]
    pub end_date: DateTime<Utc>,
}

impl Challenge {
    /// Returns a translated name of the challenge
    pub fn get_name(&self, lang: NameLang) -> String {
        if let Some(name) = self.name.get(&lang.to_string()) {
            return name.to_owned();
        }

        String::from("Unknown")
    }
}

/// A level in a challenge
#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct ChallengeLevel {
    /// The fancy level title
    pub name: String,
    /// The map id, *probably contains the "SP_###"*
    pub id: String,
    /// All physics mod for the level
    #[serde_as(as = "EnumMap")]
    pub physicsmod: Vec<PhysicsMod>,
}

/// All physics mods to ever exist.
///
/// Every mod has a value with it.
#[derive(Debug, Deserialize, Clone)]
pub enum PhysicsMod {
    /// Changes the gravity
    #[serde(rename = "gravity")]
    Gravity(f32),

    /// Changes the jump height
    #[serde(rename = "jumpmult")]
    JumpMult(f32),

    /// Changes the jump height
    #[serde(rename = "jumpforce")]
    JumpForce(f32),

    /// Changes the bounce height
    #[serde(rename = "bouncemult")]
    BounceMult(f32),

    /// Changes the scale of the marble
    #[serde(rename = "scalemult")]
    ScaleMult(f32),

    /// Changes the mass of the marble
    #[serde(rename = "massmult")]
    MassMult(f32),

    /// Changes the friction of the marble
    #[serde(rename = "frictionmult")]
    FrictionMult(f32),

    /// Changes the blast jump height
    #[serde(rename = "blastjumpmult")]
    BlastJumpMult(f32),

    /// Changes the blast push "scale"
    #[serde(rename = "blastpushmult")]
    BlastPushMult(f32),

    /// Changes the blast range
    #[serde(rename = "blastrangemult")]
    BlastRangeMult(f32),

    /// Changes the cooldown for the blast
    #[serde(rename = "blastcooldownmult")]
    BlastCooldownMult(f32),

    /// Changes the X roll speed
    #[serde(rename = "rollX")]
    RollX(f32),

    /// Changes the Y roll speed
    #[serde(rename = "rollY")]
    RollY(f32),

    /// If the marble can blast or not
    #[serde(rename = "canblast")]
    CanBlast(bool),

    /// How many airjumps is allowed
    #[serde(rename = "airjumps")]
    AirJumps(i32),

    /// If no powerups should spawn or not
    #[serde(rename = "nopowerups")]
    NoPowerups(bool),

    /// If the levels start and end should be reversed
    #[serde(rename = "reverse")]
    Reverse(bool),

    /// If each checkpoint should also spawn a gem
    #[serde(rename = "checkpointgems")]
    CheckpointGems(bool),

    /// Disables all gems
    #[serde(rename = "nogems")]
    NoGems(bool),

    /// Removes all time travels
    #[serde(rename = "notimetravel")]
    NoTimeTravel(bool),

    /// Replaces the trophy with a gem
    #[serde(rename = "trophygem")]
    TrophyGem(bool),

    /// Replaces the trophy with an end goal
    #[serde(rename = "trophyend")]
    TrophyEnd(bool),

    /// Makes the level a boomerang
    ///
    /// Spawn at the start, go to the end and back to the start
    #[serde(rename = "boomerang")]
    Boomerang(bool),

    /// Start with a specific powerup
    #[serde(rename = "startpowerup")]
    StartPowerup(String),

    /// Replaces all powerups with a specific one
    #[serde(rename = "replacepowerup")]
    ReplacePowerup(String),

    /// Changes the speed for all platforms
    #[serde(rename = "platformspeed")]
    PlatformSpeed(f32),

    /// Changes the X Blast "scale"
    #[serde(rename = "blastX")]
    BlastX(f32),

    /// Changes the Y Blast "scale"
    #[serde(rename = "blastY")]
    BlastY(f32),

    /// Changes the X Impact
    #[serde(rename = "impX")]
    ImpactX(f32),

    /// Changes the Y Impact
    #[serde(rename = "impY")]
    ImpactY(f32),

    /// To use sounds or not
    #[serde(rename = "usesounds")]
    UseSounds(bool),

    /// If the marble should have mega force
    #[serde(rename = "megaforce")]
    MegaForce(f32),

    /// Enable or disables full shadows
    #[serde(rename = "fullshadow")]
    FullShadow(bool),

    /// If the Multiplayer spawn offset is enabled
    #[serde(rename = "mpspawnoffset")]
    MPSpawnOffset(bool),
}

fn float_to_perct(f: &f32) -> String {
    format!("{}%", f * 100.0)
}

impl ToString for PhysicsMod {
    fn to_string(&self) -> String {
        match self {
            PhysicsMod::Gravity(v) => format!("Gravity: {}", float_to_perct(v)),
            PhysicsMod::JumpMult(v) => format!("Jump Height: {}", float_to_perct(v)),
            PhysicsMod::JumpForce(v) => format!("Jump Force: {}", float_to_perct(v)),
            PhysicsMod::BounceMult(v) => format!("Bounce Force: {}", float_to_perct(v)),
            PhysicsMod::ScaleMult(v) => format!("Marble Size: {}", float_to_perct(v)),
            PhysicsMod::MassMult(v) => format!("Mass: {}", float_to_perct(v)),
            PhysicsMod::FrictionMult(v) => format!("Friction Force: {}", float_to_perct(v)),
            PhysicsMod::BlastJumpMult(v) => format!("Blast Height: {}", float_to_perct(v)),
            PhysicsMod::BlastPushMult(v) => format!("Blast Push: {}", float_to_perct(v)),
            PhysicsMod::BlastRangeMult(v) => format!("Blast Range: {}", float_to_perct(v)),
            PhysicsMod::BlastCooldownMult(v) => format!("Blast Cooldown: {}", float_to_perct(v)),
            PhysicsMod::RollX(v) => format!("Roll Force X: {}", float_to_perct(v)),
            PhysicsMod::RollY(v) => format!("Roll Force Y: {}", float_to_perct(v)),
            PhysicsMod::CanBlast(_) => String::from("Blast Available"),
            PhysicsMod::AirJumps(v) => format!("Air Jumps: {}", v),
            PhysicsMod::NoPowerups(_) => String::from("No Powerups"),
            PhysicsMod::Reverse(_) => String::from("Level Reversed"),
            PhysicsMod::CheckpointGems(_) => String::from("Checkpoints Add Gems"),
            PhysicsMod::NoGems(_) => String::from("No Gems"),
            PhysicsMod::NoTimeTravel(_) => String::from("No Time Travels"),
            PhysicsMod::TrophyGem(_) => String::from("Trophy Adds Gem"),
            PhysicsMod::TrophyEnd(_) => String::from("Trophy is Goal"),
            PhysicsMod::StartPowerup(v) => format!("Start With: {}", v),
            PhysicsMod::ReplacePowerup(v) => format!("Replace Powerups: {}", v),
            PhysicsMod::PlatformSpeed(v) => format!("Platform Speed: {}", float_to_perct(v)),
            PhysicsMod::BlastX(v) => format!("Blast X: {}", float_to_perct(v)),
            PhysicsMod::BlastY(v) => format!("Blast Y: {}", float_to_perct(v)),
            PhysicsMod::ImpactX(v) => format!("Impact X: {}", float_to_perct(v)),
            PhysicsMod::ImpactY(v) => format!("Impact Y: {}", float_to_perct(v)),
            PhysicsMod::UseSounds(_) => format!("Use Sounds"),
            PhysicsMod::MegaForce(v) => format!("Mega Force: {}", float_to_perct(v)),
            PhysicsMod::FullShadow(_) => format!("Full Shadow"),
            PhysicsMod::MPSpawnOffset(_) => format!("MP Spawn Offset"),
            _ => String::from(""),
        }
    }
}

/// All the languages for Challenge names
#[allow(dead_code)]
pub enum NameLang {
    /// English
    En,
    /// Spanish
    Es,
    /// French
    Fr,
    /// German
    De,
    /// Italy
    It,
    /// Japanese
    Jp,
    /// Arabic
    Ar,
    /// Chinese
    ZhCh,
    /// Taiwainese
    ZhTw,
    /// Netherlands
    Nl,
    /// Korean
    Ko,
    /// Portgual
    Pt,
    /// Russian
    Ru,
    /// Turkey
    Tr,
}
impl ToString for NameLang {
    fn to_string(&self) -> String {
        String::from(match self {
            NameLang::En => "en",
            NameLang::Es => "es",
            NameLang::Fr => "fr",
            NameLang::De => "de",
            NameLang::It => "it",
            NameLang::Jp => "jp",
            NameLang::Ar => "ar",
            NameLang::ZhCh => "zh-CN",
            NameLang::ZhTw => "zh-TW",
            NameLang::Nl => "nl",
            NameLang::Ko => "ko",
            NameLang::Pt => "pt",
            NameLang::Ru => "ru",
            NameLang::Tr => "tr",
        })
    }
}
