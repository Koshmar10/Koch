use crate::{
    engine::{Board, ChessPiece, PieceColor, PieceType},
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
    pub current_ply: u32,
    pub taken_piece_stack: Vec<(u32, ChessPiece)>,
    pub history_stack: Vec<BoardState>, // Added history stack
}

impl Default for AnalyzerController {
    fn default() -> Self {
        Self {
            game_id: 0,
            board: Board::default(),
            current_ply: 0,
            taken_piece_stack: Vec::new(),
            history_stack: Vec::new(),
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
pub fn do_move(state: tauri::State<'_, Mutex<ServerState>>, uci: String) -> AnalyzerController {
    println!("do triggered");
    let mut state = state.lock().unwrap();
    let mut analyzer = state.analyzer_controller.clone();

    // 1. Save current FEN state to history before modifying
    let current_state = BoardState {
        turn: analyzer.board.turn.clone(),
        white_big_castle: analyzer.board.white_big_castle,
        black_big_castle: analyzer.board.black_big_castle,
        white_small_castle: analyzer.board.white_small_castle,
        black_small_castle: analyzer.board.black_small_castle,
        en_passant_target: analyzer.board.en_passant_target,
        halfmove_clock: analyzer.board.halfmove_clock,
        fullmove_number: analyzer.board.fullmove_number,
    };
    analyzer.history_stack.push(current_state);

    // Decode promotion
    let promotion: Option<PieceType> = if uci.len() == 5 {
        match uci.chars().nth(4) {
            Some('q') => Some(PieceType::Queen),
            Some('b') => Some(PieceType::Bishop),
            Some('n') => Some(PieceType::Knight),
            Some('r') => Some(PieceType::Rook),
            _ => None,
        }
    } else {
        None
    };

    let (from, to) = match analyzer.board.decode_uci_move(uci.clone()) {
        Some(x) => x,
        None => {
            eprintln!("bad uci {uci}");
            return analyzer;
        }
    };

    // Capture stack
    let is_capture = analyzer.board.squares[to.0 as usize][to.1 as usize].is_some();
    if let Some(piece) = analyzer.board.squares[to.0 as usize][to.1 as usize] {
        analyzer
            .taken_piece_stack
            .push((analyzer.current_ply, piece));
    }

    let mut piece = match analyzer.board.squares[from.0 as usize][from.1 as usize] {
        Some(p) => p,
        None => {
            eprintln!("no piece on from square");
            return analyzer;
        }
    };

    let is_pawn = piece.kind == PieceType::Pawn;

    if let Some(pro) = promotion {
        piece.kind = pro;
    }

    // Move piece
    analyzer.board.squares[from.0 as usize][from.1 as usize] = None;
    analyzer.board.squares[to.0 as usize][to.1 as usize] = Some(piece);

    // 2. Update FEN metadata manually

    // Flip Turn
    analyzer.board.turn = match analyzer.board.turn {
        PieceColor::White => PieceColor::Black,
        PieceColor::Black => PieceColor::White,
    };

    // Update Fullmove Number (increments after Black moves)
    if analyzer.board.turn == PieceColor::White {
        analyzer.board.fullmove_number += 1;
    }

    // Update Halfmove Clock (reset on pawn move or capture)
    if is_pawn || is_capture {
        analyzer.board.halfmove_clock = 0;
    } else {
        analyzer.board.halfmove_clock += 1;
    }

    // Update En Passant Target
    analyzer.board.en_passant_target = None; // Reset default
    if is_pawn {
        let diff = (from.0 as i8 - to.0 as i8).abs();
        if diff == 2 {
            // Pawn moved 2 squares, set target
            let row = if from.0 < to.0 {
                from.0 + 1
            } else {
                from.0 - 1
            };
            analyzer.board.en_passant_target = Some((row, from.1));
        }
    }

    // Update Castling Rights (Basic logic: if King moves, lose rights)
    if piece.kind == PieceType::King {
        match piece.color {
            PieceColor::White => {
                analyzer.board.white_big_castle = false;
                analyzer.board.white_small_castle = false;
            }
            PieceColor::Black => {
                analyzer.board.black_big_castle = false;
                analyzer.board.black_small_castle = false;
            }
        }
    }
    // Note: Full castling logic (rooks moving/captured) would require more checks,
    // but this covers the most common FEN invalidation cause (king moving).

    analyzer.current_ply += 1;

    // Persist back into state before returning
    state.analyzer_controller = analyzer.clone();
    analyzer
}

#[tauri::command]
pub fn undo_move(state: tauri::State<'_, Mutex<ServerState>>, uci: String) -> AnalyzerController {
    println!("undo triggered");
    let mut state = state.lock().unwrap();
    let mut analyzer = state.analyzer_controller.clone();

    if analyzer.current_ply == 0 {
        // Nothing to undo
        return analyzer;
    }

    let (from, to) = match analyzer.board.decode_uci_move(uci.clone()) {
        Some(x) => x,
        None => {
            eprintln!("bad uci {uci}");
            return analyzer;
        }
    };

    // Piece expected at 'to'
    let mut piece = match analyzer.board.squares[to.0 as usize][to.1 as usize] {
        Some(p) => p,
        None => {
            eprintln!("no piece on destination square to undo");
            return analyzer;
        }
    };

    // Reverse promotion (uci len 5 means promotion originally occurred)
    if uci.len() == 5 {
        piece.kind = PieceType::Pawn;
    }

    // Move back
    analyzer.board.squares[to.0 as usize][to.1 as usize] = None;
    analyzer.board.squares[from.0 as usize][from.1 as usize] = Some(piece);

    // Restore captured piece if stack entry matches previous ply
    if let Some((ply, captured)) = analyzer.taken_piece_stack.last() {
        if *ply == analyzer.current_ply - 1 {
            analyzer.board.squares[to.0 as usize][to.1 as usize] = Some(*captured);
            analyzer.taken_piece_stack.pop();
        }
    }

    // 3. Restore FEN metadata from history
    if let Some(prev_state) = analyzer.history_stack.pop() {
        analyzer.board.turn = prev_state.turn;
        analyzer.board.white_big_castle = prev_state.white_big_castle;
        analyzer.board.black_big_castle = prev_state.black_big_castle;
        analyzer.board.white_small_castle = prev_state.white_small_castle;
        analyzer.board.black_small_castle = prev_state.black_small_castle;
        analyzer.board.en_passant_target = prev_state.en_passant_target;
        analyzer.board.halfmove_clock = prev_state.halfmove_clock;
        analyzer.board.fullmove_number = prev_state.fullmove_number;
    }

    analyzer.current_ply -= 1;

    // Persist back
    state.analyzer_controller = analyzer.clone();
    analyzer
}
