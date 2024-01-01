//! Handles replay downloading and saving for new world records

use std::fs;

use anyhow::{anyhow, Result};
use reqwest::Client;

use crate::{config::SETTINGS, request::raw_request, score::Score};

/// Downloads a replay and saves it to disk
///
/// Saves them to `./replay/levelid/filecount_username_time.replay`
pub async fn download_replay(client: &Client, score: &Score) -> Result<()> {
    let replay_data = match score.replay.to_owned() {
        Some(name) => name,
        None => {
            return Err(anyhow!(
                "No valid parse url, replay name is None. db score?"
            ))
        }
    };

    let url = match reqwest::Url::parse(&format!(
        "https://{}/parse/files/{}/{}",
        &SETTINGS.read().unwrap().parse.domain,
        &SETTINGS.read().unwrap().parse.appid,
        replay_data.name
    )) {
        Ok(url) => url,
        Err(err) => return Err(anyhow!("Failed to parse replay url: {}", err)),
    };

    let res = match raw_request(client, url).await {
        Ok(res) => res,
        Err(err) => return Err(anyhow!("Failed to download replay: {}", err)),
    };

    if let Ok(bytes) = res.bytes().await {
        match fs::create_dir_all(&get_path(&score)) {
            Err(err) => return Err(anyhow!("Failed to create dir for replay: {}", err)),
            _ => (),
        };

        match fs::write(get_path(&score) + &get_name(&score), bytes) {
            Err(err) => return Err(anyhow!("Failed to save replay onto disk: {}", err)),
            _ => (),
        };
    }

    Ok(())
}

fn get_path(score: &Score) -> String {
    format!("./replays/{}/", score.map_id)
}

fn get_name(score: &Score) -> String {
    let file_count: isize = match fs::read_dir(get_path(&score)) {
        Ok(files) => files.count() as isize,
        Err(err) => {
            println!(
                "Failed to get file count for replay: {} - {}",
                score.map_id, err
            );
            -1
        }
    };

    format!("{}_{}_{}.replay", file_count, score.username, score.time)
}
