use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use serde::Deserialize;
use serde_with::{serde_as, EnumMap};

use crate::{config::SETTINGS, request::raw_request, score::Results};

#[derive(Debug, Deserialize, Clone)]
pub struct Weekly {
    #[serde(rename = "objectId")]
    pub object_id: String,

    #[serde(rename = "LevelID")]
    pub level_id: String,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,

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
    pub async fn fetch(client: &Client) -> Result<Weekly, String> {
        let url = match Url::parse_with_params(
            &format!(
                "https://{}/parse/classes/{}",
                &SETTINGS.read().unwrap().parse.domain,
                &SETTINGS.read().unwrap().parse.weekly.class_name_stats
            ),
            &[("where", r#"{"LevelID": "CHALLENGE_DATA"}"#)],
        ) {
            Ok(url) => url,
            Err(err) => return Err(format!("Failed to build weekly data url: {}", err)),
        };

        let resp = match raw_request(client, url).await {
            Ok(resp) => resp,
            Err(err) => return Err(format!("Failed to fetch weekly data: {}", err)),
        };

        if let Ok(text) = resp.text().await {
            return match Weekly::from_str(&text) {
                Ok(weekly) => Ok(weekly),
                Err(err) => Err(format!("Failed to get weekly from str: {}", err)),
            };
        }

        Err(String::from("Failed to return weekly data"))
    }
}

// This is because ScoreBuckets is a json string, inside a json response
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

#[derive(Debug, Deserialize, Clone)]
pub struct ScoreBucket {
    pub current: Challenge,
    pub previous: Challenge,

    #[serde(rename = "sheetID")]
    pub sheet_id: i32,

    #[serde(rename = "curID")]
    pub cur_id: i32,

    pub level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Challenge {
    #[serde(rename = "chapterSet")]
    pub chapter_set: String,

    #[serde(rename = "challengeID")]
    pub challenge_id: String,

    pub levels: Vec<ChallengeLevel>,
    pub name: HashMap<String, String>,

    #[serde(rename = "startDate")]
    pub start_date: DateTime<Utc>,

    #[serde(rename = "endDate")]
    pub end_date: DateTime<Utc>,
}

impl Challenge {
    pub fn get_name(&self, lang: NameLang) -> String {
        if let Some(name) = self.name.get(&lang.to_string()) {
            return name.to_owned();
        }

        String::from("Unknown")
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct ChallengeLevel {
    pub name: String,
    pub id: String,
    #[serde_as(as = "EnumMap")]
    pub physicsmod: Vec<PhysicsMod>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum PhysicsMod {
    #[serde(rename = "gravity")]
    Gravity(f32),

    #[serde(rename = "jumpmult")]
    JumpMult(f32),

    #[serde(rename = "bouncemult")]
    BounceMult(f32),

    #[serde(rename = "scalemult")]
    ScaleMult(f32),

    #[serde(rename = "massmult")]
    MassMult(f32),

    #[serde(rename = "frictionmult")]
    FrictionMult(f32),

    #[serde(rename = "blastjumpmult")]
    BlastJumpMult(f32),

    #[serde(rename = "blastpushmult")]
    BlastPushMult(f32),

    #[serde(rename = "blastrangemult")]
    BlastRangeMult(f32),

    #[serde(rename = "blastcooldownmult")]
    BlastCooldownMult(f32),

    #[serde(rename = "rollX")]
    RollX(f32),

    #[serde(rename = "rollY")]
    RollY(f32),

    #[serde(rename = "canblast")]
    CanBlast(bool),

    #[serde(rename = "airjumps")]
    AirJumps(i32),

    #[serde(rename = "nopowerups")]
    NoPowerups(bool),

    #[serde(rename = "reverse")]
    Reverse(bool),

    #[serde(rename = "checkpointgems")]
    CheckpointGems(bool),

    #[serde(rename = "nogems")]
    NoGems(bool),

    #[serde(rename = "notimetravel")]
    NoTimeTravel(bool),

    #[serde(rename = "trophygem")]
    TrophyGem(bool),

    #[serde(rename = "trophyend")]
    TrophyEnd(bool),

    #[serde(rename = "boomerang")]
    Boomerang(bool),

    #[serde(rename = "startpowerup")]
    StartPowerup(String),

    #[serde(rename = "replacepowerup")]
    ReplacePowerup(String),

    #[serde(rename = "platformspeed")]
    PlatformSpeed(f32),

    #[serde(rename = "blastX")]
    BlastX(f32),

    #[serde(rename = "blastY")]
    BlastY(f32),

    #[serde(rename = "impX")]
    ImpactX(f32),

    #[serde(rename = "impY")]
    ImpactY(f32),

    #[serde(rename = "usesounds")]
    UseSounds(bool),

    #[serde(rename = "megaforce")]
    MegaForce(f32),

    #[serde(rename = "fullshadow")]
    FullShadow(bool),

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

pub enum NameLang {
    En,
    Es,
    Fr,
    De,
    It,
    Jp,
    Ar,
    ZhCh,
    ZhTw,
    Nl,
    Ko,
    Pt,
    Ru,
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
