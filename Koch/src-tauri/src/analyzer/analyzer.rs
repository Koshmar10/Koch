use std::{
    collections::HashMap,
    fs::OpenOptions,
    sync::{mpsc, Mutex},
    thread,
    time::{Duration, Instant},
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
    Snapshot, // NEW: ask thread to send current PvObject
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum EngineOption {
    MultiPv,
    Threads,
    Hash,
}
use stockfish::Stockfish;
use tauri::{AppHandle, Emitter};

use crate::{
    analyzer::board_interactions::AnalyzerController,
    engine::{
        serializer::{serialize_analyzer_controller, SerializedAnalyzerController},
        PieceColor, PieceType,
    },
    server::server::{load_settings, EvalKind, PvLineData, PvObject, ServerState},
};

#[tauri::command]
pub fn stop_analyzer(state: tauri::State<'_, Mutex<ServerState>>) -> bool {
    let mut state = state.lock().unwrap();
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
) -> (mpsc::Sender<EngineCommand>, mpsc::Receiver<PvObject>) {
    let (cmd_tx, cmd_rx): (mpsc::Sender<EngineCommand>, mpsc::Receiver<EngineCommand>) =
        mpsc::channel();
    let (update_tx, update_rx): (mpsc::Sender<PvObject>, mpsc::Receiver<PvObject>) =
        mpsc::channel();

    thread::Builder::new()
        .name("analyzer-thread".to_string())
        .spawn(move || {
            let stockfish_go_config = String::from("go depth 40 movetime 3000");
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
                            // Log command with timestamp
                            let ts = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or(Duration::ZERO)
                                .as_secs_f64();
                            let uptime = start_time.elapsed().as_secs_f64();

                            match command {
                                EngineCommand::SetFen(fen) => {
                                    if is_searching {
                                        let _ = engine.uci_send("stop");
                                        loop {
                                            let l = engine.read_line();
                                            if l.starts_with("bestmove") {
                                                break;
                                            }
                                            // Log stop-drain lines too
                                        }
                                        is_searching = false;
                                    }
                                    engine.ensure_ready().ok();
                                    let _ = engine.set_fen_position(&fen);
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
                                    let _ = engine.uci_send(&stockfish_go_config);
                                    is_searching = true;
                                }
                                EngineCommand::Stop => {
                                    if is_searching {
                                        let _ = engine.uci_send("stop");
                                        loop {
                                            let l = engine.read_line();
                                            if l.starts_with("bestmove") {
                                                break;
                                            }
                                        }
                                        is_searching = false;
                                    }
                                }
                                EngineCommand::Quit => break,
                                EngineCommand::Snapshot => {
                                    let _ = app_handle.emit("pv_update", current_pv.clone());
                                    let _ = update_tx.send(current_pv.clone());
                                }
                                EngineCommand::SetAndGo(fen, mult) => {
                                    if is_searching {
                                        let _ = engine.uci_send("stop");
                                        loop {
                                            let l = engine.read_line();
                                            if l.starts_with("bestmove") {
                                                break;
                                            }
                                        }
                                        is_searching = false;
                                    }
                                    engine.ensure_ready().ok();
                                    let _ = engine.uci_send(&fen);
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
                                EngineCommand::SetHashSize(hash) => {
                                    engine.ensure_ready().ok();
                                    match engine.set_option("Hash", &hash.to_string()) {
                                        Ok(_) => {
                                            println!("[Analyzer] Set Hash to {} success", hash)
                                        }
                                        Err(e) => eprintln!(
                                            "[Analyzer] Set Hash to {} failed: {}",
                                            hash, e
                                        ),
                                    }
                                }
                                EngineCommand::SetMultiPv(cnt) => {
                                    engine.ensure_ready().ok();
                                    match engine.set_option("MultiPV", &cnt.to_string()) {
                                        Ok(_) => {
                                            println!("[Analyzer] Set MultiPV to {} success", cnt)
                                        }
                                        Err(e) => eprintln!(
                                            "[Analyzer] Set MultiPV to {} failed: {}",
                                            cnt, e
                                        ),
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
                        Err(mpsc::TryRecvError::Empty) => {}
                        Err(mpsc::TryRecvError::Disconnected) => break,
                    }

                    // Read Stockfish output, log each line
                    if is_searching {
                        if last_clock_tick.elapsed() >= Duration::from_secs(1) {
                            let uptime = start_time.elapsed().as_secs_f64();
                            println!("[Analyzer][clock={uptime:.3}s] Searching...");
                            last_clock_tick = Instant::now();
                        }

                        let line = engine.read_line();
                        if line.is_empty() {
                            continue;
                        }

                        // Append raw engine line to logfile
                        //let _ = writeln!(sf_log, "{line}");

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
        })
        .expect("Failed to spawn analyzer thread");

    (cmd_tx, update_rx)
}

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
        .move_piece(src_square, dest_square)
    {
        Ok(mut mv) => {
            if mv.is_capture {
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
                state
                    .analyzer_controller
                    .board
                    .promote_pawn(dest_square, promotion.unwrap());
                mv.promotion = promotion;
                mv.uci = state.analyzer_controller.board.encode_uci_move(
                    src_square,
                    dest_square,
                    promotion,
                )
            }
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
#[tauri::command]
pub fn set_analyzer_fen(state: tauri::State<'_, Mutex<ServerState>>, current_move: isize) -> bool {
    let state = state.lock().unwrap();
    println!("Called set fen for{}", current_move);

    let start_fen = state.analyzer_controller.get_fen();
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

    println!(
        "{} | side_to_move={} | multiplier={}",
        &fen,
        if white_to_move { "w" } else { "b" },
        multiplier
    );

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
