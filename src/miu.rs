use crate::{request::make_request, score::Score};
use futures::future::try_join_all;
use reqwest::Client;

pub async fn get_wrs(client: &Client, levels: &Vec<String>) -> Result<Vec<Score>, String> {
    let mut level_futures = Vec::new();

    for level in levels {
        level_futures.push(fetch(&client, level));
    }

    try_join_all(level_futures).await
}

async fn fetch(client: &Client, level: &str) -> Result<Score, String> {
    let level_str = format!(r#"{{"mapID":"SP_{}"}}"#, level);
    let params = vec![
        ("limit", "1"),
        ("order", "time,-updatedAt"),
        ("where", &level_str),
    ];

    match make_request(client, params, None, None).await {
        Ok(mut score) => {
            if score.is_empty() {
                return Err(format!("Empty Scores returned from: {}", level));
            }

            Ok(score.remove(0))
        }
        Err(err) => Err(format!("[{}] Failed to fetch score, {}", level, err)),
    }
}
