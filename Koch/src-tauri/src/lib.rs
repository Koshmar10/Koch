// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod engine;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::Instant;
use sysinfo::System;
//pub mod game;
pub mod analyzer;
pub mod database;
pub mod etc;
pub mod game;
pub mod server;
use crate::analyzer::analyzer::{
    get_analyzer_settings, set_analyzer_fen, set_engine_option, start_analyzer_thread,
    stop_analyzer,
};
use crate::analyzer::analyzer::{try_analyzer_move, EngineCommand};
use crate::analyzer::board_interactions::{get_board_at_index, get_fen, AnalyzerController};
use crate::database::create::{get_game_by_id, get_game_list};
use crate::engine::board::{BoardMetaData, EvalResponse, GameResult};
use crate::engine::serializer::serialize_board;
use crate::engine::serializer::SerializedBoard;
use crate::server::server::{get_system_information, load_settings};
// Added PvLineData
use crate::{
    engine::board,
    engine::{board::PieceMoves, Board, PieceColor, PieceType},
    game::controller::{GameController, TerminationBy},
    server::server::ServerState,
};
use std::collections::BTreeMap;
use std::time::Duration;
use stockfish::EngineOutput;
use stockfish::Stockfish;
use tauri::{AppHandle, Builder, Emitter, Manager, Window};

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
fn update_gameloop(
    state: tauri::State<'_, Mutex<ServerState>>,
) -> (SerializedBoard, Option<GameController>) {
    let mut state = state.lock().unwrap();
    let mut game = state.game_controller.clone();
    let game_over = game.as_ref().unwrap().game_over;
    match &mut game {
        Some(game) => {
            if state.board.is_checkmate() {
                game.lost_by = Some(TerminationBy::Checkmate);
                state.board.meta_data.result = match &state.board.turn {
                    PieceColor::Black => board::GameResult::WhiteWin,
                    PieceColor::White => board::GameResult::BlackWin,
                };
                state.board.meta_data.termination = board::TerminationBy::Checkmate;
                game.game_over = true;
            }
            if state.board.is_stalemate() {
                game.lost_by = Some(TerminationBy::StaleMate);
                state.board.meta_data.termination = board::TerminationBy::StaleMate;
                state.board.meta_data.result = GameResult::Draw;
                game.game_over = true;
            }
        }
        None => {}
    }
    if game_over {
        return (serialize_board(&state.board), game);
    }
    // If no game controller, just return current board
    if state.game_controller.is_none() {
        return (serialize_board(&state.board), game);
    }

    let player_color = state.game_controller.as_ref().unwrap().player;
    let engine_color = state.game_controller.as_ref().unwrap().enemy;

    // If it's still the human player's turn, do nothing

    if state.board.turn == player_color {
        let in_check = state.board.is_in_check(player_color);
        if let Some(g) = game.as_mut() {
            g.in_check = in_check;
        }

        return (serialize_board(&state.board), game);
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

    return (serialize_board(&state.board), game);
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
    promotion: Option<PieceType>,
) -> Option<SerializedBoard> {
    let mut state = state.lock().unwrap();

    // Ensure move cache exists (prevents missing entries for sliding pieces)
    if state.board.move_cache.is_empty() {
        state.board.rerender_move_cache();
    }

    // Capture target BEFORE moving (after move the destination holds the moving piece)
    let captured_before =
        state.board.squares[dest_square.0 as usize][dest_square.1 as usize].clone();

    match state.board.move_piece(src_square, dest_square) {
        Ok(mut move_struct) => {
            if move_struct.is_capture {
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
            if promotion.is_some() {
                state.board.promote_pawn(dest_square, promotion.unwrap());
                move_struct.promotion = promotion;
                move_struct.uci = state
                    .board
                    .encode_uci_move(src_square, dest_square, promotion)
            }
            state.board.meta_data.move_list.push(move_struct);
            // Refresh move cache after a successful move
            state.board.rerender_move_cache();
            match &state.opening_index {
                Some(op_idx) => {
                    let fen = &state.board.to_string();
                    println!("{:?}", &fen);
                    match op_idx.get(fen) {
                        None => {}
                        Some(op) => {
                            state.board.meta_data.opening = Some(op.name.to_string());
                            println!("{:?}", &state.board.meta_data.opening)
                        }
                    }
                }
                None => {}
            }
            let sb = serialize_board(&state.board);
            dbg!(&sb);
            Some(sb)
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
#[derive(Default)]
struct MyState {
    s: std::sync::Mutex<String>,
    t: std::sync::Mutex<std::collections::HashMap<String, String>>,
}
// remember to call `.manage(MyState::default())`
#[tauri::command]
fn board_fen(board: Board) -> String {
    return board.to_string();
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
fn fetch_default_game(state: tauri::State<'_, Mutex<ServerState>>) -> AnalyzerController {
    let mut state = state.lock().unwrap();
    let mut analyzer = AnalyzerController::default();
    analyzer.board.rerender_move_cache();
    state.analyzer_controller = analyzer.clone();
    analyzer
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
            analyzer.current_ply = -1;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Initialize state with default (None channels)
        .manage(Mutex::new(ServerState::default()))
        .setup(|app| {
            let mut sys = System::new_all();
            sys.refresh_all();
            let mem_bytes = sys.total_memory();
            let nb_cpu = sys.cpus().len();
            let ram_capacity = (mem_bytes / 1024 / 1024) as f64 / 1024.0;
            // Get AppHandle
            let handle = app.handle().clone();
            
            // Start thread with handle
            let (tx, rx) = start_analyzer_thread(handle);

            // Update state with channels
            let state = app.state::<Mutex<ServerState>>();
            let mut state = state.lock().unwrap();
            state.analyzer_tx = Some(tx);
            state.analyzer_rx = Some(rx);
            state.total_memory = ram_capacity;
            state.nbcpu = nb_cpu;
           
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_board,
            try_move,
            try_analyzer_move,
            get_quote,
            reset_board,
            load_fen,
            start_game,
            update_gameloop,
            promote_pawn,
            fetch_game_history,
            game_into_db,
            fetch_game,
            fetch_default_game,
            set_analyzer_fen,
            get_fen,
            stop_analyzer,
            board_fen,
            get_board_at_index,
            get_system_information,
            set_engine_option,
            get_analyzer_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
