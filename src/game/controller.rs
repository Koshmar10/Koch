use std::sync::{Arc, Mutex};

use stockfish::Stockfish;

use crate::{engine::PieceColor, game::stockfish_engine::{StockfishCmd, StockfishResult}};
#[derive(Clone)]
pub enum GameMode { PvP, PvE, Sandbox}
#[derive(Clone)]
pub enum TerminationBy {Checkmate, StaleMate, Draw, Timeout} 
pub struct GameController {
    pub mode:GameMode,
    pub player: PieceColor,
    pub enemey: PieceColor,
    pub game_over: bool,
    pub lost_by: Option<TerminationBy>,
    pub stockfish: Option<Arc<Mutex<Stockfish>>>,
    pub stockfish_rx: Option<std::sync::mpsc::Receiver<StockfishResult>>,
    pub stockfish_tx: Option<std::sync::mpsc::Sender<StockfishCmd>>,
    pub stockfish_move_pending: bool,
    pub search_depth: usize,
    pub saving_game: bool,
   
}

impl Default for GameController {
    fn default() -> Self {
        Self {
            mode: GameMode::PvE,
            player: PieceColor::White,
            enemey: PieceColor::Black,
            game_over: true,
            stockfish: None,
            stockfish_move_pending: false,
            stockfish_rx:None,
            stockfish_tx:None,
            search_depth:20,
            lost_by: None,
            saving_game: false
            
        }
    }
}


