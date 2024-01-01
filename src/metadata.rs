//! Loads metadata like level ids and titles

use std::{collections::HashMap, fs};

/// Loads and deserializes all level ids
pub fn load_name_vec() -> Vec<String> {
    let raw =
        fs::read_to_string("./miuu_level_ids.json").expect("Failed to load level name vec file");

    serde_json::from_str::<Vec<String>>(&raw).expect("Failed to parse level name vec file")
}

/// Loads and deserializes all level id > level title
pub fn load_name_conversion_map() -> HashMap<String, String> {
    let raw_map = fs::read_to_string("./miuu_levelid_to_name.json")
        .expect("Failed to load name conversion map file");

    serde_json::from_str::<HashMap<String, String>>(&raw_map)
        .expect("Failed to parse name conversion map file")
}
