//! Handles the init connection and queries to the sqlite database

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use sqlx::{prelude::FromRow, Connection, SqliteConnection};
use std::collections::HashMap;

use crate::{
    config::SETTINGS,
    miu::score::{RecapScore, Score},
};

/// Create all tables in the database
///
/// Ran upon init, but only is here for first time setup really.
pub async fn create_tables(conn: &mut SqliteConnection, levels: &Vec<String>) {
    match sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )
    "#,
    )
    .execute(&mut *conn)
    .await
    {
        Err(err) => panic!("Failed to create metadata table: {}", err),
        _ => (),
    }

    match sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS weekly_history (
            start_date TEXT PRIMARY KEY,
            end_date TEXT NOT NULL,
            scores TEXT NOT NULL,
            physics_mods TEXT NOT NULL,
            name TEXT NOT NULL,
            challenge_id TEXT NOT NULL
        )
    "#,
    )
    .execute(&mut *conn)
    .await
    {
        Err(err) => panic!("Failed to create weekly_history table: {}", err),
        _ => (),
    }

    for level in levels {
        let query = format!(
            r#"
            CREATE TABLE IF NOT EXISTS SP_{} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                time INTEGER NOT NULL,
                username TEXT NOT NULL,
                userID TEXT NOT NULL,
                skinUsed TEXT NOT NULL,
                replayVersion INTEGER NOT NULL,
                platform TEXT NOT NULL,
                createdAt TEXT NOT NULL,
                updatedAt TEXT NOT NULL
            )
        "#,
            level
        );

        match sqlx::query(&query).execute(&mut *conn).await {
            Err(err) => panic!(
                "Failed to create new table for level: {}, due to: {}",
                level, err
            ),
            _ => (),
        };
    }
}

/// Gets the current saved weekly challenge end date
pub async fn get_current_weekly_end(conn: &mut SqliteConnection) -> Result<DateTime<Utc>> {
    let date_text: DBWeekEnd =
        sqlx::query_as("SELECT value FROM metadata WHERE key = \"curr_week_end\"")
            .fetch_one(&mut *conn)
            .await?;

    Ok(date_text.value)
}

/// Upserts a weekly challenge end date into the metadata table
pub async fn upsert_weekly_end(conn: &mut SqliteConnection, date: DateTime<Utc>) {
    match sqlx::query(
        r#"
            INSERT INTO metadata 
            (key, value) VALUES 
            ("curr_week_end", ?) 
            ON CONFLICT DO 
            UPDATE SET value = ? 
            WHERE key = "curr_week_end"
        "#,
    )
    .bind(date)
    .bind(date)
    .execute(&mut *conn)
    .await
    {
        Err(err) => panic!("Failed to upsert weekly end: {}", err),
        _ => (),
    }
}

/// Gets all world records given a `Vec<String>` of level ids.
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

/// Inserts a new world record into the levels history, given the score.
pub async fn update_level(conn: &mut SqliteConnection, score: &Score) -> Result<()> {
    //mhmhm i love those .bind, probably a way to bind a struct to values or somethning
    match sqlx::query(&format!(
        r#"
    INSERT INTO {} (
        time, 
        username, 
        userID, 
        skinUsed, 
        replayVersion, 
        platform, 
        createdAt, 
        updatedAt
    ) VALUES (
            ?, 
            ?, 
            ?, 
            ?, 
            ?, 
            ?, 
            ?, 
            ?
        )"#,
        score.map_id
    ))
    .bind(score.time.clone())
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

/// Gets all world records within a `chrono::Duration`.
///
/// And only for the levels specified,
/// all levels given also needs a key in the hashmap to convert to level title.
pub async fn get_latest_world_records(
    conn: &mut SqliteConnection,
    duration: Duration,
    levels: &Vec<String>,
    level_titles: &HashMap<String, String>,
) -> Option<Vec<RecapScore>> {
    let break_point_date = Utc::now() - duration;

    let mut scores: Vec<RecapScore> = vec![];

    for level in levels {
        // Query could be optimized if we also stored unix times along side and only used a where clause,
        // But since the times are stored as ISO text strings im unsure if we can do a where, so just doing them all for now
        let query_str = format!("SELECT * FROM SP_{} ORDER BY time ASC", level);
        let db_scores: Vec<DBScore> = sqlx::query_as(&query_str)
            .fetch_all(&mut *conn)
            .await
            .expect("Failed to fetch latest wr, empty?");

        // Edge case for only one, aka new wr. should only happen to new empty databases
        if db_scores.len() == 1 {
            let only = db_scores.first().unwrap();
            if only.updated_at > break_point_date {
                scores.push(RecapScore {
                    level: level_titles.get(level).expect("how?!?").clone(),
                    improvement: 0.0,
                    scores: vec![only.to_score(level.to_owned())],
                });
            }

            continue;
        }

        let mut level_scores: Vec<Score> = vec![];

        for score in db_scores {
            if score.updated_at < break_point_date {
                let improv = match level_scores.first() {
                    Some(newest) => score.time - newest.time,
                    None => break,
                };

                let recap_score = RecapScore {
                    level: level_titles.get(level).expect("how?!?").clone(),
                    improvement: improv,
                    scores: level_scores,
                };

                scores.push(recap_score);

                break; // Old times
            }

            level_scores.push(score.to_score(level.to_owned()));
        }
    }

    if scores.is_empty() {
        return None;
    }

    Some(scores)
}

/// Establishes a connection to the database
///
/// Depending on the database_url set in the settings
pub async fn setup() -> SqliteConnection {
    let url = &SETTINGS.read().unwrap().database_url;

    match SqliteConnection::connect(url).await {
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

#[derive(Debug, FromRow)]
struct DBWeekEnd {
    pub value: DateTime<Utc>,
}
