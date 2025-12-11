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

#[derive(Clone, Debug, TS, Serialize, Deserialize)]
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
    pub meta_data: BoardMetaData,
    pub ui: BoardUi,
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

#[derive(Clone, Debug, TS, Serialize, Deserialize)]
#[ts(export)]
pub enum GameResult {
    WhiteWin,
    BlackWin,
    Draw,
    Unfinished,
}

#[derive(Clone, Debug, TS, Serialize, Deserialize)]
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
    pub opening: Option<String>,
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
            opening: None,
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
        if is_quiet
            && moving_piece.kind == PieceType::King
            && self.is_player_castle(old_pos, new_pos)
        {
            // Clear en passant; castling does not set one
            self.en_passant_target = None;

            // Halfmove clock: castling is not a pawn move nor a capture, so +1
            self.halfmove_clock = self.halfmove_clock.saturating_add(1);

            // Execute castle (moves king and rook + updates rights)
            self.execute_player_castle(old_pos, new_pos);

            // Turn/Fullmove updates
            let was_black = self.turn == PieceColor::Black;
            self.change_turn();
            if was_black {
                self.fullmove_number = self.fullmove_number.saturating_add(1);
            }

            self.been_modified = true;

            return Ok(MoveStruct {
                move_number: self.ply_count,
                uci: self.encode_uci_move(old_pos, new_pos, None), // e1g1/e1c1/e8g8/e8c8
                san: String::new(), // optional: set to "O-O" or "O-O-O" later
                promotion: None,
                is_capture: false,
                evaluation: EvalResponse {
                    value: 0.0,
                    kind: EvalType::Centipawn,
                },
                time_stamp: 0.0,
            });
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
            //self.ui.promtion_pending = Some(new_pos);
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

    /// Returns true if `sq` is attacked by any piece of `by_color`

    /*
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
    */
    pub fn is_player_castle(&self, from: (u8, u8), to: (u8, u8)) -> bool {
        matches!(
            (from, to),
            ((0, 4), (0, 6)) | ((0, 4), (0, 2)) | ((7, 4), (7, 6)) | ((7, 4), (7, 2))
        )
    }
    pub fn is_engine_castle(&self, uci: &str) -> bool {
        matches!(uci, "e1g1" | "e1c1" | "e8g8" | "e8c8")
    }
    pub fn execute_player_castle(&mut self, from: (u8, u8), to: (u8, u8)) {
        match (from, to) {
            // King-side castle
            ((r, 4), (rr, 6)) if r == rr => {
                // Move king
                if let Some(mut king) = self.squares[r as usize][4].take() {
                    king.position = (r, 6);
                    king.has_moved = true;
                    self.squares[r as usize][6] = Some(king);
                }

                // Move rook from h-file to f-file
                if let Some(mut rook) = self.squares[r as usize][7].take() {
                    rook.position = (r, 5);
                    rook.has_moved = true;
                    self.squares[r as usize][5] = Some(rook);
                }

                // Update castling rights
                if r == 7 {
                    self.white_small_castle = false;
                    self.white_big_castle = false;
                } else {
                    self.black_small_castle = false;
                    self.black_big_castle = false;
                }

                self.been_modified = true;
            }

            // Queen-side castle
            ((r, 4), (rr, 2)) if r == rr => {
                // Move king
                if let Some(mut king) = self.squares[r as usize][4].take() {
                    king.position = (r, 2);
                    king.has_moved = true;
                    self.squares[r as usize][2] = Some(king);
                }

                // Move rook from a-file to d-file
                if let Some(mut rook) = self.squares[r as usize][0].take() {
                    rook.position = (r, 3);
                    rook.has_moved = true;
                    self.squares[r as usize][3] = Some(rook);
                }

                // Update castling rights
                if r == 7 {
                    self.white_small_castle = false;
                    self.white_big_castle = false;
                } else {
                    self.black_small_castle = false;
                    self.black_big_castle = false;
                }

                self.been_modified = true;
            }

            _ => {}
        }
    }

    pub fn execute_engine_castle(&mut self, uci: &str) {
        match uci {
            // White king-side: e1g1
            "e1g1" => {
                // Move king
                if let Some(mut king) = self.squares[7][4].take() {
                    king.position = (7, 6);
                    king.has_moved = true;
                    self.squares[7][6] = Some(king);
                }
                // Move rook h1 -> f1
                if let Some(mut rook) = self.squares[7][7].take() {
                    rook.position = (7, 5);
                    rook.has_moved = true;
                    self.squares[7][5] = Some(rook);
                }
                self.white_small_castle = false;
                self.white_big_castle = false;
                self.been_modified = true;
            }
            // White queen-side: e1c1
            "e1c1" => {
                // Move king
                if let Some(mut king) = self.squares[7][4].take() {
                    king.position = (7, 2);
                    king.has_moved = true;
                    self.squares[7][2] = Some(king);
                }
                // Move rook a1 -> d1
                if let Some(mut rook) = self.squares[7][0].take() {
                    rook.position = (7, 3);
                    rook.has_moved = true;
                    self.squares[7][3] = Some(rook);
                }
                self.white_small_castle = false;
                self.white_big_castle = false;
                self.been_modified = true;
            }
            // Black king-side: e8g8
            "e8g8" => {
                // Move king
                if let Some(mut king) = self.squares[0][4].take() {
                    king.position = (0, 6);
                    king.has_moved = true;
                    self.squares[0][6] = Some(king);
                }
                // Move rook h8 -> f8
                if let Some(mut rook) = self.squares[0][7].take() {
                    rook.position = (0, 5);
                    rook.has_moved = true;
                    self.squares[0][5] = Some(rook);
                }
                self.black_small_castle = false;
                self.black_big_castle = false;
                self.been_modified = true;
            }
            // Black queen-side: e8c8
            "e8c8" => {
                // Move king
                if let Some(mut king) = self.squares[0][4].take() {
                    king.position = (0, 2);
                    king.has_moved = true;
                    self.squares[0][2] = Some(king);
                }
                // Move rook a8 -> d8
                if let Some(mut rook) = self.squares[0][0].take() {
                    rook.position = (0, 3);
                    rook.has_moved = true;
                    self.squares[0][3] = Some(rook);
                }
                self.black_small_castle = false;
                self.black_big_castle = false;
                self.been_modified = true;
            }
            _ => {}
        }
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
}
