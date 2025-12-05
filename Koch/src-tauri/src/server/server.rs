use crate::analyzer::board_interactions::AnalyzerController;
use crate::{database, engine::Board, game::controller::GameController};
use serde::Serialize;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;
use stockfish::Stockfish;
use ts_rs::TS;
#[derive(Clone, Debug, Default, TS, Serialize)]
#[ts(export)]
pub struct PvObject {
    pub fen: String,
    pub depth: u32,
    // MultiPV Index -> Move String
    pub lines: std::collections::HashMap<u8, String>,
}

// Commands the UI can send to the Engine
#[derive(Debug)]
pub enum EngineCommand {
    SetFen(String),
    GoInfinite,
    Stop,
    Quit,
}
pub struct ServerState {
    pub board: Board,
    pub engine: Option<Stockfish>,
    pub game_controller: Option<GameController>,
    pub analyzer_controller: AnalyzerController,
    pub analyzer_tx: Option<Sender<EngineCommand>>,
    pub analyzer_rx: Option<Receiver<PvObject>>,
}
impl Default for ServerState {
    fn default() -> Self {
        let mut board = Board::default();
        board.rerender_move_cache();
        let engine = match Stockfish::new("/usr/bin/stockfish") {
            Ok(mut s) => {
                if s.setup_for_new_game().is_ok() && s.set_skill_level(16).is_ok() {
                    Some(s)
                } else {
                    None
                }
            }
            Err(_) => None,
        };
        let game_controller = None;
        let analyzer_controller = AnalyzerController::default();

        database::create::create_database()
            .inspect_err(|e| eprintln!("{e}"))
            .ok();
        return ServerState {
            board,
            engine,
            game_controller,
            analyzer_controller,
            analyzer_rx: None,
            analyzer_tx: None,
        };
    }
}
