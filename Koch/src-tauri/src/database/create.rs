use crate::engine::board::{
    self, BoardMetaData, EvalResponse, EvalType, GameResult, MoveStruct, TerminationBy,
};
use crate::engine::PieceType;
use crate::game;
use rusqlite::{params, Connection, Result};

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
            white_elo INTEGER,
            result TEXT,
            termination TEXT,
            opening TEXT
        )",
        (),
    )?;

    con.execute(
        "CREATE TABLE IF NOT EXISTS moves (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id INTEGER NOT NULL,
            move_number INTEGER NOT NULL,
            uci TEXT NOT NULL,
            san TEXT NOT NULL,
            eval_value REAL,
            eval_kind TEXT,
            promotion TEXT,
            is_capture INTEGER NOT NULL,
            time_stamp REAL,
            FOREIGN KEY (game_id) REFERENCES games (id)
        )",
        (),
    )?;

    Ok(())
}
pub fn insert_game_and_get_id(game_data: &BoardMetaData) -> Result<i64> {
    let con = Connection::open("chess.db")?;
    let result: String = match &game_data.result {
        GameResult::BlackWin => String::from("0-1"),
        GameResult::WhiteWin => String::from("1-0"),
        GameResult::Draw => String::from("1/2-1/2"),
        GameResult::Unfinished => String::from("---"),
    };
    let termination: String = match &game_data.termination {
        board::TerminationBy::Checkmate => String::from("Checkmate"),
        board::TerminationBy::StaleMate => String::from("Stalemate"),
        board::TerminationBy::Draw => String::from("Draw"),
        board::TerminationBy::Timeout => String::from("Timeout"),
    };
    con.execute(
        "INSERT INTO games (played, starting_fen, black_player, white_player, black_elo, white_elo, result, termination, opening)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            &game_data.date,
            &game_data.starting_position,
            &game_data.black_player_name,
            &game_data.white_player_name,
            &game_data.black_player_elo,
            &game_data.white_player_elo,
            &result,
            &termination,
            &game_data.opening,
        ],
    )?;

    Ok(con.last_insert_rowid())
}

pub fn insert_single_move(game_id: i64, mv: MoveStruct) -> Result<()> {
    let promotion_str = mv.promotion.map(|p| format!("{:?}", p));
    let eval_kind_str = match mv.evaluation.kind {
        EvalType::Centipawn => "cp",
        EvalType::Mate => "mate",
    };
    let is_capture_int = if mv.is_capture { 1 } else { 0 };
    let con = Connection::open("chess.db")?;
    con.execute(
        "INSERT INTO moves (game_id, move_number, uci, san, eval_value, eval_kind, promotion, is_capture, time_stamp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            game_id,
            mv.move_number,
            mv.uci,
            mv.san,
            mv.evaluation.value,
            eval_kind_str,
            promotion_str,
            is_capture_int,
            mv.time_stamp
        ],
    )?;
    Ok(())
}

pub fn destroy_database() {
    let con = Connection::open("chess.db").unwrap();
    con.execute("DROP TABLE IF EXISTS moves", ()).unwrap();
    con.execute("DROP TABLE IF EXISTS games", ()).unwrap();
}

// Helper: parse result string ("1-0","0-1","1/2-1/2","---") into GameResult
fn parse_game_result(s: &str) -> GameResult {
    match s {
        "1-0" => GameResult::WhiteWin,
        "0-1" => GameResult::BlackWin,
        "1/2-1/2" => GameResult::Draw,
        _ => GameResult::Unfinished,
    }
}

// Helper: parse termination string into TerminationBy
fn parse_termination(s: &str) -> TerminationBy {
    match s {
        "Checkmate" => TerminationBy::Checkmate,
        "Stalemate" | "StaleMate" => TerminationBy::StaleMate,
        "Draw" => TerminationBy::Draw,
        "Timeout" => TerminationBy::Timeout,
        _ => TerminationBy::Draw, // fallback
    }
}

pub fn get_game_list() -> Result<Vec<BoardMetaData>> {
    let con = Connection::open("chess.db")?;

    let mut query_result = con.prepare("SELECT * FROM games")?;
    let games_iter = query_result.query_map([], |row| {
        let game_id: i64 = row.get(0)?;
        let mut game_move_query = con.prepare(
            "SELECT move_number, uci, san, eval_value, eval_kind, promotion, is_capture, time_stamp
             FROM moves WHERE game_id = ? ORDER BY move_number ASC",
        )?;
        let game_move_iter = game_move_query.query_map(params![game_id], |mrow| {
            let eval_value: f32 = mrow.get::<_, f64>(3)? as f32;
            let eval_kind_str: String = mrow.get(4)?;
            let eval_kind = match eval_kind_str.as_str() {
                "mate" => EvalType::Mate,
                _ => EvalType::Centipawn,
            };
            let promo_opt: Option<String> = mrow.get(5)?;
            let promotion = promo_opt.as_ref().and_then(|s| match s.as_str() {
                "Queen" => Some(PieceType::Queen),
                "Rook" => Some(PieceType::Rook),
                "Bishop" => Some(PieceType::Bishop),
                "Knight" => Some(PieceType::Knight),
                _ => None,
            });

            Ok(MoveStruct {
                move_number: mrow.get(0)?,
                uci: mrow.get(1)?,
                san: mrow.get(2)?,
                evaluation: EvalResponse {
                    value: eval_value,
                    kind: eval_kind,
                },
                promotion,
                is_capture: mrow.get::<_, i64>(6)? == 1,
                time_stamp: mrow.get::<_, f64>(7)? as f32,
                ..Default::default()
            })
        })?;

        let mut move_list = Vec::new();
        for mv in game_move_iter {
            move_list.push(mv?);
        }

        let termination_str: String = row.get(8)?;
        let result_str: String = row.get(7)?;

        Ok(BoardMetaData {
            starting_position: row.get(2)?,
            date: row.get(1)?,
            move_list,
            termination: parse_termination(&termination_str),
            result: parse_game_result(&result_str),
            black_player_name: row.get(3)?,
            white_player_name: row.get(4)?,
            black_player_elo: row.get(5)?,
            white_player_elo: row.get(6)?,
            opening: row.get(9)?,
            ..Default::default()
        })
    })?;

    let mut games = Vec::new();
    for game in games_iter {
        games.push(game?);
    }
    Ok(games)
}
pub fn get_game_by_id(id: usize) -> Result<BoardMetaData> {
    let con = Connection::open("chess.db")?;
    let mut query_result =
        con.prepare(format!("SELECT * FROM games g WHERE g.id = {}", &id).as_str())?;
    let games_iter = query_result.query_map([], |row| {
        let game_id: i64 = id as i64;
        let mut game_move_query = con.prepare(
            "SELECT move_number, uci, san, eval_value, eval_kind, promotion, is_capture, time_stamp
             FROM moves WHERE game_id = ? ORDER BY move_number ASC",
        )?;
        let game_move_iter = game_move_query.query_map(params![game_id], |mrow| {
            let eval_value: f32 = mrow.get::<_, f64>(3)? as f32;
            let eval_kind_str: String = mrow.get(4)?;
            let eval_kind = match eval_kind_str.as_str() {
                "mate" => EvalType::Mate,
                _ => EvalType::Centipawn,
            };
            let promo_opt: Option<String> = mrow.get(5)?;
            let promotion = promo_opt.as_ref().and_then(|s| match s.as_str() {
                "Queen" => Some(PieceType::Queen),
                "Rook" => Some(PieceType::Rook),
                "Bishop" => Some(PieceType::Bishop),
                "Knight" => Some(PieceType::Knight),
                _ => None,
            });

            Ok(MoveStruct {
                move_number: mrow.get(0)?,
                uci: mrow.get(1)?,
                san: mrow.get(2)?,
                evaluation: EvalResponse {
                    value: eval_value,
                    kind: eval_kind,
                },
                promotion,
                is_capture: mrow.get::<_, i64>(6)? == 1,
                time_stamp: mrow.get::<_, f64>(7)? as f32,
                ..Default::default()
            })
        })?;

        let mut move_list = Vec::new();
        for mv in game_move_iter {
            move_list.push(mv?);
        }

        let termination_str: String = row.get(8)?;
        let result_str: String = row.get(7)?;

        Ok(BoardMetaData {
            starting_position: row.get(2)?,
            date: row.get(1)?,
            move_list,
            termination: parse_termination(&termination_str),
            result: parse_game_result(&result_str),
            black_player_name: row.get(3)?,
            white_player_name: row.get(4)?,
            black_player_elo: row.get(5)?,
            white_player_elo: row.get(6)?,
            opening: row.get(9)?,
            ..Default::default()
        })
    })?;

    let mut games = Vec::new();
    for game in games_iter {
        games.push(game?);
    }
    let first_game = games.get(0).unwrap();
    return Ok(first_game.clone());
}
pub fn get_last_n_game(_n: u32) {}
