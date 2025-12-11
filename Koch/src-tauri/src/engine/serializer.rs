use serde::{Deserialize, Serialize};
use ts_rs::TS;

// Ensure TerminationBy and GameResult are imported so they can be used in the serialized struct
use crate::analyzer::board_interactions::AnalyzerController;
use crate::engine::board::{GameResult, TerminationBy};
use crate::engine::{Board, PieceColor};

// 1. Define the compressed metadata structure
#[derive(Debug, TS, Serialize, Deserialize)]
#[ts(export)]
pub struct SerializedBoardMetaData {
    pub starting_position: String,
    pub date: String,
    pub move_list: String, // Compressed: "e2e4 e7e5 ..."
    pub termination: TerminationBy,
    pub result: GameResult,
    pub white_player_elo: u32,
    pub black_player_elo: u32,
    pub white_player_name: String,
    pub black_player_name: String,
    pub opening: Option<String>,
}

#[derive(Debug, TS, Serialize, Deserialize)]
#[ts(export)]
pub struct SerializedBoard {
    pub fen: String,
    pub piece_moves: String,
    pub piece_index_mapper: String,
    pub meta_data: SerializedBoardMetaData, // Updated type
}

#[derive(Debug, TS, Serialize, Deserialize)]
#[ts(export)]
pub struct SerializedAnalyzerController {
    pub game_id: usize,
    pub serialized_board: SerializedBoard,
    pub current_ply: i32,
}

pub fn serialize_board(board: &Board) -> SerializedBoard {
    let fen = board.to_string();

    // ...existing piece_index_mapper logic...
    let mut piece_index_mapper = String::new();
    for (row_idx, row) in board.squares.iter().enumerate() {
        for (col_idx, square) in row.iter().enumerate() {
            if let Some(sq) = square {
                let square_idx = format!("{row_idx}{col_idx}");
                piece_index_mapper.push_str(&format!("{}:{}|", square_idx, sq.id));
            }
        }
    }
    if piece_index_mapper.ends_with('|') {
        piece_index_mapper.pop();
    }

    // ...existing piece_moves logic...
    let mut piece_moves = String::new();
    for (id, moves) in &board.move_cache {
        piece_moves.push_str(&format!("{}:", id));
        piece_moves.push_str("Q");
        for qm in &moves.quiet_moves {
            piece_moves.push_str(&format!("{}{}", qm.0, qm.1));
        }

        piece_moves.push_str("C");
        for cm in &moves.capture_moves {
            piece_moves.push_str(&format!("{}{}", cm.0, cm.1));
        }

        piece_moves.push_str("A");
        for am in &moves.attacks {
            piece_moves.push_str(&format!("{}{}", am.0, am.1));
        }
        piece_moves.push_str("|");
    }
    if piece_moves.ends_with('|') {
        piece_moves.pop();
    }

    // 2. Compress the move list into a space-separated string
    let move_list_str = board
        .meta_data
        .move_list
        .iter()
        .map(|m| m.uci.clone())
        .collect::<Vec<String>>()
        .join(" ");

    // 3. Construct the serialized metadata
    let serialized_meta = SerializedBoardMetaData {
        starting_position: board.meta_data.starting_position.clone(),
        date: board.meta_data.date.clone(),
        move_list: move_list_str,
        termination: board.meta_data.termination.clone(),
        result: board.meta_data.result.clone(),
        white_player_elo: board.meta_data.white_player_elo,
        black_player_elo: board.meta_data.black_player_elo,
        white_player_name: board.meta_data.white_player_name.clone(),
        black_player_name: board.meta_data.black_player_name.clone(),
        opening: board.meta_data.opening.clone(),
    };

    SerializedBoard {
        fen: fen,
        piece_moves: piece_moves,
        piece_index_mapper: piece_index_mapper,
        meta_data: serialized_meta,
    }
}

pub fn serialize_analyzer_controller(
    controller: &AnalyzerController,
) -> SerializedAnalyzerController {
    SerializedAnalyzerController {
        game_id: controller.game_id,
        serialized_board: serialize_board(&controller.board),
        current_ply: controller.current_ply as i32,
    }
}
