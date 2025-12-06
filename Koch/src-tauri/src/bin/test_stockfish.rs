use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use stockfish::Stockfish;

// --- Data Structures ---

#[derive(Clone, Debug, Default)]
struct PvLine {
    moves: String,
    score_type: String, // "cp" (centipawns) or "mate"
    score_value: i32,
}

#[derive(Clone, Debug, Default)]
struct PvObject {
    fen: String,
    depth: u32,
    // MultiPV Index -> Line Data
    lines: std::collections::HashMap<u8, PvLine>,
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
    let (cmd_tx, cmd_rx): (Sender<EngineCommand>, Receiver<EngineCommand>) = mpsc::channel();
    let (update_tx, update_rx): (Sender<PvObject>, Receiver<PvObject>) = mpsc::channel();

    // 2. Spawn the Engine Thread
    thread::spawn(move || {
        // DEBUG: Print start attempt
        println!("THREAD: Attempting to start Stockfish...");

        // CHANGE THIS PATH if your stockfish is located elsewhere (e.g., "stockfish" if in PATH)
        let mut engine = Stockfish::new("/usr/bin/stockfish").unwrap_or_else(|e| {
            panic!("THREAD PANIC: Could not start Stockfish: {}", e);
        });

        println!("THREAD: Stockfish started successfully.");

        engine.set_option("Threads", "6").unwrap();
        engine.set_option("Hash", "1024").unwrap();
        engine.set_option("MultiPV", "3").unwrap();

        let mut is_searching = false;
        let mut current_fen = String::new();
        let mut current_pv = PvObject::default();

        loop {
            // A. Check for Incoming Commands
            match cmd_rx.try_recv() {
                Ok(command) => match command {
                    EngineCommand::SetFen(fen) => {
                        engine.set_fen_position(&fen).unwrap();
                        current_fen = fen.clone();
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
                        }
                    }
                    EngineCommand::Quit => break,
                },
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => break,
            }

            // B. Handle Engine Output
            if is_searching {
                // read_line blocks, so we use a small trick or just rely on it being fast
                // stockfish crate read_line usually blocks until newline.
                let line = engine.read_line();

                if line.starts_with("bestmove") {
                    is_searching = false;
                    continue;
                }

                // Parse PV lines with Score
                if line.contains(" pv ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    println!("{:?}", &parts);
                    let mut multipv_idx = 1;
                    let mut depth = 0;
                    let mut moves = String::new();
                    let mut score_type = String::from("cp");
                    let mut score_value = 0;

                    for i in 0..parts.len() {
                        if parts[i] == "multipv" {
                            multipv_idx = parts[i + 1].parse().unwrap_or(1);
                        }
                        if parts[i] == "depth" {
                            depth = parts[i + 1].parse().unwrap_or(0);
                        }
                        if parts[i] == "score" {
                            if i + 2 < parts.len() {
                                score_type = parts[i + 1].to_string();
                                score_value = parts[i + 2].parse().unwrap_or(0);
                            }
                        }
                        if parts[i] == "pv" {
                            moves = parts[i + 1..].join(" ");
                            break;
                        }
                    }

                    if depth >= current_pv.depth {
                        current_pv.depth = depth;
                        current_pv.lines.insert(
                            multipv_idx,
                            PvLine {
                                moves,
                                score_type,
                                score_value,
                            },
                        );
                        let _ = update_tx.send(current_pv.clone());
                    }
                }
            } else {
                thread::sleep(Duration::from_millis(10));
            }
        }
    });

    // 3. UI / Controller Logic
    println!("--- UI: Starting Analysis ---");

    // Give thread a moment to spawn and potentially fail
    thread::sleep(Duration::from_millis(100));

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    // Check if channel is still open before sending
    if let Err(_) = cmd_tx.send(EngineCommand::SetFen(fen.to_string())) {
        println!("UI Error: Engine thread is dead (SetFen failed).");
        return;
    }
    if let Err(_) = cmd_tx.send(EngineCommand::GoInfinite) {
        println!("UI Error: Engine thread is dead (GoInfinite failed).");
        return;
    }

    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        // Increased to 5s
        match update_rx.try_recv() {
            Ok(pv) => {
                let best_score = if let Some(line) = pv.lines.get(&1) {
                    format!("{} {}", line.score_type, line.score_value)
                } else {
                    "N/A".to_string()
                };

                // print!(
                //     "\rUI Update -> Depth: {} | Lines: {} | Best Score: {:<10}",
                //     pv.depth,
                //     pv.lines.len(),
                //     best_score
                // );
                // // Flush stdout to ensure \r works
                // use std::io::Write;
                // std::io::stdout().flush().unwrap();
            }
            Err(mpsc::TryRecvError::Empty) => {
                // No message yet, wait a bit
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                println!("\nUI Error: Engine thread disconnected unexpectedly!");
                break;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }

    println!("\n\n--- UI: Quitting ---");
    cmd_tx.send(EngineCommand::Quit).unwrap();
}
