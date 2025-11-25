use rand::seq::IndexedRandom;
use std::{fs, path::Path};

use serde::Deserialize;
use serde_json;

use crate::server::server::ServerState;

#[derive(Deserialize)]
pub struct Quote {
    pub id: usize,
    pub name: String,
    pub quote: String,
}

impl ServerState {
    pub fn get_quote(&self) -> String {
        let default = String::from("black");
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        // join our relative asset path to it:
        let full_path =
            Path::new("/home/petru/storage/Projects/chess_app/Koch/src/assets/qoutes/qoutes.json");
        let json_string = match fs::read_to_string(full_path) {
            Ok(json_string) => json_string,
            Err(e) => {
                println!("{:?}", e);
                return default;
            }
        };
        let qoute_vec: Vec<Quote> = match serde_json::from_str(&json_string) {
            Ok(vector) => vector,
            Err(e) => {
                println!("{:?}", e);
                return default;
            }
        };

        return match qoute_vec.choose(&mut rand::rng()) {
            Some(quote) => String::from(format!("- {} -", quote.quote)),
            None => default,
        };
    }
}
