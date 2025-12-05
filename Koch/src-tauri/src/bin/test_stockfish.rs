/*
use koch_lib::engine;
use serde_json::de;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use stockfish::Stockfish;
use tauri::UserAttentionType;

fn main() {
    let start_time = Instant::now();
    let mut depth_variatons: HashMap<usize, Vec<Vec<String>>> = HashMap::new();
    let mut engine = match Stockfish::new("/usr/bin/stockfish") {
        Ok(mut s) => {
            if s.setup_for_new_game().is_ok() && s.set_skill_level(20).is_ok() {
                s.set_depth(40);
                s.set_option("Threads", "8").unwrap();
                s.set_option("Hash", "1024").unwrap();
                s.set_option("MultiPV", "2").unwrap();
                s.set_fen_position("rn1qkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 3")
                    .unwrap();
                Some(s)
            } else {
                None
            }
        }
        Err(_) => None,
    };
    #[derive(Debug)]
    struct PvObject {
        fen: String,
        depth: u32,
        lines: Vec<Vec<String>>,
    }

    match &mut engine {
        Some(engine) => {
            let mut lines: [(Vec<String>, i32); 3] = std::array::from_fn(|_| (Vec::new(), 0));
            let depths = [5, 10, 15];
            let mut k = 0;
            /*
            for i in 0..5 {
                for pv in &mut pv_vec {
                    engine.set_depth(pv.depth);
                    let out = engine.go().unwrap();
                    pv.line.push(out.best_move().clone());

                    engine.play_move(&out.best_move());
                    pv.fen = engine.get_fen().unwrap();
                    pv.print_line();

                }
            }
            for pv in &mut pv_vec {
                engine.set_depth(25);
                let out = engine.go_for(;
                pv.final_eval = Some(out.eval());
                pv.print_line();
            }

            */
            let mut calculation_time = 1000;

            while calculation_time <= 30000 {
                engine.uci_send("go");
                let mut pv_obj = PvObject {
                    fen: engine.get_fen().unwrap(),
                    depth: 0,
                    lines: Vec::new(),
                };
                std::thread::sleep(Duration::from_millis(calculation_time));
                engine.uci_send("stop");
                loop {
                    let line = engine.read_line();
                    let mut segments = line.split(" ");

                    let first_segment = segments
                        .next()
                        .expect("should be able to get first segment");
                    if first_segment == "bestmove" {
                        break;
                    }
                    let segments_vec = segments.collect::<Vec<&str>>();
                    let mut depth: u32 = 0;
                    for (i, seg) in segments_vec.iter().enumerate() {
                        if seg == &"depth" {
                            depth = segments_vec[i + 1].parse::<u32>().unwrap();
                        }
                        if seg == &"pv" {
                            if depth < pv_obj.depth {
                                continue;
                            } else if depth == pv_obj.depth {
                                let mut pv: Vec<String> = Vec::new();
                                for j in i + 1..segments_vec.len() {
                                    pv.push(segments_vec[j].to_string());
                                }
                                pv_obj.lines.push(pv);
                            } else {
                                let mut pv: Vec<String> = Vec::new();
                                for j in i + 1..segments_vec.len() {
                                    pv.push(segments_vec[j].to_string());
                                }
                                pv_obj.lines = vec![pv];
                                pv_obj.depth = depth;
                            }
                        }
                    }
                }
                println!("{} : {:?}\n", pv_obj.depth, pv_obj.lines);
                calculation_time += 1000;
            }
            /*
            pv.line.push(out.best_move().clone());

            engine.play_move(&out.best_move());
            pv.fen = engine.get_fen().unwrap();
            pv.print_line();
            */
        }
        None => {
            println!("Engine setup failed");
        }
    }

    let duration = start_time.elapsed();
    println!("Total execution time: {:?}", duration);
}
*/
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use stockfish::Stockfish;

// --- Data Structures ---

#[derive(Clone, Debug, Default)]
struct PvObject {
    fen: String,
    depth: u32,
    // MultiPV Index -> Move String
    lines: std::collections::HashMap<u8, String>,
}

// Commands the UI can send to the Engine
enum EngineCommand {
    SetFen(String),
    GoInfinite,
    Stop,
    Quit,
}

// --- Main Logic ---

fn main() {
    // 1. Create Channels
    // cmd: Main -> Engine
    let (cmd_tx, cmd_rx): (Sender<EngineCommand>, Receiver<EngineCommand>) = mpsc::channel();
    // update: Engine -> Main
    let (update_tx, update_rx): (Sender<PvObject>, Receiver<PvObject>) = mpsc::channel();

    // 2. Spawn the Engine Thread
    thread::spawn(move || {
        let mut engine = Stockfish::new("/usr/bin/stockfish").expect("Failed to start engine");

        // Setup Options
        engine.set_option("Threads", "6").unwrap();
        engine.set_option("Hash", "1024").unwrap();
        engine.set_option("MultiPV", "3").unwrap();

        // State tracking
        let mut is_searching = false;
        let mut current_fen = String::new();
        let mut current_pv = PvObject::default();

        loop {
            // A. Check for Incoming Commands (Non-blocking)
            // We use try_recv() so we don't get stuck waiting for commands if the engine needs to output data
            match cmd_rx.try_recv() {
                Ok(command) => match command {
                    EngineCommand::SetFen(fen) => {
                        // If searching, stop first (omitted for brevity, usually good practice)
                        engine.set_fen_position(&fen).unwrap();
                        current_fen = fen.clone();
                        // Reset PV object for new position
                        current_pv = PvObject {
                            fen: fen,
                            depth: 0,
                            lines: std::collections::HashMap::new(),
                        };
                    }
                    EngineCommand::GoInfinite => {
                        engine.uci_send("go infinite").unwrap();
                        is_searching = true;
                    }
                    EngineCommand::Stop => {
                        if is_searching {
                            engine.uci_send("stop").unwrap();
                            // We remain "is_searching = true" until we see "bestmove" line below
                        }
                    }
                    EngineCommand::Quit => break,
                },
                Err(mpsc::TryRecvError::Empty) => {} // No commands, continue
                Err(mpsc::TryRecvError::Disconnected) => break, // Channel closed
            }

            // B. Handle Engine Output
            if is_searching {
                // If we are searching, we expect output.
                // NOTE: read_line() is blocking. In a high-performance GUI,
                // you might run this on a separate inner thread, but for "go infinite",
                // Stockfish outputs often enough that it won't block commands for long.
                let line = engine.read_line();
                // 1. Check if search finished
                if line.starts_with("bestmove") {
                    is_searching = false;
                    continue;
                }

                // 2. Parse PV lines
                if line.contains("multipv") && line.contains(" pv ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    let mut multipv_idx = 0;
                    let mut depth = 0;
                    let mut moves = String::new();

                    for i in 0..parts.len() {
                        if parts[i] == "multipv" {
                            multipv_idx = parts[i + 1].parse().unwrap_or(1);
                        }
                        if parts[i] == "depth" {
                            depth = parts[i + 1].parse().unwrap_or(0);
                        }
                        if parts[i] == "pv" {
                            moves = parts[i + 1..].join(" ");
                            break;
                        }
                    }

                    if depth >= current_pv.depth {
                        current_pv.depth = depth;
                        current_pv.lines.insert(multipv_idx, moves);
                        // Send update to Main Thread
                        let _ = update_tx.send(current_pv.clone());
                    }
                }
            } else {
                // If idle, sleep briefly to save CPU
                thread::sleep(Duration::from_millis(10));
            }
        }
    });

    // 3. UI / Controller Logic
    // Simulate a user interaction flow
    println!("--- UI: Starting Analysis ---");
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    cmd_tx.send(EngineCommand::SetFen(fen.to_string())).unwrap();
    cmd_tx.send(EngineCommand::GoInfinite).unwrap();

    // Poll for updates for 2 seconds
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        // Non-blocking check for updates
        if let Ok(pv) = update_rx.try_recv() {
            // In a real UI, you would update your state here
            // Printing only depth to reduce spam in console
            print!(
                "\rUI Update -> Depth: {} | Lines found: {}",
                pv.depth,
                pv.lines.len()
            );
        }
        thread::sleep(Duration::from_millis(50));
    }

    println!("\n\n--- UI: Stopping and Changing Position ---");
    cmd_tx.send(EngineCommand::Stop).unwrap();

    // Wait a bit for engine to stop
    thread::sleep(Duration::from_secs(1));

    // Start new position
    let new_fen = "r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3";
    cmd_tx
        .send(EngineCommand::SetFen(new_fen.to_string()))
        .unwrap();
    cmd_tx.send(EngineCommand::GoInfinite).unwrap();

    // Listen again
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        if let Ok(pv) = update_rx.try_recv() {
            print!(
                "\rUI Update -> Depth: {} | Lines found: {}",
                pv.depth,
                pv.lines.len()
            );
        }
        thread::sleep(Duration::from_millis(50));
    }

    println!("\n\n--- UI: Quitting ---");
    cmd_tx.send(EngineCommand::Quit).unwrap();
}
