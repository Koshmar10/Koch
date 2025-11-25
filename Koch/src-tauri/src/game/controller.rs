use rand::seq::IndexedRandom;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use stockfish::Stockfish;
use ts_rs::TS;

use crate::engine::PieceColor;

#[derive(Clone, TS, Serialize)]
#[ts(export)]
pub enum TerminationBy {
    Checkmate,
    StaleMate,
    Draw,
    Timeout,
}
#[derive(Clone, TS, Serialize)]
#[ts(export)]
pub struct GameController {
    pub player: PieceColor,
    pub enemy: PieceColor,
    pub game_over: bool,
    pub lost_by: Option<TerminationBy>,
    pub in_check: bool,
}

impl Default for GameController {
    fn default() -> Self {
        Self {
            player: PieceColor::White,
            enemy: PieceColor::Black,
            game_over: false,
            lost_by: None,
            in_check: false,
        }
    }
}
impl GameController {
    pub fn new() -> Self {
        let mut rng = rand::rng();
        let colors = vec![PieceColor::White, PieceColor::Black];
        let player_color = *colors.choose(&mut rng).unwrap();
        let enemy_color = match player_color {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        };
        return GameController {
            player: player_color,
            enemy: enemy_color,
            game_over: false,
            lost_by: None,
            in_check: false,
        };
    }
}
