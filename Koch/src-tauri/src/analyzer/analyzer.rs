use std::sync::mpsc::{sync_channel, Receiver, RecvError, SyncSender, TryRecvError};
use std::time::{Duration, Instant};
use std::{
    collections::HashMap,
    fs::OpenOptions,
    sync::{
        mpsc::{self, RecvTimeoutError},
        Mutex,
    },
    thread,
};
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum EngineCommand {
    SetFen(String),
    GoInfinite,
    Stop,
    Quit,
    SetAndGo(String, i32), // <- CHANGED: add i32 multiplier
    SetMultiPv(usize),
    SetHashSize(usize),
    SetThreads(usize),
    GetThreat(String, i32),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum EngineOption {
    MultiPv,
    Threads,
    Hash,
}
use serde::{Deserialize, Serialize};
use stockfish::Stockfish;
use tauri::{AppHandle, Emitter, Manager};
use ts_rs::TS;

use crate::engine::Board;
use crate::{
    engine::{
        serializer::{serialize_analyzer_controller, SerializedAnalyzerController},
        ChessPiece, PieceColor, PieceType,
    },
    server::server::{load_settings, EvalKind, PvLineData, PvObject, ServerState},
};
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BoardState {
    pub turn: PieceColor,
    pub white_big_castle: bool,
    pub white_small_castle: bool,
    pub black_big_castle: bool,
    pub black_small_castle: bool,
    pub en_passant_target: Option<(u8, u8)>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

// per-kind move payloads
#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub enum MoveKind {
    Normal {
        promotion: Option<PieceType>,
    },
    EnPassant {
        captured_at: (u8, u8),
    },
    Castling {
        rook_from: (u8, u8),
        rook_to: (u8, u8),
    },
}

// compact undo record combining common + variant
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UndoInfo {
    pub from: (u8, u8),
    pub to: (u8, u8),
    pub captured: Option<ChessPiece>,
    pub prev_state: BoardState,
    pub kind: MoveKind,
}
#[derive(Clone, TS, Serialize, Deserialize, Debug)]
#[ts(export)]

pub enum AiChatMessageRole {
    System,
    User,
    Assistant,
    Function,
    Tool,
}

impl From<String> for AiChatMessageRole {
    fn from(value: String) -> Self {
        match value.as_str() {
            "System" => AiChatMessageRole::System,
            "User" => AiChatMessageRole::User,
            "Assistant" => AiChatMessageRole::Assistant,
            "Function" => AiChatMessageRole::Function,
            "Tool" => AiChatMessageRole::Tool,
            _ => AiChatMessageRole::User, // fallback/default
        }
    }
}
impl ToString for AiChatMessageRole {
    fn to_string(&self) -> String {
        match self {
            AiChatMessageRole::System => "System".to_string(),
            AiChatMessageRole::User => "User".to_string(),
            AiChatMessageRole::Assistant => "Assistant".to_string(),
            AiChatMessageRole::Function => "Function".to_string(),
            AiChatMessageRole::Tool => "Tool".to_string(),
        }
    }
}
impl From<&str> for AiChatMessageRole {
    fn from(value: &str) -> Self {
        match value {
            "System" => AiChatMessageRole::System,
            "User" => AiChatMessageRole::User,
            "Assistant" => AiChatMessageRole::Assistant,
            "Function" => AiChatMessageRole::Function,
            "Tool" => AiChatMessageRole::Tool,
            _ => AiChatMessageRole::User, // fallback/default
        }
    }
}
#[derive(Clone, TS, Serialize, Deserialize, Debug)]
#[ts(export)]
pub struct AiChatMessage {
    pub role: AiChatMessageRole,
    pub text: String,
    pub sent_at: String,
    pub move_index: i32,
}

#[derive(Clone, TS, Serialize)]
#[ts(export)]
pub enum LocalMessageRole {
    User,
    Assistent,
}

impl ToString for LocalMessageRole {
    fn to_string(&self) -> String {
        match self {
            LocalMessageRole::User => "User".to_string(),
            LocalMessageRole::Assistent => "Assistent".to_string(),
        }
    }
}
impl From<String> for LocalMessageRole {
    fn from(value: String) -> Self {
        match value.as_str() {
            "User" => LocalMessageRole::User,
            "Assistent" => LocalMessageRole::Assistent,
            _ => LocalMessageRole::User, // fallback/default
        }
    }
}

impl From<&str> for LocalMessageRole {
    fn from(value: &str) -> Self {
        match value {
            "User" => LocalMessageRole::User,
            "Assistent" => LocalMessageRole::Assistent,
            _ => LocalMessageRole::User, // fallback/default
        }
    }
}

#[derive(Clone, TS, Serialize)]
#[ts(export)]

pub struct LocalChat {
    pub chat_id: i32,
    pub chat_messages: Vec<LocalMessage>,
}
impl Default for LocalChat {
    fn default() -> Self {
        Self {
            chat_id: -1,
            chat_messages: Vec::new(),
        }
    }
}
#[derive(Clone, TS, Serialize)]
#[ts(export)]

pub struct LocalMessage {
    pub role: LocalMessageRole,
    pub content: String,
    pub move_index: isize,
    pub sent_at: String,
}

impl Default for LocalMessage {
    fn default() -> Self {
        Self {
            role: LocalMessageRole::User,
            content: String::new(),
            move_index: -1,
            sent_at: String::new(),
        }
    }
}
impl From<rig::completion::Message> for LocalMessage {
    fn from(m: rig::completion::Message) -> Self {
        // Map the fields accordingly
        match m {
            rig::message::Message::Assistant { id, content } => Self {
                role: LocalMessageRole::Assistent,
                content: content
                    .iter()
                    .map(|cont| match cont {
                        rig::message::AssistantContent::Text(a) => a.text.clone(),
                        _ => "unrecognized content".into(),
                    })
                    .collect::<Vec<String>>()
                    .join(" "),
                ..Default::default()
            },
            rig::message::Message::User { content } => Self {
                role: LocalMessageRole::User,
                content: content
                    .iter()
                    .map(|cont| match cont {
                        rig::message::UserContent::Text(a) => a.text.clone(),
                        _ => "unrecognized content".into(),
                    })
                    .collect::<Vec<String>>()
                    .join(" "),
                ..Default::default()
            },
        }
    }
}
impl From<LocalMessage> for rig::completion::Message {
    fn from(value: LocalMessage) -> Self {
        match value.role {
            LocalMessageRole::User => rig::completion::Message::user(value.content),
            LocalMessageRole::Assistent => rig::completion::Message::assistant(value.content),
        }
    }
}
#[derive(Clone, TS, Serialize)]
#[ts(export)]
pub struct AnalyzerController {
    pub game_id: usize,
    pub board: Board,
    pub current_ply: i32,
    pub board_undo: Vec<UndoInfo>,
    pub last_threat: Option<String>,
    pub last_pv: Option<PvObject>,
    pub chat_history: LocalChat,
}

impl Default for AnalyzerController {
    fn default() -> Self {
        Self {
            game_id: 0,
            board: Board::default(),
            // start at -1 to represent the initial position (no moves applied)
            current_ply: -1,
            board_undo: Vec::new(),
            last_threat: None,
            last_pv: None,
            chat_history: LocalChat::default(),
        }
    }
}

impl AnalyzerController {
    pub fn get_fen(&self) -> String {
        return self.board.to_string();
    }
}

#[tauri::command]
pub fn stop_analyzer(state: tauri::State<'_, Mutex<ServerState>>) -> bool {
    let state = state.lock().unwrap();
    match &state.analyzer_tx {
        Some(tx) => match tx.send(EngineCommand::Stop) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("[Analyzer] stop_analyzer send failed: {e}");
                false
            }
        },
        None => {
            eprintln!("[Analyzer] stop_analyzer: tx missing");
            false
        }
    }
}

pub fn start_analyzer_thread(
    app_handle: AppHandle,
) -> (SyncSender<EngineCommand>, Receiver<PvObject>) {
    let (cmd_tx, cmd_rx) = sync_channel::<EngineCommand>(64);
    let (pv_tx, pv_rx) = sync_channel::<PvObject>(8);

    thread::spawn(move || {
        let stockfish_go_config = String::from("go depth 40 movetime 20000");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            println!("[Analyzer] Thread starting...");
            let mut engine = match Stockfish::new("/usr/bin/stockfish") {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("[Analyzer] Failed to start engine: {e}");
                    return;
                }
            };

            let mut is_searching = false;
            let mut current_fen = String::new();
            let mut current_pv = PvObject::default();
            let mut color_multiplier: i32 = 1; // <- NEW: multiplier for eval (white perspective)

            let start_time = Instant::now();
            let mut last_clock_tick = Instant::now();
            match load_settings() {
                Ok(mut settings) => {
                    let pv = settings
                        .map
                        .entry("MultiPV".to_string())
                        .or_insert("1".to_string());
                    match engine.set_option("MultiPV", pv) {
                        Ok(()) => {}
                        Err(e) => eprintln!("{e}"),
                    }

                    let th = settings
                        .map
                        .entry("Threads".to_string())
                        .or_insert("1".to_string());
                    match engine.set_option("Threads", th) {
                        Ok(()) => {}
                        Err(e) => eprintln!("{e}"),
                    }

                    let hs = settings
                        .map
                        .entry("HashSize".to_string())
                        .or_insert("128".to_string());
                    match engine.set_option("HashSize", hs) {
                        Ok(()) => {}
                        Err(e) => eprintln!("{e}"),
                    }
                }
                Err(e) => {
                    eprint!("{e}");
                }
            }

            loop {
                // Handle commands
                match cmd_rx.try_recv() {
                    Ok(command) => {
                        match command {
                            EngineCommand::SetFen(fen) => {
                                if is_searching {
                                    is_searching = false;
                                    let _ = engine.uci_send("stop");
                                    drain_until_bestmove(&mut engine);
                                }
                                engine.ensure_ready().ok();

                                match engine.set_fen_position(&fen) {
                                    Ok(_) => println!("[Analyzer] FEN set successfully: {}", &fen),
                                    Err(e) => eprintln!("[Analyzer] Failed to set FEN: {}", e),
                                }
                                current_fen = fen;
                                current_pv = PvObject {
                                    fen: current_fen.clone(),
                                    depth: 0,
                                    lines: HashMap::new(),
                                };

                                let _ = app_handle.emit("pv_update", current_pv.clone());
                            }
                            EngineCommand::GoInfinite => {
                                engine.ensure_ready().ok();
                                match engine.uci_send(&stockfish_go_config).ok() {
                                    Some(_) => is_searching = true,
                                    None => eprintln!("[Analyzer] fif not sbt stockfish go ocngic"),
                                };
                            }
                            EngineCommand::Stop => {
                                if is_searching {
                                    is_searching = false;
                                    let _ = engine.uci_send("stop");
                                    drain_until_bestmove(&mut engine);
                                }
                            }
                            EngineCommand::Quit => break,

                            EngineCommand::SetAndGo(fen, mult) => {
                                if is_searching {
                                    is_searching = false;
                                    let _ = engine.uci_send("stop");
                                    drain_until_bestmove(&mut engine);
                                }
                                engine.ensure_ready().ok();
                                //position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1  moves e2e4 g7g6 d2d4 f8g7s
                                match engine.uci_send(&fen) {
                                    Ok(_) => println!("[Analyzer] FEn Set: {}", &fen),
                                    Err(e) => eprintln!("{e}"),
                                };
                                current_fen = fen;
                                color_multiplier = mult; // <- store multiplier for later
                                current_pv = PvObject {
                                    fen: current_fen.clone(),
                                    depth: 0,
                                    lines: HashMap::new(),
                                };
                                engine.ensure_ready().ok();

                                let _ = app_handle.emit("pv_update", current_pv.clone());

                                if let Err(e) = engine.uci_send(&stockfish_go_config) {
                                    eprintln!("Failed to send go infinite: {e}");
                                } else {
                                    is_searching = true;
                                }
                            }
                            EngineCommand::GetThreat(fen, _mult) => {
                                let app_handle_clone = app_handle.clone();
                                let flipped_fen = flip_fen_turn(&fen);

                                // 1. Spawning a NEW thread for the transient engine
                                thread::spawn(move || {
                                    println!("[Analyzer] Starting background threat search...");

                                    // 2. This engine is TOTALLY SEPARATE from the main one
                                    if let Ok(mut temp_engine) =
                                        Stockfish::new("/usr/bin/stockfish")
                                    {
                                        temp_engine.set_fen_position(&flipped_fen).ok();

                                        // 3. This go_for now owns its own pipe exclusively
                                        match temp_engine.go() {
                                            Ok(out) => {
                                                let threat_move = out.best_move();
                                                println!("[Analyzer]THreat foun {}", threat_move);
                                                // Update the global state and UI
                                                if let Ok(mut global_state) = app_handle_clone
                                                    .state::<Mutex<ServerState>>()
                                                    .lock()
                                                {
                                                    global_state.analyzer_controller.last_threat =
                                                        Some(threat_move.into());
                                                }
                                                let _ = app_handle_clone
                                                    .emit("threat_update", threat_move);
                                            }
                                            Err(e) => eprintln!("[Threat Search] Error: {e}"),
                                        }
                                        // temp_engine goes out of scope and shuts down cleanly here
                                    }
                                });
                            }
                            EngineCommand::SetHashSize(hash) => {
                                engine.ensure_ready().ok();
                                match engine.set_option("Hash", &hash.to_string()) {
                                    Ok(_) => {
                                        println!("[Analyzer] Set Hash to {} success", hash)
                                    }
                                    Err(e) => {
                                        eprintln!("[Analyzer] Set Hash to {} failed: {}", hash, e)
                                    }
                                }
                            }
                            EngineCommand::SetMultiPv(cnt) => {
                                engine.ensure_ready().ok();
                                match engine.set_option("MultiPV", &cnt.to_string()) {
                                    Ok(_) => {
                                        println!("[Analyzer] Set MultiPV to {} success", cnt)
                                    }
                                    Err(e) => {
                                        eprintln!("[Analyzer] Set MultiPV to {} failed: {}", cnt, e)
                                    }
                                }
                            }
                            EngineCommand::SetThreads(tcnt) => {
                                engine.ensure_ready().ok();
                                match engine.set_option("Threads", &tcnt.to_string()) {
                                    Ok(_) => {
                                        println!("[Analyzer] Set Threads to {} success", tcnt)
                                    }
                                    Err(e) => eprintln!(
                                        "[Analyzer] Set Threads to {} failed: {}",
                                        tcnt, e
                                    ),
                                }
                            }
                        }
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => break,
                }

                // Read Stockfish output, log each line
                if is_searching {
                    if last_clock_tick.elapsed() >= Duration::from_secs(1) {
                        let uptime = start_time.elapsed().as_secs_f64();
                        //println!("[Analyzer][clock={uptime:.3}s] Searching...");
                        last_clock_tick = Instant::now();
                    }

                    let line = engine.read_line();
                    if line.is_empty() {
                        continue;
                    }

                    // Append raw engine line to logfile
                    //let _ = writeln!(f_log, "{line}");

                    if line.starts_with("bestmove") {
                        // Keep search alive; ignore spontaneous bestmove
                        let _ = engine.uci_send(&stockfish_go_config);
                        continue;
                    }

                    if line.contains(" multipv ") && line.contains(" pv ") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        let mut multipv_idx: u8 = 1;
                        let mut depth: u32 = 0;
                        let mut moves = String::new();
                        let mut score_value: i32 = 0;
                        let mut score_kind = EvalKind::Centipawn;

                        let mut i = 0;
                        while i < parts.len() {
                            match parts[i] {
                                "multipv" if i + 1 < parts.len() => {
                                    multipv_idx = parts[i + 1].parse().unwrap_or(1);
                                    i += 1;
                                }
                                "depth" if i + 1 < parts.len() => {
                                    depth = parts[i + 1].parse().unwrap_or(0);
                                    i += 1;
                                }
                                "score" if i + 2 < parts.len() => {
                                    score_kind = if parts[i + 1] == "mate" {
                                        EvalKind::Mate
                                    } else {
                                        EvalKind::Centipawn
                                    };
                                    score_value = parts[i + 2].parse().unwrap_or(0);
                                    i += 2;
                                }
                                "pv" => {
                                    moves = parts[i + 1..].join(" ");
                                    break;
                                }
                                _ => {}
                            }
                            i += 1;
                        }

                        if depth >= current_pv.depth {
                            current_pv.depth = depth;
                            let line_data = PvLineData {
                                moves,
                                eval_kind: score_kind,
                                // <- Apply normalization here
                                eval_value: score_value * color_multiplier,
                            };
                            current_pv.lines.insert(multipv_idx, line_data);
                            if let Ok(mut global_state) =
                                app_handle.state::<Mutex<ServerState>>().lock()
                            {
                                global_state.analyzer_controller.last_pv = Some(current_pv.clone());
                            }
                            let _ = app_handle.emit("pv_update", current_pv.clone());
                        }
                    }
                } else {
                    thread::sleep(Duration::from_millis(10));
                }
            }
            println!("[Analyzer] Thread exited normally");
        }));

        if let Err(e) = result {
            eprintln!("[Analyzer] CRITICAL: Thread panicked! {:?}", e);
        }
    });

    (cmd_tx, pv_rx)
}
/*
#[tauri::command]
pub fn try_analyzer_move(
    state: tauri::State<'_, Mutex<ServerState>>,
    src_square: (u8, u8),
    dest_square: (u8, u8),
    promotion: Option<PieceType>,
) -> Option<SerializedAnalyzerController> {
    let mut state = state.lock().unwrap();

    // Ensure move cache exists (prevents missing entries for sliding pieces)
    if state.board.move_cache.is_empty() {
        state.board.rerender_move_cache();
    }

    // Capture target BEFORE moving (after move the destination holds the moving piece)
    let captured_before =
        state.board.squares[dest_square.0 as usize][dest_square.1 as usize].clone();
    match state
        .analyzer_controller
        .board
        .move_piece(src_square, dest_square, promotion)
    {
        Ok(mut mv) => {

            mv.uci =
                state
                    .analyzer_controller
                    .board
                    .encode_uci_move(src_square, dest_square, promotion);

            state.analyzer_controller.board.meta_data.move_list.push(mv);
            // Refresh move cache after a successful move
            state.analyzer_controller.current_ply = state
                .analyzer_controller
                .board
                .meta_data
                .move_list
                .len()
                .saturating_sub(1) as i32;
            state.analyzer_controller.board.rerender_move_cache();
            return Some(serialize_analyzer_controller(&state.analyzer_controller));
        }

        Err(e) => {
            dbg!(e);
        }
    }
    None
}
 */
#[tauri::command]
pub fn set_analyzer_fen(state: tauri::State<'_, Mutex<ServerState>>, current_move: isize) -> bool {
    let state = state.lock().unwrap();
    println!("Called set fen for{}", current_move);

    let start_fen = state
        .analyzer_controller
        .board
        .meta_data
        .starting_position
        .clone();
    let fengo = format! {"position fen {} ", start_fen};
    let mut fen = fengo;

    // Determine starting side to move from FEN (field 2 is "w" or "b")
    let fen_parts: Vec<&str> = start_fen.split_whitespace().collect();
    let mut white_to_move = if fen_parts.len() >= 2 {
        fen_parts[1] == "w"
    } else {
        true // default if malformed
    };

    if current_move != -1 {
        let mut moves = String::new();
        for i in 0..=current_move {
            match state
                .analyzer_controller
                .board
                .meta_data
                .move_list
                .get(i as usize)
            {
                Some(mv) => {
                    moves.push_str(&format!("{} ", mv.uci.clone()));
                    // flip side to move for each half-move
                    white_to_move = !white_to_move;
                }
                None => {}
            }
        }
        if !moves.is_empty() {
            fen = format!("{fen} moves {moves}");
        }
    }

    // Normalize eval to White perspective:
    //   if white_to_move: multiplier = 1  (Stockfish eval is from White's POV)
    //   if black_to_move: multiplier = -1 (Stockfish eval is from Black's POV)
    let multiplier: i32 = if white_to_move { 1 } else { -1 };

    // //println!(
    //     "{} | side_to_move={} | multiplier={}",
    //     &fen,
    //     if white_to_move { "w" } else { "b" },
    //     multiplier
    // );

    let Some(tx) = &state.analyzer_tx else {
        eprintln!("[Analyzer] tx missing");
        return false;
    };

    if tx.send(EngineCommand::SetAndGo(fen, multiplier)).is_err() {
        eprintln!("[Analyzer] SetAndGo send failed");
        return false;
    }

    true
}
#[tauri::command]
pub fn set_engine_option(
    state: tauri::State<'_, Mutex<ServerState>>,
    option: EngineOption,
    value: &str,
) {
    let mut state = state.lock().unwrap();
    if let Some(tx) = &state.analyzer_tx {
        match tx.send(EngineCommand::Stop) {
            Ok(_) => {}
            Err(_) => {}
        }
    }
    if let Some(tx) = &state.analyzer_tx {
        match option {
            EngineOption::Hash => match value.parse::<usize>() {
                Ok(hash_size) => match tx.send(EngineCommand::SetHashSize(hash_size)) {
                    Ok(_) => {
                        println!("[Analyzer] Set HashSize to {} sent successfully", hash_size);
                        state
                            .settings
                            .update("HashSize".to_string(), value.to_string());
                        if let Err(e) = state.settings.save() {
                            eprintln!("[Analyzer] Failed to save settings: {}", e);
                        }
                    }
                    Err(e) => eprintln!("[Analyzer] Failed to send SetHashSize: {}", e),
                },
                Err(e) => eprintln!("[Analyzer] Invalid hash size value '{}': {}", value, e),
            },
            EngineOption::Threads => match value.parse::<usize>() {
                Ok(thread_count) => match tx.send(EngineCommand::SetThreads(thread_count)) {
                    Ok(_) => {
                        println!(
                            "[Analyzer] Set Threads to {} sent successfully",
                            thread_count
                        );
                        state
                            .settings
                            .update("Threads".to_string(), value.to_string());
                        if let Err(e) = state.settings.save() {
                            eprintln!("[Analyzer] Failed to save settings: {}", e);
                        }
                    }
                    Err(e) => eprintln!("[Analyzer] Failed to send SetThreads: {}", e),
                },
                Err(e) => eprintln!("[Analyzer] Invalid thread count value '{}': {}", value, e),
            },
            EngineOption::MultiPv => match value.parse::<usize>() {
                Ok(pv_count) => match tx.send(EngineCommand::SetMultiPv(pv_count)) {
                    Ok(_) => {
                        println!("[Analyzer] Set MultiPv to {} sent successfully", pv_count);
                        state
                            .settings
                            .update("MultiPV".to_string(), value.to_string());
                        if let Err(e) = state.settings.save() {
                            eprintln!("[Analyzer] Failed to save settings: {}", e);
                        }
                    }
                    Err(e) => eprintln!("[Analyzer] Failed to send SetMultiPv: {}", e),
                },
                Err(e) => eprintln!("[Analyzer] Invalid MultiPv value '{}': {}", value, e),
            },
        }
    }
}
#[tauri::command]
pub fn get_analyzer_settings(
    state: tauri::State<'_, Mutex<ServerState>>,
) -> Option<(usize, usize, usize)> {
    let mut state = state.lock().unwrap();
    let pvs = match state.settings.map.get("MultiPV") {
        Some(val) => match val.parse::<usize>() {
            Ok(v) => v,
            Err(_) => return None,
        },
        None => return None,
    };
    let th = match state.settings.map.get("Threads") {
        Some(val) => match val.parse::<usize>() {
            Ok(v) => v,
            Err(_) => return None,
        },
        None => return None,
    };
    let hs = match state.settings.map.get("HashSize") {
        Some(val) => match val.parse::<usize>() {
            Ok(v) => v,
            Err(_) => return None,
        },
        None => return None,
    };
    Some((pvs, th, hs))
}
fn flip_fen_turn(fen: &str) -> String {
    let mut parts: Vec<&str> = fen.split_whitespace().collect();
    if parts.len() > 1 {
        // Switch 'w' to 'b' or 'b' to 'w'
        parts[1] = if parts[1] == "w" { "b" } else { "w" };
    }
    parts.join(" ")
}
pub fn drain_until_bestmove(engine: &mut Stockfish) {
    loop {
        let l = engine.read_line();
        if l.starts_with("bestmove") {
            break;
        }
        // Log stop-drain lines too
    }
}
#[tauri::command]
pub fn get_threat(state: tauri::State<'_, Mutex<ServerState>>) {
    let mut state = state.lock().unwrap();
    let fen = state.analyzer_controller.board.to_string();
    let multiplier: i32 = if fen.contains('w') { 1 } else { -1 };

    let Some(tx) = &state.analyzer_tx else {
        eprintln!("[Analyzer] tx missing");
        return;
    };

    if tx.send(EngineCommand::GetThreat(fen, multiplier)).is_err() {
        eprintln!("[Analyzer] GetThreat send failed");
        return;
    }
}
