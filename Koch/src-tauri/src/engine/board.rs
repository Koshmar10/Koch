use crate::{
    engine::{fen::fen_parser, move_gen::MoveError, ChessPiece, PieceColor, PieceType},
    etc::{DEFAULT_FEN, DEFAULT_STARTING},
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, TS, Serialize, Deserialize, Debug)]
#[ts(export)]
pub enum EvalType {
    Centipawn,
    Mate,
}

#[derive(Clone, TS, Serialize, Deserialize)]
#[ts(export)]
pub enum TerminationBy {
    Checkmate,
    StaleMate,
    Draw,
    Timeout,
}

#[derive(Clone, TS, Serialize, Deserialize, Debug)]
#[ts(export)]
pub struct EvalResponse {
    pub value: f32,
    pub kind: EvalType,
}

#[derive(Clone, TS, Serialize, Deserialize)]
#[ts(export)]
pub struct Board {
    pub squares: [[Option<ChessPiece>; 8]; 8],
    pub turn: PieceColor,
    pub white_big_castle: bool,
    pub black_big_castle: bool,
    pub white_small_castle: bool,
    pub black_small_castle: bool,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub en_passant_target: Option<(u8, u8)>,
    pub ui: BoardUi,
    pub meta_data: BoardMetaData,
    pub move_cache: std::collections::HashMap<u32, PieceMoves>,
    pub been_modified: bool,
    pub next_id: u32,
    pub ply_count: u32,
}

#[derive(Clone, TS, Serialize, Deserialize)]
#[ts(export)]
pub struct PieceMoves {
    pub quiet_moves: Vec<(u8, u8)>,
    pub capture_moves: Vec<(u8, u8)>,
    pub attacks: Vec<(u8, u8)>,
}

#[derive(Clone, TS, Serialize, Deserialize)]
#[ts(export)]
pub struct MoveInfo {
    pub old_pos: (u8, u8),
    pub new_pos: (u8, u8),
    pub promotion: Option<PieceType>,
    pub is_capture: bool,
}

#[derive(Clone, TS, Serialize, Deserialize)]
#[ts(export)]
pub struct BoardUi {
    pub selected_piece: Option<ChessPiece>,
    pub moved_piece: Option<(u8, u8)>,
    pub pov: PieceColor,
    pub white_taken: Vec<(PieceType, PieceColor)>,
    pub black_taken: Vec<(PieceType, PieceColor)>,
    pub promtion_pending: Option<(u8, u8)>,
    pub checkmate_square: Option<(u8, u8)>,
    pub bar_eval: f32,
}

#[derive(Clone, TS, Serialize, Deserialize)]
#[ts(export)]
pub enum GameResult {
    WhiteWin,
    BlackWin,
    Draw,
    Unfinished,
}

#[derive(Clone, TS, Serialize, Deserialize)]
#[ts(export)]
pub struct BoardMetaData {
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

#[derive(Clone, TS, Serialize, Deserialize, Debug)]
#[ts(export)]
pub struct MoveStruct {
    pub move_number: u32,
    pub san: String,
    pub uci: String,
    pub promotion: Option<PieceType>,
    pub is_capture: bool,
    pub evaluation: EvalResponse,
    pub time_stamp: f32,
}

#[derive(Clone, TS, Serialize, Deserialize)]
#[ts(export)]
pub enum CastleType {
    QueenSide,
    KingSide,
}

impl Default for Board {
    fn default() -> Self {
        match fen_parser(&DEFAULT_FEN.to_owned()) {
            Ok(board) => board,
            Err(_) => Board {
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
                ply_count: 0,
            },
        }
    }
}
impl Default for BoardUi {
    fn default() -> Self {
        Self {
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
        Self {
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
            evaluation: EvalResponse {
                value: 0.0,
                kind: EvalType::Centipawn,
            },
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
impl Board {
    pub fn move_piece(
        &mut self,
        old_pos: (u8, u8),
        new_pos: (u8, u8),
    ) -> Result<MoveStruct, MoveError> {
        let moving_piece = match self.squares[old_pos.0 as usize][old_pos.1 as usize] {
            Some(piece) => piece,
            None => return Err(MoveError::NoAviailableMoves),
        };

        if moving_piece.color != self.turn {
            return Err(MoveError::IllegalMove);
        }

        // Refresh move cache to avoid stale legality checks
        self.rerender_move_cache();
        let empty_moves = PieceMoves {
            quiet_moves: vec![],
            capture_moves: vec![],
            attacks: vec![],
        };
        let legal = self
            .move_cache
            .get(&moving_piece.id)
            .unwrap_or(&empty_moves);

        let is_capture = legal.capture_moves.contains(&new_pos);
        let is_quiet = legal.quiet_moves.contains(&new_pos);

        if !is_capture && !is_quiet {
            return Err(MoveError::IllegalMove);
        }

        let mut captured_piece: Option<ChessPiece> = None;
        let previous_en_passant = self.en_passant_target;
        self.en_passant_target = None;

        // Handle captures (including en passant) before removing the moving piece
        if is_capture {
            if moving_piece.kind == PieceType::Pawn
                && self.squares[new_pos.0 as usize][new_pos.1 as usize].is_none()
            {
                // En passant capture; ensure target still valid
                if previous_en_passant != Some(new_pos) {
                    return Err(MoveError::IllegalMove);
                }
                let dir = if moving_piece.color == PieceColor::White {
                    1
                } else {
                    -1
                };
                let captured_r = (new_pos.0 as i8 + dir) as u8;
                captured_piece = self.squares[captured_r as usize][new_pos.1 as usize].take();
            } else {
                captured_piece = self.squares[new_pos.0 as usize][new_pos.1 as usize].take();
            }
        }

        // Now remove the moving piece and update its state
        let mut moving_piece = self.squares[old_pos.0 as usize][old_pos.1 as usize]
            .take()
            .ok_or(MoveError::NoAviailableMoves)?;

        // Promotion trigger
        if (moving_piece.kind == PieceType::Pawn
            && moving_piece.color == PieceColor::White
            && new_pos.0 == 0)
            || (moving_piece.kind == PieceType::Pawn
                && moving_piece.color == PieceColor::Black
                && new_pos.0 == 7)
        {
            self.ui.promtion_pending = Some(new_pos);
        }

        // Set en passant target for double pawn advance
        if moving_piece.kind == PieceType::Pawn && !is_capture {
            if new_pos.0.abs_diff(old_pos.0) == 2 {
                let mid_r = (old_pos.0 + new_pos.0) / 2;
                self.en_passant_target = Some((mid_r, old_pos.1));
            }
        }

        // Update castling rights when king/rook moves or a rook is captured
        match (moving_piece.kind, moving_piece.color) {
            (PieceType::King, PieceColor::White) => {
                self.white_big_castle = false;
                self.white_small_castle = false;
            }
            (PieceType::King, PieceColor::Black) => {
                self.black_big_castle = false;
                self.black_small_castle = false;
            }
            (PieceType::Rook, PieceColor::White) => {
                if old_pos == (7, 0) {
                    self.white_big_castle = false;
                }
                if old_pos == (7, 7) {
                    self.white_small_castle = false;
                }
            }
            (PieceType::Rook, PieceColor::Black) => {
                if old_pos == (0, 0) {
                    self.black_big_castle = false;
                }
                if old_pos == (0, 7) {
                    self.black_small_castle = false;
                }
            }
            _ => {}
        }

        if let Some(captured) = captured_piece {
            if captured.kind == PieceType::Rook {
                match (captured.color, captured.position) {
                    (PieceColor::White, (7, 0)) => self.white_big_castle = false,
                    (PieceColor::White, (7, 7)) => self.white_small_castle = false,
                    (PieceColor::Black, (0, 0)) => self.black_big_castle = false,
                    (PieceColor::Black, (0, 7)) => self.black_small_castle = false,
                    _ => {}
                }
            }
        }

        let is_pawn_move = moving_piece.kind == PieceType::Pawn;

        // Execute move (capture handled by overwriting)
        moving_piece.position = new_pos;
        moving_piece.has_moved = true;
        self.squares[new_pos.0 as usize][new_pos.1 as usize] = Some(moving_piece);

        // 50-move rule clock
        if is_capture || is_pawn_move {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock = self.halfmove_clock.saturating_add(1);
        }

        let was_black = self.turn == PieceColor::Black;
        self.deselect_piece();
        self.change_turn();
        if was_black {
            self.fullmove_number = self.fullmove_number.saturating_add(1);
        }

        self.been_modified = true;

        Ok(MoveStruct {
            move_number: self.ply_count,
            uci: self.encode_uci_move(old_pos, new_pos, None),
            san: String::new(),
            promotion: None,
            is_capture,
            evaluation: EvalResponse {
                value: 0.0,
                kind: EvalType::Centipawn,
            },
            time_stamp: 0.0,
        })
    }

    pub fn try_castle(&mut self, king_pos: (u8, u8), rook_pos: (u8, u8)) -> bool {
        if self.can_castle(king_pos, rook_pos) {
            self.execute_castle(king_pos, rook_pos);
        } else {
            return false;
        }

        return true;
    }
    /// Returns true if `sq` is attacked by any piece of `by_color`
    fn is_square_attacked_by(&self, sq: (u8, u8), by_color: PieceColor) -> bool {
        for rank in 0..8 {
            for file in 0..8 {
                if let Some(p) = self.squares[rank][file] {
                    if p.color != by_color {
                        continue;
                    }
                    match p.kind {
                        PieceType::Pawn => {
                            // White pawns attack up-left/up-right; Black down-left/down-right
                            let (r, c) = (p.position.0 as i8, p.position.1 as i8);
                            let dir = if by_color == PieceColor::White { -1 } else { 1 };
                            let attacks = [(r + dir, c - 1), (r + dir, c + 1)];
                            for (ar, ac) in attacks {
                                if (0..8).contains(&ar) && (0..8).contains(&ac) {
                                    if (ar as u8, ac as u8) == sq {
                                        return true;
                                    }
                                }
                            }
                        }
                        _ => {
                            // Use pseudo moves for attack map (good for N,B,R,Q,K)
                            if self.get_all_moves(&p).contains(&sq) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    pub fn can_castle(&self, king_pos: (u8, u8), rook_pos: (u8, u8)) -> bool {
        // Get the king and rook pieces
        let (king, rook) = match (
            self.squares[king_pos.0 as usize][king_pos.1 as usize].as_ref(),
            self.squares[rook_pos.0 as usize][rook_pos.1 as usize].as_ref(),
        ) {
            (Some(k), Some(r)) if k.kind == PieceType::King && r.kind == PieceType::Rook => (k, r),
            _ => {
                return false;
            }
        };

        // Must be same color and current side to move
        if king.color != self.turn || rook.color != self.turn {
            return false;
        }
        // Pieces must not have moved
        if king.has_moved || rook.has_moved {
            return false;
        }

        // Check squares are the standard ones (e1/e8 with a1/h1 or a8/h8)
        let castle_squares: [((u8, u8), (u8, u8)); 2] = match self.turn {
            PieceColor::Black => [((0, 4), (0, 0)), ((0, 4), (0, 7))],
            PieceColor::White => [((7, 4), (7, 0)), ((7, 4), (7, 7))],
        };
        if !castle_squares.contains(&(king_pos, rook_pos)) {
            return false;
        }

        // Check castling rights flags
        let castle_type = if king_pos.1.abs_diff(rook_pos.1) == 4 {
            CastleType::QueenSide
        } else {
            CastleType::KingSide
        };
        match (self.turn, &castle_type) {
            (PieceColor::White, CastleType::KingSide) if !self.white_small_castle => return false,
            (PieceColor::White, CastleType::QueenSide) if !self.white_big_castle => return false,
            (PieceColor::Black, CastleType::KingSide) if !self.black_small_castle => return false,
            (PieceColor::Black, CastleType::QueenSide) if !self.black_big_castle => return false,
            _ => {}
        }

        // Path between king and rook must be empty
        let range = if rook_pos.1 < king_pos.1 {
            (rook_pos.1 + 1)..king_pos.1 // queenside: b..d
        } else {
            (king_pos.1 + 1)..rook_pos.1 // kingside: f..g
        };
        for file in range {
            if self.squares[king_pos.0 as usize][file as usize].is_some() {
                return false;
            }
        }

        // King cannot be in check, and transit/destination squares must not be attacked
        if self.is_in_check(self.turn) {
            return false;
        }

        let opponent = match self.turn {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        };
        // Transit and destination files for king: d,c (queenside) or f,g (kingside)
        let (t1, t2) = match castle_type {
            CastleType::QueenSide => (king_pos.1 - 1, king_pos.1 - 2), // d, c
            CastleType::KingSide => (king_pos.1 + 1, king_pos.1 + 2),  // f, g
        };

        let transit1 = (king_pos.0, t1);
        let transit2 = (king_pos.0, t2);

        if self.is_square_attacked_by(transit1, opponent) {
            return false;
        }
        if self.is_square_attacked_by(transit2, opponent) {
            return false;
        }

        true
    }
    pub fn execute_castle(&mut self, king_pos: (u8, u8), rook_pos: (u8, u8)) {
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
        let mut king = self.squares[king_pos.0 as usize][king_pos.1 as usize]
            .take()
            .unwrap();
        king.position = (king_pos.0, king_new_col);
        king.has_moved = true;
        self.squares[king_pos.0 as usize][king_new_col as usize] = Some(king);

        // Move rook
        let mut rook = self.squares[rook_pos.0 as usize][rook_pos.1 as usize]
            .take()
            .unwrap();
        rook.position = (rook_pos.0, rook_new_col);
        rook.has_moved = true;
        self.squares[rook_pos.0 as usize][rook_new_col as usize] = Some(rook);

        // Update castling rights
        if self.turn == PieceColor::White {
            self.white_small_castle = false;
            self.white_big_castle = false;
        } else {
            self.black_small_castle = false;
            self.black_big_castle = false;
        }

        self.been_modified = true;
    }

    pub fn change_turn(&mut self) {
        self.turn = match self.turn {
            PieceColor::Black => PieceColor::White,
            PieceColor::White => PieceColor::Black,
        };
    }

    pub fn promote_pawn(&mut self, pos: (u8, u8), kind: PieceType) {
        if let Some(pawn) = self.squares[pos.0 as usize][pos.1 as usize].as_mut() {
            pawn.kind = kind;
        }
    }
    pub fn select_piece(&mut self, piece: ChessPiece) {
        if self.turn == piece.color {
            self.ui.selected_piece = Some(piece);
        } else {
            self.deselect_piece();
        }
    }
    pub fn deselect_piece(&mut self) {
        self.ui.selected_piece = None;
    }
}
