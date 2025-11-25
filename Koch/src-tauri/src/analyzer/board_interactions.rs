use crate::{
    engine::{Board, ChessPiece, PieceType},
    server::server::ServerState,
};
use serde::Serialize;
use std::{char, error::Error, sync::Mutex};
use ts_rs::TS;
pub enum AnalyzerWindowOprion {
    Emtpy,
    HeatMap,
}

#[derive(Clone, TS, Serialize)]
#[ts(export)]
pub struct AnalyzerController {
    pub game_id: usize,
    pub board: Board,
    pub current_ply: u32,
    pub taken_piece_stack: Vec<(u32, ChessPiece)>,
}
impl Default for AnalyzerController {
    fn default() -> Self {
        Self {
            game_id: 0,
            board: Board::default(),
            current_ply: 0,
            taken_piece_stack: Vec::new(),
        }
    }
}
#[tauri::command]
pub fn do_move(state: tauri::State<'_, Mutex<ServerState>>, uci: String) -> AnalyzerController {
    println!("do triggered");
    let mut state = state.lock().unwrap();
    let mut analyzer = state.analyzer_controller.clone();

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

    if let Some(pro) = promotion {
        piece.kind = pro;
    }

    analyzer.board.squares[from.0 as usize][from.1 as usize] = None;
    analyzer.board.squares[to.0 as usize][to.1 as usize] = Some(piece);
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

    analyzer.current_ply -= 1;

    // Persist back
    state.analyzer_controller = analyzer.clone();
    analyzer
}
