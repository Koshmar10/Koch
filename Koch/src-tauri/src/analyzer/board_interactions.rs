use crate::{
    engine::{
        serializer::{
            serialize_analyzer_controller, SerializedAnalyzerController, SerializedBoard,
        },
        Board, ChessPiece, PieceColor, PieceType,
    },
    server::server::ServerState,
};
use serde::Serialize;
use std::{char, error::Error, sync::Mutex};
use ts_rs::TS;

pub enum AnalyzerWindowOprion {
    Emtpy,
    HeatMap,
}

// New struct to store irreversible FEN state
#[derive(Clone, TS, Serialize, Debug)]
#[ts(export)]
pub struct BoardState {
    pub turn: PieceColor,
    pub white_big_castle: bool,
    pub black_big_castle: bool,
    pub white_small_castle: bool,
    pub black_small_castle: bool,
    pub en_passant_target: Option<(u8, u8)>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

#[derive(Clone, TS, Serialize)]
#[ts(export)]
pub struct AnalyzerController {
    pub game_id: usize,
    pub board: Board,
    pub current_ply: i32,
}

impl Default for AnalyzerController {
    fn default() -> Self {
        Self {
            game_id: 0,
            board: Board::default(),
            current_ply: 0,
        }
    }
}

impl AnalyzerController {
    pub fn get_fen(&self) -> String {
        return self.board.to_string();
    }
}

#[tauri::command]
pub fn get_fen(state: tauri::State<'_, Mutex<ServerState>>) -> String {
    let state = state.lock().unwrap();
    let fen = state.analyzer_controller.get_fen();
    return fen;
}

#[tauri::command]
pub fn get_board_at_index(
    state: tauri::State<'_, Mutex<ServerState>>,
    move_index: isize,
) -> Option<SerializedAnalyzerController> {
    let state = state.lock().unwrap();

    let mut starting_board =
        Board::from(&state.analyzer_controller.board.meta_data.starting_position);

    let game_moves = &state.analyzer_controller.board.meta_data.move_list;

    if move_index == -1 {
        starting_board.meta_data = state.analyzer_controller.board.meta_data.clone();
        let anal = AnalyzerController {
            game_id: state.analyzer_controller.game_id,
            board: starting_board,
            current_ply: -1,
        };
        println!("FEN at index -1: {}", anal.get_fen());
        let serialized_anal = serialize_analyzer_controller(&anal);
        return Some(serialized_anal);
    }

    if move_index < -1 || (move_index as usize) >= game_moves.len() {
        return None;
    }

    for i in 0..=(move_index as usize) {
        let current_move = &game_moves[i];
        if let Some((from, to)) = &starting_board.decode_uci_move(current_move.uci.clone()) {
            match starting_board.move_piece(*from, *to) {
                Ok(_) => {}
                Err(_) => {
                    return None;
                }
            }
        } else {
            return None;
        }
    }

    starting_board.meta_data = state.analyzer_controller.board.meta_data.clone();
    let anal = AnalyzerController {
        game_id: state.analyzer_controller.game_id,
        board: starting_board,
        current_ply: move_index as i32,
    };
    println!("FEN at index {}: {}", move_index, anal.get_fen());
    let serialized_anal = serialize_analyzer_controller(&anal);
    Some(serialized_anal)
}
