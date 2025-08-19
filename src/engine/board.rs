use std::{cell::Cell, clone, error::Error, path::Display};

use crate::{engine::{fen::fen_parser, move_gen::MoveError, ChessPiece, PieceColor, PieceType}, etc::{DEFAULT_FEN, DEFAULT_STARTING}, game::{controller::TerminationBy, evaluator::EvalResponse}, ui::app::MyApp};
use chrono::Local;
use eframe::egui::CentralPanel;

#[derive(Clone)]
pub struct Board{
    pub squares: [[Option<ChessPiece>; 8]; 8],
    pub turn: PieceColor,
    pub white_big_castle: bool,
    pub black_big_castle: bool,
    pub white_small_castle: bool,
    pub black_small_castle: bool,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub en_passant_target: Option<(u8,u8)>,
    pub ui: BoardUi,
    pub meta_data: BoardMetaData,
    pub move_cache: std::collections::HashMap<u32, PieceMoves>,
    pub been_modified: bool,
    pub next_id: u32,
    pub ply_count: u32,
}

#[derive(Clone)]
pub struct PieceMoves{
    pub quiet_moves: Vec<(u8,u8)>,
    pub capture_moves: Vec<(u8,u8)>,
}
pub struct MoveInfo{
    pub old_pos: (u8, u8),
    pub new_pos: (u8, u8), 
    pub promotion: Option<PieceType>,
    pub is_capture: bool,
}

#[derive(Clone)]
pub struct BoardUi {
    pub selected_piece: Option<ChessPiece>,
    pub moved_piece: Option<(u8,u8)>,
    pub pov: PieceColor,
    pub white_taken:     Vec<((PieceType, PieceColor))>,
    pub black_taken:     Vec<((PieceType, PieceColor))>,
    pub promtion_pending: Option<(u8, u8)>,
    pub checkmate_square: Option<(u8, u8)>,
    pub bar_eval : f32,

}
#[derive(Clone)]
pub enum GameResult {WhiteWin, BlackWin, Draw, Unfinished}

#[derive(Clone)]
pub struct BoardMetaData{
    pub starting_position: String,
    pub date: String,
    pub move_list: Vec<MoveStruct>,
    pub termination: TerminationBy,
    pub result: GameResult,
    pub white_player_elo: u32,
    pub black_player_elo: u32,
    pub white_player_name: String,
    pub black_player_name: String,

}

#[derive(Clone)]
pub struct MoveStruct{
    pub move_number: u32,
    pub san: String,
    pub uci: String,
    pub promotion: Option<PieceType>,
    pub is_capture:bool,
    pub evaluation: EvalResponse,
    pub time_stamp: f32,
}

#[derive(Clone)]
pub enum CastleType {QueenSide, KingSide}
    

impl Default for Board{
    fn default() -> Self {
        match fen_parser(&DEFAULT_FEN.to_owned()) {
            Ok(board) => board,
            Err(_) => Board{
                squares: [[None; 8]; 8],
                turn: PieceColor::White,
                white_big_castle: true,
                white_small_castle: true,
                black_big_castle: true, 
                black_small_castle: true, 
                halfmove_clock: 0,
                fullmove_number: 1,
                en_passant_target: None,
                been_modified: true,
                ui: BoardUi::default(),
                meta_data: BoardMetaData::default(),
                move_cache: std::collections::HashMap::new(),
                next_id: 0,
                ply_count: 0
            }
        }
    }
}
impl Default for BoardUi{
    fn default() -> Self {
        Self{
            selected_piece: None,
            pov: DEFAULT_STARTING, 
            white_taken: Vec::new(),
            black_taken: Vec::new(),
            promtion_pending: None,
            checkmate_square: None,
            bar_eval: 0.0,
            moved_piece: None,
          
        }
    }
}
impl Default for BoardMetaData {
    fn default() -> Self {
        Self{
            starting_position: DEFAULT_FEN.to_string(),
            date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            move_list: Vec::new(),
            termination: TerminationBy::Draw,
            result: GameResult::Unfinished,
            white_player_elo: 0,
            black_player_elo: 0,
            white_player_name: String::new(),
            black_player_name: String::new(),
        }
    }
        }
  
impl Default for MoveStruct {
    fn default() -> Self {
        MoveStruct {
            move_number: 0,
            san: String::new(),
            uci: String::new(),
            promotion: None,
            is_capture: false,
            evaluation: EvalResponse { value: 0.0, kind: stockfish::EvalType::Centipawn },
            time_stamp: 0.0,
        }
    }
}

impl From<&str> for MoveStruct {
    fn from(uci: &str) -> Self {
        let mut mv = MoveStruct::default();
        mv.uci = uci.to_string();
        if let Some(p) = uci.chars().nth(4) {
            mv.promotion = Some(match p.to_ascii_lowercase() {
                'q' => PieceType::Queen,
                'r' => PieceType::Rook,
                'b' => PieceType::Bishop,
                'n' => PieceType::Knight,
                _ => return mv,
            });
        }
        mv
    }
}

impl From<&String> for Board {
    fn from(fen: &String) -> Self {
        match fen_parser(fen) {
            Ok(board) => board,
            Err(_) => Board::default(),
        }
    }
}   
impl Board{
    pub fn move_piece(&mut self, old_pos: (u8, u8), new_pos: (u8, u8)) -> Result<MoveStruct, MoveError> {
        let mut failed_to_move = false;

        let mut moving_piece = match self.squares[old_pos.0 as usize][old_pos.1 as usize].take() {
            Some(piece) => piece,
            None => return Err(MoveError::NoAviailableMoves),
        };
        let replacer = PieceMoves {
                quiet_moves: Vec::new(),
                capture_moves: Vec::new()};
        let moves = self
            .move_cache
            .get(&moving_piece.id)
            .unwrap_or(&replacer);

        let is_capture = moves.capture_moves.contains(&new_pos);
        let is_quiet = moves.quiet_moves.contains(&new_pos);
        let is_pawn_move = moving_piece.kind == PieceType::Pawn;

        if !is_quiet && !is_capture {
            failed_to_move = true;
        } else {
            match moving_piece.kind {
                PieceType::Pawn => {
                    if is_capture {
                        // En passant capture if target square is empty
                        if self.squares[new_pos.0 as usize][new_pos.1 as usize].is_none() {
                            if let Some((x, y)) = self.en_passant_target {
                                self.squares[x as usize][y as usize] = None;
                            }
                        }
                    } else {
                        // If pawn moved two squares, set en passant target
                        if new_pos.0.abs_diff(old_pos.0) > 1 {
                            self.en_passant_target = Some(new_pos);
                        }
                    }
                    if moving_piece.color == PieceColor::White{
                        if new_pos.0 == 0{
                            self.ui.promtion_pending = Some(new_pos);
                        }
                    }
                    else {
                        if new_pos.0 == 7{
                            self.ui.promtion_pending = Some(new_pos);
                        }
                    }
                    moving_piece.position = new_pos;
                    moving_piece.has_moved = true;
                    self.squares[new_pos.0 as usize][new_pos.1 as usize] = Some(moving_piece);
                }
                _ => {
                    moving_piece.position = new_pos;
                    moving_piece.has_moved = true;
                    self.squares[new_pos.0 as usize][new_pos.1 as usize] = Some(moving_piece);
                }
            }
        }

        if !failed_to_move {
            // Update 50-move rule halfmove clock
            if is_capture || is_pawn_move {
                self.halfmove_clock = 0;
            } else {
                self.halfmove_clock = self.halfmove_clock.saturating_add(1);
            }

            // Fullmove number increments after Black's move
            let was_black_to_move = self.turn == PieceColor::Black;

            self.deselect_piece();
            self.change_turn();

            if was_black_to_move {
                self.fullmove_number = self.fullmove_number.saturating_add(1);
            }

            self.been_modified = true;
            Ok(MoveStruct {
                move_number: self.ply_count,
                uci: self.encode_uci_move(old_pos, new_pos, None),
                san: "".to_string(),
                promotion: None,
                is_capture,
                evaluation: EvalResponse { value: 0.0, kind: stockfish::EvalType::Centipawn },
                time_stamp: 0.0,
            })
        } else {
            // Revert piece if move failed
            self.squares[old_pos.0 as usize][old_pos.1 as usize] = Some(moving_piece);
            Err(MoveError::IllegalMove)
        }
    }

    pub fn try_castle(&mut self, king_pos: (u8,u8), rook_pos: (u8,u8)) -> bool{
        if self.can_castle(king_pos, rook_pos) {
            self.execute_castle(king_pos, rook_pos);
        }
        else { return false;}

        return true;
    }
    pub fn can_castle(&self, king_pos: (u8,u8), rook_pos: (u8,u8)) -> bool{
        // Get the king and rook pieces
        if let (Some(king), Some(rook)) = (
            self.squares[king_pos.0 as usize][king_pos.1 as usize].as_ref(),
            self.squares[rook_pos.0 as usize][rook_pos.1 as usize].as_ref()
        ) {
            // Verify pieces are correct type and color
            if king.kind != PieceType::King  {return false;}
            if rook.kind != PieceType::Rook  {return false;}
        } else {
            // Either king or rook position doesn't have a piece
            return false;
        };
        
         let castle_squares: [((u8,u8), (u8,u8)); 2] = match self.turn {
             PieceColor::Black => {
                [((0,4), (0, 0)), ((0,4), (0, 7))]
             },
             PieceColor:: White => {
                [((7,4), (7, 0)), ((7,4), (7, 7))]
             },
         };
        //check that the king and rook are in position
        if !castle_squares.contains(&(king_pos, rook_pos)){return false};
        //we check if the squares between them are not occupied
        
        let castle_type = if king_pos.1.abs_diff(rook_pos.1) == 4 {CastleType::QueenSide} else {CastleType::KingSide};

        match castle_type {
            CastleType::QueenSide => {
                for i in rook_pos.1+1..king_pos.1{
                    if self.squares[king_pos.0 as usize][i as usize].is_some() {
                        return false;
                    }
                    // Check if the king would pass through a square that's under attack
                    // For queenside castling, the king passes through two squares: (king_pos.1-1) and (king_pos.1-2)
                    for i in king_pos.1-2..=king_pos.1 {
                        // Check if this square is a target in any opponent's capture_moves
                        let check_square = (king_pos.0, i);
                        for (_, piece_moves) in self.move_cache.iter() {
                            if piece_moves.capture_moves.contains(&check_square) {
                                return false; // Can't castle through check
                            }
                        }
                    }
                }
            }
            CastleType::KingSide => {
                for i in king_pos.1+1..rook_pos.1{
                    if self.squares[king_pos.0 as usize][i as usize].is_some() {
                        return false;
                    }
                };
                // Check if the king would pass through a square that's under attack
                // For kingside castling, the king passes through two squares: king_pos.1 and (king_pos.1+1)
                for i in king_pos.1..=king_pos.1+1 {
                    // Check if this square is a target in any opponent's capture_moves
                    let check_square = (king_pos.0, i);
                    for (_, piece_moves) in self.move_cache.iter() {
                        if piece_moves.capture_moves.contains(&check_square) {
                            return false; // Can't castle through check
                        }
                    }
                }
            }
        } 
        // we check if the king is in check
        if self.is_in_check(self.turn) {return false;}

        return true;
    }
    pub fn execute_castle(&mut self, king_pos: (u8,u8), rook_pos: (u8,u8)){
        // Determine castle type based on king and rook positions
        let castle_type = if king_pos.1.abs_diff(rook_pos.1) == 4 {
            CastleType::QueenSide
        } else {
            CastleType::KingSide
        };

        // Calculate new positions based on castle type
        let (king_new_col, rook_new_col) = match castle_type {
            CastleType::KingSide => (king_pos.1 + 2, king_pos.1 + 1),
            CastleType::QueenSide => (king_pos.1 - 2, king_pos.1 - 1),
        };

        // Move king
        let king = self.squares[king_pos.0 as usize][king_pos.1 as usize].take();
        self.squares[king_pos.0 as usize][king_new_col as usize] = king;

        // Move rook
        let rook = self.squares[rook_pos.0 as usize][rook_pos.1 as usize].take();
        self.squares[rook_pos.0 as usize][rook_new_col as usize] = rook;

        // Update castling rights
        if self.turn == PieceColor::White {
            self.white_small_castle = false;
            self.white_big_castle = false;
        } else {
            self.black_small_castle = false;
            self.black_big_castle = false;
        }
        self.rerender_move_cache();

    }

    pub fn change_turn(&mut self){
        self.turn = match self.turn {
            PieceColor::Black => PieceColor::White,
            PieceColor::White => PieceColor::Black,
        };
    }
    
    
    pub fn promote_pawn(&mut self, pos: (u8,u8), kind: PieceType){
        if let Some(pawn) = self.squares[pos.0 as usize][pos.1 as usize].as_mut() {
            pawn.kind = kind;
        }
    }
    pub fn select_piece(&mut self, piece: ChessPiece) {
        if self.turn == piece.color {
            self.ui.selected_piece = Some(piece);
        }
        else {
            self.deselect_piece();
        }
    }
    pub fn deselect_piece(&mut self) {
        self.ui.selected_piece = None;
        
    }
    
    
}