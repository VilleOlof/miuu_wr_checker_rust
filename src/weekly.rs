//! Fetches and handles weekly challenges

use anyhow::{anyhow, Result};
use colored::Colorize;
use reqwest::Client;
use sqlx::SqliteConnection;

use crate::{
    config::SETTINGS,
    db,
    request::make_request,
    score::Score,
    weekly_data::{self, ScoreBucket},
};

/// Fetches the world record for a given week state and scorebucket
pub async fn fetch(client: &Client, state: &WeekState, bucket: &ScoreBucket) -> Result<Vec<Score>> {
    let bucket_state = match state {
        WeekState::Current => bucket.current.clone(),
        WeekState::Previous => bucket.previous.clone(),
    };
    let (start, end) = (bucket_state.start_date, bucket_state.end_date);
    let base_params: Vec<(&str, &str)> = vec![("order", "time"), ("limit", "1")];

    let mut scores: Vec<Score> = Vec::with_capacity(5);

    for (i, level) in bucket_state.levels.into_iter().enumerate() {
        let map_id = format!("{}{}", bucket_state.chapter_set, i);

        let where_value = format!(
            r#"
            {{
                "mapID":"{}", 
                "updatedAt":{{
                    "$gte":{{
                        "__type":"Date",
                        "iso":"{}"
                    }}, 
                    "$lt":{{
                        "__type":"Date",
                        "iso":"{}"
                    }}
                }}
            }}
            "#,
            map_id,
            start.format("%+").to_string(),
            end.format("%+").to_string()
        );

        let mut params = base_params.clone();
        params.push(("where", &where_value));

        let score = match make_request(
            client,
            params,
            None,
            Some(SETTINGS.read().unwrap().parse.weekly.class_name.clone()),
        )
        .await
        {
            Ok(resp) => resp,
            Err(err) => return Err(anyhow!("Failed to fetch weekly: {}", err)),
        };
        let mut score = score
            .first()
            .expect("No elements in scores, weekly")
            .clone();

        // Since weekly challenge map_ids are A/B#, we just quickly convert them back
        score.map_id = level.name;

        scores.push(score);
    }

    Ok(scores)
}

/// Checks if theres a new weekly challenge or not
///
/// Uses the saved end date in the database and compares to the server
pub async fn check(
    conn: &mut SqliteConnection,
    client: &Client,
) -> (bool, Option<weekly_data::Weekly>) {
    let newest_data = match weekly_data::Weekly::fetch(client).await {
        Ok(data) => data,
        Err(_) => {
            println!("{}", "Failed to fetch newest weekly data".red().bold());
            return (false, None);
        }
    };

    let db_date = match db::get_current_weekly_end(conn).await {
        Ok(date) => date,
        Err(_) => return (true, Some(newest_data)), // Probably means it doesnt exist yet
    };

    // If the current start date is different, we got a new one
    if newest_data.score_buckets.current.end_date != db_date {
        return (true, Some(newest_data));
    }

    (false, None)
}

/// The week state
#[allow(dead_code)]
pub enum WeekState {
    /// For the current weekly challenge
    Current,
    /// For the previous weekly challenge
    Previous,
}
