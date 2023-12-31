use std::{
    collections::HashMap,
    thread::sleep,
    time::{Duration, Instant},
};

use colored::*;
use reqwest::Client;
use sqlx::SqliteConnection;

use crate::{db::*, metadata::*, miu::get_wrs, replay::download_replay, score::Score, webhook::*};

mod config;
mod db;
mod embed;
mod metadata;
mod miu;
mod replay;
mod request;
mod score;
mod webhook;
mod weekly;
mod weekly_data;

const SLEEPDURATION: Duration = Duration::from_secs(120);

// !! convert all String related Results to custom error
// Send a DB backup once every 2 weeks?
// Send a weekly recap of every new WR? sql query to sort newly ones

// Weekly WR Recap: Total Improvement, each level should be a field with the name as title, value should be
// the score, username, time, when, and then on a new line, the improvement during the week?

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{} {}",
        "Starting MIU WRChecker".bold(),
        "(Rust Edition)".bright_black()
    );

    let level_ids = load_name_vec();
    let level_titles = load_name_conversion_map();

    let client = Client::new();
    let mut conn = setup().await;

    println!("- {}", "Init Sequence Finished".green().bold());

    let weekly_data = weekly_data::Weekly::fetch(&client).await.unwrap();
    let prev_scores = weekly::fetch(
        &client,
        &weekly::WeekState::Previous,
        &weekly_data.score_buckets,
    )
    .await
    .unwrap();
    weekly::send_weekly_embed(&client, &weekly_data, &prev_scores).await;

    return Ok(());

    let mut confirmed_wrs: HashMap<String, Score> = get_all(&mut conn, &level_ids).await;

    let mut iter_count: u32 = 0;
    loop {
        let start = Instant::now();

        let new_scores = match get_wrs(&client, &level_ids).await {
            Ok(scores) => scores,
            Err(err) => {
                println!("{}: {}", "Failed to get new WRs".red().bold(), err);
                continue;
            }
        };

        let mut new_wrs: Vec<(Score, Score)> = vec![];

        for score in new_scores {
            let confirmed = match confirmed_wrs.get(&score.map_id) {
                Some(score) => score,
                None => {
                    println!(
                        "{}: {}",
                        "Failed to get confirmed wr for".red().bold(),
                        score.map_id
                    );
                    continue;
                }
            };

            if score.time >= confirmed.time {
                continue;
            }

            // New World record
            new_wrs.push((score.clone(), confirmed.clone()));
            //Update confirmed_wrs
            match confirmed_wrs.get_mut(&score.map_id) {
                Some(c_score) => {
                    *c_score = score.to_owned();
                    println!(
                        "{}: {} ({}, {}, {})",
                        "New World Record For".green().bold(),
                        score.map_id,
                        score.time,
                        score.username,
                        score.platform
                    );

                    new_wr(&client, &mut conn, score).await;
                }
                None => {
                    println!("{}", "Failed to update confirmed wrs".red().bold());
                    continue;
                }
            };
        }

        send_webhooks(&client, new_wrs, &level_titles).await;

        println!(
            "{} {:0>3} - {}",
            format!("[{:0<5}s]", start.elapsed().as_millis() as f64 / 1000.0).bright_black(),
            iter_count,
            "Finished WR Checking Iteration".green().bold()
        );
        iter_count += 1;

        sleep(SLEEPDURATION);
    }
}

async fn new_wr(client: &Client, conn: &mut SqliteConnection, score: Score) {
    match update_level(conn, &score).await {
        Err(err) => println!("{}: {}", "Failed to update score into db".red().bold(), err),
        _ => (),
    };

    match download_replay(client, &score).await {
        Ok(_) => println!(
            "{}: [{}] {}, {}",
            "Downloaded Replay For".green(),
            score.map_id,
            score.username,
            score.time
        ),
        Err(err) => println!("Failed during replay handle: {}", err),
    };
}
