use crate::analyzer::analyzer::{LocalChat, LocalMessage, LocalMessageRole};
use crate::engine::board::{
    self, BoardMetaData, EvalResponse, EvalType, GameResult, MoveStruct, TerminationBy,
};
use crate::engine::{Board, PieceType};
use crate::game;
use crate::game::controller::TerminationReason;
use regex::Regex;
use rusqlite::{params, Connection, Result};

#[derive(Debug, Clone)]
pub struct PgnGame(String);
/*
impl PgnGame {
    pub fn new(s: String) -> Result<Self, String> {

}
}
*/
pub fn create_database() -> Result<()> {
    let con = Connection::open("chess.db")?;

    con.execute(
        "CREATE TABLE IF NOT EXISTS games (
            game_id INTEGER PRIMARY KEY AUTOINCREMENT,
            date_played TEXT,            
            white_player TEXT NOT NULL,
            black_player TEXT NOT NULL,
            white_elo NUMBER NOT NULL,
            black_elo NUMBER NOT NULL,
            result TEXT NOT NULL,
            opening TEXT NOT NULL,
            time_control TEXT,
            pgn_data TEXT NOT NULL
        );",
        (),
    )?;
    con.execute(
        "

        CREATE TABLE IF NOT EXISTS chats (
            chat_id INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id INTEGER,
            FOREIGN KEY(game_id) REFERENCES games(game_id)
            UNIQUE (game_id)
        );",
        (),
    )?;
    con.execute(
        "
        CREATE TABLE IF NOT EXISTS messages (
                message_id INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_id INTEGER,
                role TEXT,
                content TEXT,
                sent_at TEXT,
                move_index NUMBER,
                FOREIGN KEY(chat_id) REFERENCES chats(chat_id),
                UNIQUE(content) 
                );",
        (),
    )?;

    Ok(())
}

pub fn destroy_database() {
    let con = Connection::open("chess.db").unwrap();

    con.execute("DROP TABLE IF EXISTS games", ()).unwrap();
}

// Helper: parse result string ("1-0","0-1","1/2-1/2","---") into GameResult
pub fn parse_game_result(s: &str) -> GameResult {
    match s {
        "1-0" => GameResult::WhiteWin,
        "0-1" => GameResult::BlackWin,
        "1/2-1/2" => GameResult::Draw,
        _ => GameResult::Unfinished,
    }
}

// Helper: parse termination string into TerminationBy
fn parse_termination(s: &str) -> TerminationReason {
    let s_l = s.to_lowercase();
    if s_l.contains("checkmate") || s_l.contains("mate") {
        TerminationReason::Checkmate
    } else if s_l.contains("stalemate") || s_l.contains("stale") {
        TerminationReason::StaleMate
    } else if s_l.contains("resign") || s_l.contains("resignation") {
        // prefer a Resignation variant if present, otherwise fall back to Unknown
        #[allow(non_snake_case)]
        {
            // try to use Resignation if exists
            TerminationReason::Resignation
        }
    } else if s_l.contains("timeout") {
        TerminationReason::Timeout
    } else if s_l.contains("draw") {
        TerminationReason::Draw
    } else {
        TerminationReason::Resignation
    }
}

fn set_tag(metadata: &mut BoardMetaData, tag: String, value: String) {
    let value = value.trim_matches('"').to_string();
    match tag.as_str() {
        "Event" => {
            metadata.event = Some(value.clone());
            println!("Set Event: {}", value);
        }
        "Site" => {
            metadata.site = Some(value.clone());
            println!("Set Site: {}", value);
        }
        "Date" => {
            metadata.date = value.clone();
            println!("Set Date: {}", value);
        }
        "Round" => {
            metadata.round = Some(value.clone());
            println!("Set Round: {}", value);
        }
        "White" => {
            metadata.white_player_name = value.clone();
            println!("Set White: {}", value);
        }
        "Black" => {
            metadata.black_player_name = value.clone();
            println!("Set Black: {}", value);
        }
        "Result" => {
            metadata.result = GameResult::from(value.as_str());
            println!("Set Result: {}", value);
        }
        "WhiteElo" => {
            metadata.white_player_elo = value.parse::<u32>().unwrap_or(1);
            println!("Set WhiteElo: {}", value);
        }
        "BlackElo" => {
            metadata.black_player_elo = value.parse::<u32>().unwrap_or(1);
            println!("Set BlackElo: {}", value);
        }
        "TimeControl" => {
            metadata.time_control = Some(value.clone());
            println!("Set TimeControl: {}", value);
        }
        "Termination" => {
            metadata.termination = parse_termination(&value);
            println!("Set Termination: {}", value);
        }
        "ECO" => {
            metadata.eco = Some(value.clone());
            println!("Set ECO: {}", value);
        }
        "Opening" => {
            metadata.opening = Some(value.clone());
            println!("Set Opening: {}", value);
        }
        "EndTime" => {
            metadata.end_time = Some(value.clone());
            println!("Set EndTime: {}", value);
        }
        "Link" => {
            metadata.link = Some(value.clone());
            println!("Set Link: {}", value);
        }
        _ => {}
    }
}
pub fn normalize_pgn(pgn_data: String) -> String {
    let mut elipsis_re = Regex::new(r"(\d+\.\.\.)").unwrap();
    let normalized_pgn = elipsis_re.replace_all(&pgn_data, "").to_string();
    normalized_pgn
}
fn parse_pgn_string(s: String) -> BoardMetaData {
    let mut clocks: Vec<String> = Vec::new();
    let mut timestamps: Vec<u32> = Vec::new();
    let s = normalize_pgn(s);
    let mut move_string = String::new();
    let mut metadata = BoardMetaData::default();
    let mut translation_board = Board::from(&metadata.starting_position);
    let pgn_lines: Vec<String> = s.lines().map(|line| line.to_string()).collect();

    for (i, line) in pgn_lines.iter().enumerate() {
        let line = line.trim();

        if line.starts_with('[') && line.ends_with(']') {
            let trimed_line = line.trim_matches(|c| c == '[' || c == ']');

            let trimed_line_items: Vec<String> =
                trimed_line.splitn(2, ' ').map(|s| s.to_string()).collect();

            if trimed_line_items.len() >= 2 {
                let tag = trimed_line_items[0].clone();
                let mut value = trimed_line_items[1].clone();
                // Remove quotes from value if present
                value = value.trim_matches('"').to_string();

                set_tag(&mut metadata, tag, value);
            } else {
                println!(
                    "  Warning: Could not split tag line properly: '{}'",
                    trimed_line
                );
            }
        } else {
            if line.is_empty() {
                continue;
            }
            move_string.push(' ');
            move_string.push_str(line.trim());
        }
    }
    let clk_re = Regex::new(r"\[%clk\s+([\d:.]+)\]").unwrap();
    let ts_re = Regex::new(r"\[%timestamp\s+([\d:.]+)\]").unwrap();

    for cap in clk_re.captures_iter(&move_string) {
        clocks.push(cap[1].trim().to_string());
    }
    for cap in ts_re.captures_iter(&move_string) {
        timestamps.push(cap[1].trim().parse::<u32>().unwrap_or(0));
    }

    // remove clock tokens from move_string so they don't interfere with move parsing
    move_string = clk_re.replace_all(&move_string, "").to_string();
    move_string = ts_re.replace_all(&move_string, "").to_string();
    let re = Regex::new(r"(\d:\d{1,2})?(\d{1,3}\.)").unwrap();
    // Split by move numbers, trim and remove any empty/whitespace-only segments
    let move_pairs: Vec<String> = re
        .split(&move_string)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    println!("{:?}", &move_pairs);
    let mut ct_c = 0;
    let mut moves: Vec<MoveStruct> = Vec::new();
    let mut move_counter = 1;
    for pair in move_pairs {
        // pair is guaranteed non-empty here; use as &str for the rest of the logic
        let pair = pair.as_str();
        // Extract comments
        let mut comments: Vec<String> = Vec::new();
        let mut nags: Vec<i32> = Vec::new();
        let re_comment = Regex::new(r"\{([^{}]*)\}").unwrap();
        for cap in re_comment.captures_iter(pair) {
            comments.push(cap[1].trim().to_string());
        }

        // Extract NAGs
        let re_nag = Regex::new(r"\$(\d+)").unwrap();
        for cap in re_nag.captures_iter(pair) {
            if let Ok(nag_num) = cap[1].parse::<i32>() {
                nags.push(nag_num);
            }
        }

        // Remove comments and NAGs for move string normalization
        let pair = re_comment.replace_all(pair, "").trim().to_string();
        let pair = re_nag.replace_all(&pair, " ").to_string();
        let pair = pair.split_whitespace().collect::<Vec<_>>().join(" ");
        let pair_moves: Vec<&str> = pair.split(' ').collect();
        //println!("{:?}", pair);
        //println!("Comments: {:?}", comments);
        //println!("NAGs: {:?}", nags);
        // For each move in the pair (usually 1 or 2 moves per pair)

        for (i, mv) in pair_moves.iter().enumerate() {
            if mv.is_empty() {
                println!("  Skipping empty move string at move pair index {}", i);
                continue;
            }
            println!("  Parsing move {}: '{}'", i, mv);
            let mut uci = translation_board.san_to_uci(mv).unwrap_or_else(|e| {
                println!("    Failed to convert SAN '{}' to UCI: {:?}", mv, e);
                "".to_string()
            });
            if !&uci.is_empty() {
                println!("    SAN '{}' translated to UCI '{}'", mv, uci);
                let sqs = translation_board.decode_uci_move(&uci);
                match sqs {
                    Some((from, to, promotion)) => {
                        println!(
                            "    Decoded UCI '{}' to squares: from {:?}, to {:?}",
                            uci, from, to
                        );
                        match translation_board.move_piece(from, to, promotion) {
                            Ok(_) => {
                                println!(
                                    "    Successfully moved piece from {:?} to {:?}",
                                    from, to
                                );
                            }
                            Err(e) => {
                                println!(
                                    "    Move translation went wrong or invalid move: {:?}",
                                    e
                                );
                            }
                        }
                    }
                    None => {
                        println!(
                            "    Failed to decode UCI move '{}', setting UCI to empty",
                            uci
                        );
                        uci = String::new();
                    }
                }
            } else {
                println!("    UCI string is empty for SAN '{}'", mv);
            }
            let comment = comments.get(i).cloned();
            let nag = nags.get(i).cloned();
            let mut promotion = None;
            if let Some(eq_idx) = mv.find('=') {
                let promo_char = mv.chars().nth(eq_idx + 1);
                promotion = match promo_char {
                    Some('Q') => Some(PieceType::Queen),
                    Some('R') => Some(PieceType::Rook),
                    Some('B') => Some(PieceType::Bishop),
                    Some('N') => Some(PieceType::Knight),
                    _ => None,
                };
            }
            let move_struct = MoveStruct {
                move_number: move_counter,
                san: mv.to_string(),
                uci: uci,
                annotation: comment,
                nag: nag,
                promotion,
                is_capture: mv.to_string().contains('x'),
                time_stamp: timestamps.get(ct_c).copied(),
                clock: clocks.get(ct_c).cloned(),
            };
            ct_c += 1;
            //println!("{:?}", &move_struct);
            move_counter += 1;
            moves.push(move_struct);
            // If you want to collect them, push to moves vector
            // moves.push(move_struct);
        }
    }
    moves.pop();
    metadata.move_list = moves;

    //println!("Final metadata: {:?}", &metadata);
    metadata
}

use std::fmt::Write;

fn termination_to_string(t: &TerminationReason) -> Option<&'static str> {
    match t {
        TerminationReason::Checkmate => Some("checkmate"),
        TerminationReason::StaleMate => Some("stalemate"),
        TerminationReason::Resignation => Some("resignation"),
        TerminationReason::Timeout => Some("timeout"),
        TerminationReason::Draw => Some("draw"),
    }
}

pub fn metadata_to_pgn(metadata: &BoardMetaData) -> String {
    let mut pgn = String::new();

    // Core tags
    writeln!(
        pgn,
        "[Event \"{}\"]",
        metadata.event.clone().unwrap_or_else(|| "?".to_string())
    )
    .unwrap();
    writeln!(
        pgn,
        "[Site \"{}\"]",
        metadata.site.clone().unwrap_or_else(|| "?".to_string())
    )
    .unwrap();
    writeln!(pgn, "[Date \"{}\"]", metadata.date).unwrap();
    writeln!(
        pgn,
        "[Round \"{}\"]",
        metadata.round.clone().unwrap_or_else(|| "?".to_string())
    )
    .unwrap();

    // Players & ratings
    writeln!(pgn, "[White \"{}\"]", metadata.white_player_name).unwrap();
    writeln!(pgn, "[Black \"{}\"]", metadata.black_player_name).unwrap();
    writeln!(pgn, "[WhiteElo \"{}\"]", metadata.white_player_elo).unwrap();
    writeln!(pgn, "[BlackElo \"{}\"]", metadata.black_player_elo).unwrap();

    // Result
    writeln!(pgn, "[Result \"{}\"]", metadata.result.to_string()).unwrap();

    // Additional tags
    if let Some(ref tc) = metadata.time_control {
        writeln!(pgn, "[TimeControl \"{}\"]", tc).unwrap();
    }
    if let Some(ref eco) = metadata.eco {
        writeln!(pgn, "[ECO \"{}\"]", eco).unwrap();
    }
    if let Some(ref opening) = metadata.opening {
        writeln!(pgn, "[Opening \"{}\"]", opening).unwrap();
    }
    if let Some(ref end_time) = metadata.end_time {
        writeln!(pgn, "[EndTime \"{}\"]", end_time).unwrap();
    }
    if let Some(ref link) = metadata.link {
        writeln!(pgn, "[Link \"{}\"]", link).unwrap();
    }
    if let Some(term_str) = termination_to_string(&metadata.termination) {
        writeln!(pgn, "[Termination \"{}\"]", term_str).unwrap();
    }

    // Moves - SAN, comments and NAGs
    let mut move_line = String::new();
    let mut move_number = 1;
    for (i, mv) in metadata.move_list.iter().enumerate() {
        if i % 2 == 0 {
            write!(move_line, "{}. ", move_number).unwrap();
            move_number += 1;
        }
        write!(move_line, "{}", mv.san).unwrap();

        // Build a single comment string containing clock/timestamp (if any)
        // placed before the existing annotation (if any) in the same { ... }.
        let mut comment_parts = String::new();
        if let (Some(clk), Some(ts)) = (mv.clock.as_ref(), mv.time_stamp) {
            write!(comment_parts, "[%clk {}][%timestamp {}]", clk, ts).unwrap();
        } else if let Some(clk) = mv.clock.as_ref() {
            write!(comment_parts, "[%clk {}]", clk).unwrap();
        } else if let Some(ts) = mv.time_stamp {
            write!(comment_parts, "[%timestamp {}]", ts).unwrap();
        }

        if let Some(ref comment) = mv.annotation {
            if !comment_parts.is_empty() {
                comment_parts.push(' ');
            }
            comment_parts.push_str(comment);
        }

        if !comment_parts.is_empty() {
            write!(move_line, " {{{}}}", comment_parts).unwrap();
        }

        if let Some(nag) = mv.nag {
            write!(move_line, " ${}", nag).unwrap();
        }
        move_line.push(' ');
    }

    // Final result token (if moves present or not)
    write!(move_line, "{}", metadata.result.to_string()).unwrap();

    // Combine tags and moves
    pgn.push('\n');
    pgn.push_str(&move_line.trim_end());
    pgn.push('\n');

    pgn
}
impl BoardMetaData {
    pub fn to_pgn(&self) {}
}
pub fn get_game(id: usize) -> BoardMetaData {
    let con = Connection::open("chess.db").expect("Failed to open database");

    let mut stmt = con
        .prepare("SELECT pgn_data FROM games WHERE game_id = ?1")
        .expect("Failed to prepare statement");

    let pgn: String = stmt
        .query_row([id as u32], |row| row.get(0))
        .expect("Game not found");

    parse_pgn_string(pgn)
}
pub enum SaveType {
    MetaDataSave { data: BoardMetaData },
    GameSave,
}
pub fn save_game(metadata: &BoardMetaData) -> Result<(), rusqlite::Error> {
    let con = Connection::open("chess.db")?;

    let pgn_data = metadata_to_pgn(&metadata);
    let time_control = metadata.time_control.clone().unwrap_or_default();
    println!("Saving to time_control: '{}'", time_control);

    con.execute(
        "INSERT INTO games (
            date_played,
            white_player,
            black_player,
            white_elo,
            black_elo,
            result,
            opening,
            pgn_data,
            time_control
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            metadata.date,
            metadata.white_player_name,
            metadata.black_player_name,
            metadata.white_player_elo,
            metadata.black_player_elo,
            metadata.result.to_string(),
            metadata.opening.clone().unwrap_or_default(),
            pgn_data,
            time_control
        ],
    )?;

    Ok(())
}

pub fn get_game_list() -> Result<Vec<BoardMetaData>, rusqlite::Error> {
    let con = Connection::open("chess.db")?;

    let mut stmt = con.prepare(
        "SELECT game_id, date_played, white_player, black_player, result, opening, pgn_data, white_elo, black_elo, time_control
         FROM games",
    )?;

    let rows = stmt.query_map([], |row| {
        let mut meta = BoardMetaData::default();

        // column indices: 0=game_id,1=date_played,2=white_player,3=black_player,4=result,5=opening,6=pgn_data,7=white_elo,8=black_elo
        let _game_id: i64 = row.get(0)?;
        meta.date = row.get::<_, String>(1)?;
        meta.white_player_name = row.get::<_, String>(2)?;
        meta.black_player_name = row.get::<_, String>(3)?;
        // read as Option<u32> to handle NULLs
        meta.white_player_elo = row.get::<_, Option<u32>>(7)?.unwrap_or(0);
        meta.black_player_elo = row.get::<_, Option<u32>>(8)?.unwrap_or(0);
        let result_str: String = row.get(4)?;
        meta.result = parse_game_result(&result_str);
        meta.opening = row.get::<_, Option<String>>(5)?;
        meta.time_control = row.get::<_, Option<String>>(9)?;
        Ok(meta)
    })?;

    let mut games = Vec::new();
    for r in rows {
        games.push(r?);
    }
    Ok(games)
}
#[tauri::command]
pub fn load_pgn_game(input_string: String) -> Result<(), String> {
    let metadata = parse_pgn_string(input_string);
    if let Err(e) = save_game(&metadata) {
        eprintln!("Error saving game: {}", e);
        return Err(e.to_string());
    }
    Ok(())
}
pub fn get_game_by_id(game_id: usize) -> Result<BoardMetaData, rusqlite::Error> {
    let con = Connection::open("chess.db")?;
    let mut stmt = con.prepare("SELECT pgn_data FROM games WHERE game_id = ?1")?;
    let pgn: String = stmt.query_row([game_id as u32], |row| row.get(0))?;
    Ok(parse_pgn_string(pgn))
}
pub fn get_game_chat_by_id(game_id: usize) -> Result<LocalChat, rusqlite::Error> {
    //check if chat with this id existis
    let chat = {
        let con = Connection::open("chess.db")?;
        let mut stmt = con.prepare("Select chat_id from chats where game_id = ?1")?;
        let chat = stmt
            .query_map([game_id as u32], |row| {
                let chat_id: i64 = row.get(0)?;
                Ok(chat_id)
            })?
            .filter_map(Result::ok)
            .collect::<Vec<i64>>();
        if chat.len() == 0 {
            // There is no conversation for set game, so create one and link it to the game
            let chat_id = {
                let mut insert_stmt = con.prepare("INSERT INTO chats (game_id) VALUES (?1)")?;
                insert_stmt.execute([game_id as u32])?;
                // Get the last inserted chat_id
                con.last_insert_rowid()
            };
            println!("Created chat wit id {}", chat_id);
            LocalChat {
                chat_id: chat_id as i32,
                chat_messages: Vec::new(),
            }
        } else {
            //cosntruct messages
            let chat_id = chat[0];
            let mut stmt = con.prepare(
                "Select role, content, move_index, sent_at from messages where chat_id = ?1",
            )?;
            let result: Vec<LocalMessage> = stmt
                .query_map([chat_id], |row| {
                    Ok(LocalMessage {
                        role: LocalMessageRole::from(
                            row.get::<_, String>(0).unwrap_or("User".to_string()),
                        ),
                        content: row.get(1)?,
                        move_index: row.get(2)?,
                        sent_at: row.get(3)?,
                    })
                })?
                .filter_map(Result::ok)
                .collect();
            LocalChat {
                chat_id: chat_id as i32,
                chat_messages: result,
            }
        }
    };
    Ok(chat)
}
impl LocalChat {
    pub fn save(&self) -> Result<(), String> {
        let chat_id = self.chat_id;
        let con = Connection::open("chess.db").map_err(|e| "Failed to open db".to_string())?;
        let mut stmt = con
            .prepare(
                "Insert into messages (chat_id, role, content, move_index, sent_at) Values (?1, ?2, ?3, ?4, ?5) ON CONFLICT(content) DO NOTHING",
            )
            .map_err(|e| e.to_string())?;
        for chat_message in &self.chat_messages {
            let res = stmt
                .execute((
                    chat_id,
                    chat_message.role.to_string(),
                    &chat_message.content,
                    &chat_message.move_index,
                    &chat_message.sent_at,
                ))
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
