use std::{sync::{ mpsc::{self, Receiver, Sender}, Arc, Mutex}, thread};

use stockfish::Stockfish;

use crate::ui::app::MyApp;

pub enum StockfishCmd {
    NewGame,
    GoDepth(usize),
    Go(String), // pass a FEN (prefer Board::fen() if available)
    Stop,
    Eval,
}
pub enum StockfishResult {
    Succes,
    Fail,
    Move(String),
}

pub enum StockfishOutput { InvalidOutput, ValidOutput(String)}
impl MyApp{

    pub fn start_stockfish(&mut self){
        self.game.stockfish = match Stockfish::new("/usr/bin/stockfish") {
            Ok(mut s) => {
                if s.setup_for_new_game().is_ok() && s.set_skill_level(16).is_ok() {
                    Some(Arc::new(Mutex::new(s)))
                } else {
                    None
                }
            }
            Err(_) => { None }
        };

        let stockfish: Arc<Mutex<Stockfish>> = match &self.game.stockfish {
            Some(s) => Arc::clone(s),
            None => return,
        };

        let search_dept = self.game.search_depth as u32;

        let (cmd_tx, cmd_rx): (Sender<StockfishCmd>, Receiver<StockfishCmd>) = mpsc::channel();
        let (res_tx, res_rx): (Sender<StockfishResult>, Receiver<StockfishResult>) = mpsc::channel();

        self.game.stockfish_rx = Some(res_rx);

        let _ = thread::Builder::new().name("player_stockfish".to_string()).spawn(move || {
            let mut engine = stockfish.lock().unwrap();
            let _ = engine.setup_for_new_game();
            let _ = engine.set_depth(search_dept);

            loop {
                match cmd_rx.recv() {
                    Ok(cmd) => {
                        match cmd {
                            StockfishCmd::NewGame => {
                                if engine.setup_for_new_game().is_ok() {
                                    let _ = res_tx.send(StockfishResult::Succes);
                                } else {
                                    let _ = res_tx.send(StockfishResult::Fail);
                                }
                            }
                            StockfishCmd::GoDepth(depth) => {
                                let _ = engine.set_depth(depth as u32);
                            }
                            StockfishCmd::Go(fen) => {
                                if let Err(e) = engine.set_fen_position(&fen) {
                                    eprintln!("{}:{} set_fen_position error: {}", file!(), line!(), e);
                                    let _ = res_tx.send(StockfishResult::Fail);
                                    continue;
                                }
                                match engine.go() {
                                    Ok(output) => {
                                        let mv = output.best_move().to_string();
                                        let _ = res_tx.send(StockfishResult::Move(mv));
                                    }
                                    Err(e) => {
                                        eprintln!("{}:{} engine.go() error: {}", file!(), line!(), e);
                                        let _ = res_tx.send(StockfishResult::Fail);
                                    }
                                }
                            }
                            StockfishCmd::Stop => {
                                let _ = engine.setup_for_new_game();
                                let _ = res_tx.send(StockfishResult::Succes);
                                break;
                            }
                            StockfishCmd::Eval => {
                                // Set a small time/depth if you use this path
                                match engine.go() {
                                    Ok(_) => { let _ = res_tx.send(StockfishResult::Succes); }
                                    Err(e) => {
                                        eprintln!("{}:{} engine.go() error: {}", file!(), line!(), e);
                                        let _ = res_tx.send(StockfishResult::Fail);
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => break, // channel closed
                }
            }
        });

        self.game.stockfish_tx = Some(cmd_tx);
    }

    // Send one GO command if none is in flight
    pub fn request_stockfish_move(&mut self) {
        if self.game.stockfish_move_pending {
            return;
        }
        // Prefer Board::fen() if you have it; fallback to to_string()
        let fen = self.board.to_string();
        if let Some(tx) = &self.game.stockfish_tx {
            if tx.send(StockfishCmd::Go(fen)).is_ok() {
                self.game.stockfish_move_pending = true;
            }
        }
    }

    // Poll the result without blocking; call once per frame
    pub fn poll_stockfish_move(&mut self) {
        if !self.game.stockfish_move_pending {
            return;
        }
        if let Some(rx) = &self.game.stockfish_rx {
            match rx.try_recv() {
                Ok(StockfishResult::Move(mv)) => {
                    if let Some((from, to)) = self.board.decode_uci_move(mv) {
                        let _ = self.board.move_piece(from, to);
                    }
                    self.game.stockfish_move_pending = false;
                }
                Ok(StockfishResult::Succes) | Ok(StockfishResult::Fail) => {
                    self.game.stockfish_move_pending = false;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => { /* no result yet */ }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.game.stockfish_move_pending = false;
                }
            }
        }
    }
}