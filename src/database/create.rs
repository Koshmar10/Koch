
use rusqlite::{params, Connection, Result};

use crate::engine::board;
use crate::engine::board::BoardMetaData;
use crate::engine::board::GameResult;
use crate::engine::board::MoveStruct;
use crate::game::controller::TerminationBy;
use crate::game::evaluator::from_sql_params;

pub fn create_database() -> Result<()> {
    let con = Connection::open("chess.db")?;

    con.execute(
        "CREATE TABLE IF NOT EXISTS games (
            id INTEGER PRIMARY KEY,
            played DATE,
            starting_fen TEXT,
            black_player TEXT,
            white_player TEXT,
            black_elo INTEGER,
            white_elo INTEGER
        )",
        (),
    )?;

    con.execute(
        "CREATE TABLE IF NOT EXISTS moves (
            id INTEGER PRIMARY KEY,
            game_id INTEGER NOT NULL,
            uci TEXT NOT NULL,
            san TEXT,
            eval_score REAL NOT NULL,
            eval_type TEXT NOT NULL,
            FOREIGN KEY (game_id) REFERENCES games (id)
        )",
        (),
    )?;

    Ok(())
}
pub fn insert_game_and_get_id(game_data: &BoardMetaData) -> Result<i64> {
    let con = Connection::open("chess.db")?;

    con.execute(
        "INSERT INTO games (played, starting_fen, black_player, white_player, black_elo, white_elo)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            &game_data.date,
            &game_data.starting_position,
            &game_data.black_player_name,
            &game_data.white_player_name,
            &game_data.black_player_elo,
            &game_data.white_player_elo,
        ],
    )?;

    Ok(con.last_insert_rowid())
}

pub fn insert_single_move(game_id: i64, ply: &MoveStruct) -> Result<()> {
    let con = Connection::open("chess.db")?;

    let (eval_score, eval_type) = ply.evaluation.to_sql_params();

    con.execute(
        "INSERT INTO moves (game_id, uci, san, eval_score, eval_type) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![game_id, &ply.uci, &ply.san, eval_score, eval_type],
    )?;

    Ok(())
}
pub fn destroy_database(){
    let con = Connection::open("chess.db").unwrap();
    con.execute("DROP TABLE IF EXISTS moves", ()).unwrap();
    con.execute("DROP TABLE IF EXISTS games", ()).unwrap();
}

pub fn get_game_list() -> Result<Vec<BoardMetaData>>{

    let con = Connection::open("chess.db")?;

    let mut query_result = con.prepare("
    SELECT * FROM games
    ")?;
    let games_iter = query_result.query_map([], |row|{
        let game_id: i64 = row.get(0)?;
        let mut game_move_query = con.prepare(&format!("SELECT * FROM moves WHERE game_id = {}", game_id))?;
        let game_move_iter = game_move_query.query_map([], |row|{
            Ok(
                MoveStruct{
                    move_number: 0,
                    san: row.get(3)?,
                    uci: row.get(2)?,
                    evaluation: from_sql_params(row.get(4)?, row.get::<_, String>(5)?.as_str()),
                    ..Default::default()         
                }
            )
        })?;
        let mut move_list = Vec::new();
        for mv in game_move_iter {
            move_list.push(mv?);
        }
        Ok(BoardMetaData {
            starting_position : row.get(2)?,
            date: row.get(1)?,
            move_list: move_list,
            termination: TerminationBy::StaleMate,
            result: GameResult::Unfinished,
            black_player_name: row.get(3)?,
            white_player_name:row.get(4)?,
            black_player_elo:row.get(5)?,
            white_player_elo:row.get(6)?,
            ..Default::default()
        })

    })?;
    let mut games = Vec::new();
    for game in games_iter {
        games.push(game?);
    }
    Ok(games)
}

pub fn get_last_n_game(_n: u32){

}