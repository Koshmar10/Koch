use std::{fs, sync::Mutex};

use chrono::{Datelike, Local};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{database::create::load_pgn_game, server::server::ServerState};

#[derive(Debug, Serialize, Deserialize)]
struct ChessGame {
    url: String,
    pgn: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct ChessDotComGames {
    games: Vec<ChessGame>,
}
async fn get_chessdotcom_games(
    chessdotcom_user: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36";
    let now = Local::now();
    let current_year = now.year();
    let current_month = now.month();
    let url = format!(
        "https://api.chess.com/pub/player/{}/games/{current_year}/09",
        chessdotcom_user
    );
    let client = Client::builder()
        .user_agent(user_agent) // This is the magic line
        .build()?;
    println!("{}", &url);
    //let response = client.get(url).send().await?;
    //let text = response.text().await?;
    //println!("Successfully fetched response: {}", &text);
    let text =
        fs::read_to_string("/home/petru/storage/Projects/chess_app/Koch/src-tauri/src/games.json")?;
    Ok(text)
}
fn load_chessdotcom_games(
    chessdotcom_api_response: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let games_data: ChessDotComGames = serde_json::from_str(&chessdotcom_api_response)?;
    for game in &games_data.games {
        load_pgn_game(game.pgn.clone())?;
    }
    Ok(())
}
#[tauri::command]
pub async fn sync_with_chessdotcom(
    state: tauri::State<'_, Mutex<ServerState<'_>>>,
) -> Result<(), String> {
    // Lock only to extract the username, then drop the lock before await
    let chessdotcom_user = {
        let state = state.lock().map_err(|e| e.to_string())?;
        match state.settings.map.get("chessdotcom_user") {
            Some(cu) => cu.clone(),
            None => return Err("nu chieie in setari".to_string()),
        }
    };
    let api_response = get_chessdotcom_games(&chessdotcom_user)
        .await
        .map_err(|e| e.to_string())?;
    load_chessdotcom_games(&api_response).map_err(|e| e.to_string())?;
    Ok(())
}
