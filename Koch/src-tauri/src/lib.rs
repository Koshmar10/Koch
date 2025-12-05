// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod engine;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;

//pub mod game;
pub mod analyzer;
pub mod database;
pub mod etc;
pub mod game;
pub mod server;

use crate::analyzer::board_interactions::{do_move, get_fen, undo_move, AnalyzerController};
use crate::database::create::{get_game_by_id, get_game_list};
use crate::engine::board::{BoardMetaData, EvalResponse, GameResult};
use crate::server::server::{EngineCommand, PvObject};
use crate::{
    engine::board,
    engine::{board::PieceMoves, Board, PieceColor, PieceType},
    game::controller::{GameController, TerminationBy},
    server::server::ServerState,
};
use std::time::Duration;
use stockfish::Stockfish;
use tauri::{Builder, Manager};

fn make_engine_move(state: &mut ServerState, fen: String) -> Option<String> {
    let engine = &mut state.engine;
    match engine {
        Some(engine) => {
            engine
                .set_fen_position(&fen)
                .inspect_err(|e| eprintln!("{e}"))
                .ok();
            let engine_output = engine.go().inspect_err(|e| eprintln!("{e}")).ok();
            match engine_output {
                Some(engine_output) => {
                    let uci_move = engine_output.best_move();
                    return Some(uci_move.into());
                }
                None => None,
            }
        }
        None => None,
    }
}
#[tauri::command]
fn promote_pawn(
    state: tauri::State<'_, Mutex<ServerState>>,
    pos: (u8, u8),
    kind: PieceType,
) -> Board {
    let mut state = state.lock().unwrap();
    state.board.promote_pawn(pos, kind);
    state.board.rerender_move_cache();
    return state.board.clone();
}
#[tauri::command]
fn start_game(state: tauri::State<'_, Mutex<ServerState>>) -> (Board, Option<GameController>) {
    let user_elo = 800;
    let engine_elo = 1000;
    let engine_name = "Stockfish".to_string();
    let user_name = "Petru".to_string();

    let mut state = state.lock().unwrap();
    let new_game = GameController::new();
    let mut new_board = Board::default();
    new_board.rerender_move_cache();
    state.board = new_board;
    match &new_game.player {
        PieceColor::White => {
            state.board.meta_data.white_player_name = user_name;
            state.board.meta_data.white_player_elo = user_elo;
            state.board.meta_data.black_player_name = engine_name;
            state.board.meta_data.black_player_elo = engine_elo;
        }
        PieceColor::Black => {
            state.board.meta_data.white_player_name = engine_name;
            state.board.meta_data.white_player_elo = engine_elo;
            state.board.meta_data.black_player_name = user_name;
            state.board.meta_data.black_player_elo = user_elo;
        }
    }
    state.game_controller = Some(new_game);
    (state.board.clone(), state.game_controller.clone())
}
#[tauri::command]
fn update_gameloop(state: tauri::State<'_, Mutex<ServerState>>) -> (Board, Option<GameController>) {
    let mut state = state.lock().unwrap();
    let mut game = state.game_controller.clone();
    let game_over = game.as_ref().unwrap().game_over;
    match &mut game {
        Some(game) => {
            if state.board.is_chackmate() {
                game.lost_by = Some(TerminationBy::Checkmate);
                state.board.meta_data.result = match &state.board.turn {
                    PieceColor::Black => board::GameResult::WhiteWin,
                    PieceColor::White => board::GameResult::BlackWin,
                };
                state.board.meta_data.termination = board::TerminationBy::Checkmate;
                game.game_over = true;
            }
            if state.board.is_stale_mate() {
                game.lost_by = Some(TerminationBy::StaleMate);
                state.board.meta_data.termination = board::TerminationBy::StaleMate;
                state.board.meta_data.result = GameResult::Draw;
                game.game_over = true;
            }
        }
        None => {}
    }
    if game_over {
        return (state.board.clone(), game);
    }
    // If no game controller, just return current board
    if state.game_controller.is_none() {
        return (state.board.clone(), game);
    }

    let player_color = state.game_controller.as_ref().unwrap().player;
    let engine_color = state.game_controller.as_ref().unwrap().enemy;

    // If it's still the human player's turn, do nothing

    if state.board.turn == player_color {
        let in_check = state.board.is_in_check(player_color);
        if let Some(g) = game.as_mut() {
            g.in_check = in_check;
        }

        return (state.board.clone(), game);
    }

    // Engine's turn
    if state.board.turn == engine_color {
        let fen = state.board.to_string();
        if let Some(uci) = make_engine_move(&mut state, fen) {
            if let Some((from, to)) = state.board.decode_uci_move(uci.clone()) {
                // Capture target before move
                let captured_before = state.board.squares[to.0 as usize][to.1 as usize].clone();

                match state.board.move_piece(from, to) {
                    Ok(mv_struct) => {
                        // Track captures for UI
                        if mv_struct.is_capture {
                            if let Some(captured_piece) = captured_before {
                                match captured_piece.color {
                                    PieceColor::Black => state
                                        .board
                                        .ui
                                        .white_taken
                                        .push((captured_piece.kind, captured_piece.color)),
                                    PieceColor::White => state
                                        .board
                                        .ui
                                        .black_taken
                                        .push((captured_piece.kind, captured_piece.color)),
                                }
                            }
                        }
                        state.board.meta_data.move_list.push(mv_struct);
                        state.board.rerender_move_cache();
                    }
                    Err(e) => {
                        eprintln!("Engine move failed {:?} -> {:?}: {:?}", from, to, e);
                    }
                }
            } else {
                eprintln!("Could not decode UCI move: {}", uci);
            }
        } else {
            eprintln!("Engine produced no move.");
        }
    };

    return (state.board.clone(), game);
}
#[tauri::command]
fn load_fen(state: tauri::State<'_, Mutex<ServerState>>, fen: String) -> Board {
    let mut state = state.lock().unwrap();
    state.board.set_fen(fen);
    state.board.rerender_move_cache();
    return state.board.clone();
}
#[tauri::command]
fn reset_board(state: tauri::State<'_, Mutex<ServerState>>) -> Board {
    let mut state = state.lock().unwrap();
    state.board = Board::default();
    state.board.rerender_move_cache();
    return state.board.clone();
}
#[tauri::command]
fn get_board(state: tauri::State<'_, Mutex<ServerState>>) -> Board {
    let mut state = state.lock().unwrap();
    return state.board.clone();
}
#[tauri::command]
fn get_quote(state: tauri::State<'_, Mutex<ServerState>>) -> String {
    let state = state.lock().unwrap();
    return state.get_quote();
}

#[tauri::command]
fn try_move(
    state: tauri::State<'_, Mutex<ServerState>>,
    src_square: (u8, u8),
    dest_square: (u8, u8),
) -> Option<Board> {
    let mut state = state.lock().unwrap();
    let bc = &mut state.board;

    // Ensure move cache exists (prevents missing entries for sliding pieces)
    if bc.move_cache.is_empty() {
        bc.rerender_move_cache();
    }

    // Capture target BEFORE moving (after move the destination holds the moving piece)
    let captured_before = bc.squares[dest_square.0 as usize][dest_square.1 as usize].clone();

    match bc.move_piece(src_square, dest_square) {
        Ok(move_struct) => {
            if move_struct.is_capture {
                if let Some(captured_piece) = captured_before {
                    match captured_piece.color {
                        PieceColor::Black => bc
                            .ui
                            .white_taken
                            .push((captured_piece.kind, captured_piece.color)),
                        PieceColor::White => bc
                            .ui
                            .black_taken
                            .push((captured_piece.kind, captured_piece.color)),
                    }
                }
            }
            bc.meta_data.move_list.push(move_struct);
            // Refresh move cache after a successful move
            bc.rerender_move_cache();

            Some(bc.clone())
        }
        Err(err) => {
            // Optional: log error
            eprintln!(
                "Illegal move {:?} -> {:?}: {:?}",
                src_square, dest_square, err
            );
            None
        }
    }
}
#[tauri::command]
fn game_into_db(state: tauri::State<'_, Mutex<ServerState>>) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let board = state.board.clone();
    match &mut state.engine {
        Some(stockfish) => {
            let id = database::create::insert_game_and_get_id(&board.meta_data).unwrap_or(0);
            let game_moves = board.meta_data.move_list;
            let board_start_fen = board.meta_data.starting_position;
            let mut test_board = Board::from(&board_start_fen);
            for (idx, mv) in game_moves.iter().enumerate() {
                let mut new_mv = mv.clone();

                let (from, to) = test_board.decode_uci_move(new_mv.uci.clone()).unwrap();
                test_board
                    .move_piece(from, to)
                    .inspect_err(|e| eprintln!("{:?}", e))
                    .ok();
                stockfish
                    .set_fen_position(&test_board.to_string())
                    .inspect_err(|e| eprintln!("{e}"))
                    .ok();
                match stockfish.go() {
                    Ok(out) => {
                        let eval = out.eval();
                        let eval_kind = match &eval.eval_type() {
                            stockfish::EvalType::Centipawn => board::EvalType::Centipawn,
                            stockfish::EvalType::Mate => board::EvalType::Mate,
                        };
                        let eval_score = eval.value();
                        new_mv.evaluation = EvalResponse {
                            value: eval_score as f32,
                            kind: eval_kind,
                        };
                        new_mv.move_number = (idx as u32) + 1;
                        new_mv.time_stamp = idx as f32;
                        println!("{:?}", &new_mv);
                    }
                    Err(e) => {
                        return Err(e.to_string());
                    }
                }
                match database::create::insert_single_move(id, new_mv) {
                    Ok(_) => {
                        println!("move added");
                    }
                    Err(e) => {
                        println!("{}", e);
                        return Err(e.to_string());
                    }
                }
            }
            return Ok(());
        }
        None => {
            println!("nu e stockfish");
            return Ok(());
        }
    }

    Ok(())
}
#[tauri::command]
fn fetch_game_history() -> Vec<BoardMetaData> {
    match get_game_list() {
        Ok(list) => list,
        Err(e) => {
            eprintln!("fetch_game_history DB error: {e}");
            Vec::new()
        }
    }
}
#[tauri::command]
fn fetch_game(state: tauri::State<'_, Mutex<ServerState>>, id: usize) -> AnalyzerController {
    let mut state = state.lock().unwrap();
    if state.analyzer_controller.game_id == id {
        return state.analyzer_controller.clone();
    }
    match get_game_by_id(id) {
        Ok(list) => {
            let mut analyzer = AnalyzerController::default();
            let mut board = Board::from(&list.starting_position);
            board.meta_data = list; // includes full move_list from DB
            analyzer.board = board;
            analyzer.game_id = id;
            analyzer.current_ply = 0;
            analyzer.taken_piece_stack.clear();

            // Persist so do_move/undo_move operate on the same instance
            state.analyzer_controller = analyzer.clone();
            analyzer
        }
        Err(e) => {
            eprintln!("fetch_game_history DB error: {e}");
            AnalyzerController::default()
        }
    }
}
pub fn start_analyzer_thread() -> (mpsc::Sender<EngineCommand>, mpsc::Receiver<PvObject>) {
    let (cmd_tx, cmd_rx): (mpsc::Sender<EngineCommand>, mpsc::Receiver<EngineCommand>) =
        mpsc::channel();
    let (update_tx, update_rx): (mpsc::Sender<PvObject>, mpsc::Receiver<PvObject>) =
        mpsc::channel();

    thread::spawn(move || {
        println!("[Analyzer] Thread starting...");
        let mut engine = match Stockfish::new("/usr/bin/stockfish") {
            Ok(e) => {
                println!("[Analyzer] Stockfish started");
                e
            }
            Err(e) => {
                eprintln!("[Analyzer] Failed to start engine: {e}");
                return;
            }
        };

        // Setup Options
        if let Err(e) = engine.set_option("Threads", "6") {
            eprintln!("[Analyzer] Failed to set Threads: {e}");
        }
        if let Err(e) = engine.set_option("Hash", "1024") {
            eprintln!("[Analyzer] Failed to set Hash: {e}");
        }
        if let Err(e) = engine.set_option("MultiPV", "3") {
            eprintln!("[Analyzer] Failed to set MultiPV: {e}");
        }

        let mut is_searching = false;
        let mut current_fen = String::new();
        let mut current_pv = PvObject::default();

        loop {
            match cmd_rx.try_recv() {
                Ok(command) => {
                    println!("[Analyzer] Command received: {:?}", command);
                    match command {
                        EngineCommand::SetFen(fen) => {
                            if let Err(e) = engine.set_fen_position(&fen) {
                                eprintln!("[Analyzer] set_fen_position error: {e}");
                            } else {
                                println!("[Analyzer] FEN set");
                            }
                            current_fen = fen.clone();
                            current_pv = PvObject {
                                fen,
                                depth: 0,
                                lines: std::collections::HashMap::new(),
                            };
                        }
                        EngineCommand::GoInfinite => match engine.ensure_ready() {
                            Ok(_) => {
                                println!("[Analyzer] Engine ready");
                                if let Err(e) = engine.uci_send("go infinite") {
                                    eprintln!("[Analyzer] go infinite error: {e}");
                                } else {
                                    println!("[Analyzer] Search started");
                                    is_searching = true;
                                }
                            }
                            Err(e) => {
                                eprintln!("[Analyzer] ensure_ready error: {e}");
                            }
                        },
                        EngineCommand::Stop => {
                            if is_searching {
                                if let Err(e) = engine.uci_send("stop") {
                                    eprintln!("[Analyzer] stop error: {e}");
                                } else {
                                    println!("[Analyzer] Stop sent");
                                    is_searching = false;
                                }
                            } else {
                                println!("[Analyzer] Stop ignored (not searching)");
                            }
                        }
                        EngineCommand::Quit => {
                            println!("[Analyzer] Quit received, exiting thread");
                            break;
                        }
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    eprintln!("[Analyzer] Command channel disconnected, exiting thread");
                    break;
                }
            }

            if is_searching {
                let line = engine.read_line();
                if line.is_empty() {
                    continue;
                }
                // Debug raw output
                // println!("[Analyzer] Engine: {line}");

                if line.starts_with("bestmove") {
                    println!("[Analyzer] bestmove received, search finished");
                    is_searching = false;
                    continue;
                }

                if line.contains("multipv") && line.contains(" pv ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    let mut multipv_idx = 0;
                    let mut depth = 0;
                    let mut moves = String::new();

                    for i in 0..parts.len() {
                        if parts[i] == "multipv" {
                            multipv_idx =
                                parts.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(1);
                        }
                        if parts[i] == "depth" {
                            depth = parts.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(0);
                        }
                        if parts[i] == "pv" {
                            moves = parts[i + 1..].join(" ");
                            break;
                        }
                    }

                    if depth >= current_pv.depth {
                        current_pv.depth = depth;
                        current_pv.lines.insert(multipv_idx, moves.clone());
                        if let Err(e) = update_tx.send(current_pv.clone()) {
                            eprintln!("[Analyzer] Failed to send update: {e}");
                        } else {
                            println!(
                                "[Analyzer] Update sent: depth={}, multipv={}, moves={}",
                                depth, multipv_idx, moves
                            );
                        }
                    }
                }
            } else {
                thread::sleep(Duration::from_millis(10));
            }
        }
        println!("[Analyzer] Thread exited");
    });
    (cmd_tx, update_rx)
}
#[tauri::command]
fn set_analyzer_fen(state: tauri::State<'_, Mutex<ServerState>>, fen: &str) {
    println!("set commandt tried");
    let mut state = state.lock().unwrap();
    match &state.analyzer_tx {
        Some(rx) => {
            match rx.send(EngineCommand::Stop) {
                Ok(res) => {
                    println!("analyzer stopped");
                }
                Err(e) => {
                    println!("error on stop");
                }
            };
            match rx.send(EngineCommand::SetFen(fen.to_string())) {
                Ok(res) => {
                    println!("set fen on thread ok");
                }
                Err(e) => {
                    println!("set fen on thread err");
                }
            }
            match rx.send(EngineCommand::GoInfinite) {
                Ok(res) => {
                    println!("analyzer started for new fen");
                }
                Err(e) => {
                    println!("error when trying to start");
                }
            };
        }
        None => {
            println!("not tx");
        }
    }
}

#[tauri::command]
fn stop_analyzer(state: tauri::State<'_, Mutex<ServerState>>) -> bool {
    // println!("poll commandt tried"); // Optional: remove spam logs
    let mut state = state.lock().unwrap();
    match &state.analyzer_tx {
        Some(tx) => {
            match tx.send(EngineCommand::Stop) {
                Ok(res) => {
                    println!("analyzer stopped");
                    return true;
                }
                Err(e) => {
                    println!("error on stop");
                    return true;
                }
            };
        }
        None => return false,
    }
}

#[tauri::command]
fn poll_analyzer(state: tauri::State<'_, Mutex<ServerState>>) -> Option<PvObject> {
    // println!("poll commandt tried"); // Optional: remove spam logs
    let mut state = state.lock().unwrap();
    match &state.analyzer_rx {
        Some(rx) => {
            let mut latest = None;
            // DRAIN the channel: keep reading until empty to get the freshest data
            while let Ok(msg) = rx.try_recv() {
                latest = Some(msg);
            }
            latest
        }
        None => None,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (tx, rx) = start_analyzer_thread();
    tauri::Builder::default()
        .manage(Mutex::new(ServerState {
            analyzer_tx: Some(tx),
            analyzer_rx: Some(rx),
            ..Default::default()
        }))
        .setup(|_app| Ok(()))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_board,
            try_move,
            get_quote,
            reset_board,
            load_fen,
            start_game,
            update_gameloop,
            promote_pawn,
            fetch_game_history,
            game_into_db,
            fetch_game,
            do_move,
            undo_move,
            poll_analyzer,
            set_analyzer_fen,
            get_fen,
            stop_analyzer,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
