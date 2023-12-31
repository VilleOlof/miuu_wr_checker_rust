// {"mapID":"B2", "updatedAt":{"$gte":{"__type":"Date","iso":"2023-12-27T17:00:00Z"}, "$lt":{"__type":"Date","iso":"2024-01-03T17:00:00Z"}}}

use reqwest::{header::CONTENT_TYPE, Client};

use crate::{
    config::SETTINGS,
    embed::get_weekly_embed,
    request::make_request,
    score::Score,
    webhook::WebhookRequest,
    weekly_data::{ScoreBucket, Weekly},
};

pub async fn fetch(
    client: &Client,
    state: &WeekState,
    bucket: &ScoreBucket,
) -> Result<Vec<Score>, String> {
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
            Err(err) => return Err(format!("Failed to fetch weekly: {}", err)),
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

pub async fn send_weekly_embed(client: &Client, weekly: &Weekly, previous_scores: &Vec<Score>) {
    let embed = get_weekly_embed(&weekly, &previous_scores);
    let request_struct = WebhookRequest {
        embeds: vec![embed],
    };

    for url in &SETTINGS.read().unwrap().discord.weekly_webhooks {
        match client
            .post(url.clone() + "?wait=true")
            .json(&request_struct)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
        {
            Ok(_) => (),
            Err(_) => println!("Failed to send challenge webhook"),
        };
    }
}

pub fn check() -> bool {
    todo!()
}

pub enum WeekState {
    Current,
    Previous,
}
