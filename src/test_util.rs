//! Used for utility functions for testing

use chrono::Utc;
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use std::ops::Range;

use crate::miu::score::{Replay, Score};

/// Generates a fake score
pub fn get_fake_score(time_range: Range<f32>) -> Score {
    fn get_random_elem(mut rng: &mut ThreadRng, vec: &Vec<String>) -> String {
        vec.choose(&mut rng).unwrap().to_owned()
    }

    let mut rng = rand::thread_rng();

    let user_ids: Vec<String> = vec!["UserId1".into(), "UserId2".into(), "UserId3".into()];
    let usernames: Vec<String> = vec!["Username1".into(), "Username2".into(), "Username3".into()];

    Score {
        time: rand::thread_rng().gen_range(time_range),
        user_id: get_random_elem(&mut rng, &user_ids),
        username: get_random_elem(&mut rng, &usernames),
        map_id: "test_level".into(),
        skin_used: "swirl".into(),
        replay_version: 5,
        platform: "PC".into(),
        replay: Some(Replay {
            r#type: "File".into(),
            name: "REPLAY_USERID_USERNAME.replay".into(),
            url: "https://localhost:0/name.replay".into(),
        }),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        object_id: None,
    }
}
