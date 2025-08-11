use std::{fs, path::Path};
use rand::seq::IndexedRandom;

use serde::Deserialize;
use serde_json;
use crate::ui::app::MyApp;

#[derive(Deserialize)]
pub struct Quote {
    pub id: usize,
    pub name: String,
    pub quote: String,
}

impl MyApp {
    pub fn get_quote(&self) -> String {
        
        let default = self.ui.default_subtitle.clone();
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        // join our relative asset path to it:
        let full_path = Path::new(manifest_dir).join("assets/qoutes/qoutes.json");
        let json_string = 
        match fs::read_to_string(full_path){
            Ok(json_string) =>json_string, 
            Err(e) => {println!("{:?}", e); return default;}
        };
        let qoute_vec:Vec<Quote> = match serde_json::from_str(&json_string){
            Ok(vector) => vector,
            Err(e) => {println!("{:?}", e); return default;}
        };

        return match qoute_vec.choose(&mut rand::rng()){
            Some(quote) => String::from(format!("- {} -", quote.quote)),
            None => default,
        };
    }
}