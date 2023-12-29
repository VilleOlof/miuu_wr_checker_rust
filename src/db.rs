use chrono::{DateTime, Utc};
use dotenvy::var;
use sqlx::{prelude::FromRow, Connection, SqliteConnection};
use std::collections::HashMap;

use crate::score::Score;

pub async fn get_all(conn: &mut SqliteConnection, levels: &Vec<String>) -> HashMap<String, Score> {
    // maybe optimize and execute_many at some point?
    let mut scores: HashMap<String, Score> = HashMap::new();

    for level in levels {
        let query_str = format!("SELECT * FROM SP_{} ORDER BY time ASC LIMIT 1", level);
        let db_score: DBScore = sqlx::query_as(&query_str)
            .fetch_one(&mut *conn)
            .await
            .expect("Failed to fetch latest wr, empty?");

        scores.insert(
            String::from("SP_") + &level,
            db_score.to_score(level.to_owned()),
        );
    }

    scores
}

pub async fn update_level(conn: &mut SqliteConnection, score: &Score) -> Result<(), String> {
    //mhmhm i love those .bind, probably a way to bind a struct to values or somethning
    match sqlx::query("INSERT INTO ? (time, username, userID, skinUsed, replayVersion, platform, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&score.map_id)
        .bind(score.time)
        .bind(score.username.clone())
        .bind(score.user_id.clone())
        .bind(score.skin_used.clone())
        .bind(score.replay_version)
        .bind(score.platform.clone())
        .bind(score.created_at)
        .bind(score.updated_at)
        .execute(conn)
        .await
    {
        Ok(_) => Ok(()),
        Err(err) => panic!("Failed to insert new score: {}", err),
    }
}

pub async fn setup() -> SqliteConnection {
    let url = match var("DATABASE_URL") {
        Ok(url) => url,
        Err(err) => panic!("Failed to get database Url: {}", err),
    };

    match SqliteConnection::connect(&url).await {
        Ok(conn) => conn,
        Err(err) => panic!("Failed to connect to database: {}", err),
    }
}

#[derive(Debug, FromRow)]
struct DBScore {
    #[sqlx(rename = "id")]
    _id: i32,

    time: f32,
    username: String,

    #[sqlx(rename = "userID")]
    user_id: String,

    #[sqlx(rename = "skinUsed")]
    skin_used: String,

    #[sqlx(rename = "replayVersion")]
    replay_version: u32,

    platform: String,

    #[sqlx(rename = "createdAt")]
    created_at: DateTime<Utc>,

    #[sqlx(rename = "updatedAt")]
    updated_at: DateTime<Utc>,
}

impl DBScore {
    fn to_score(&self, level: String) -> Score {
        Score {
            time: self.time,
            username: self.username.clone(),
            user_id: self.user_id.clone(),
            skin_used: self.skin_used.clone(),
            map_id: level,
            replay_version: self.replay_version,
            platform: self.platform.clone(),
            replay: None,
            created_at: self.created_at,
            updated_at: self.updated_at,
            object_id: None,
        }
    }
}
