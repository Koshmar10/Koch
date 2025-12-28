use crate::{
    engine::{fen::fen_parser, move_gen::MoveError, ChessPiece, PieceColor, PieceType},
    etc::{DEFAULT_FEN, DEFAULT_STARTING},
    game::controller::TerminationReason,
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
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
    Resignation,
    Unknown,
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
    pub move_cache: std::collections::HashMap<u32, PieceMoves>,
    pub next_id: u32,
    pub game_phase: GamePhase,
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
    pub pov: PieceColor,
    pub white_taken: Vec<(PieceType, PieceColor)>,
    pub black_taken: Vec<(PieceType, PieceColor)>,
}

#[derive(Clone, Debug, TS, Serialize, Deserialize)]
#[ts(export)]
pub enum GameResult {
    WhiteWin,
    BlackWin,
    Draw,
    Unfinished,
}
impl From<&str> for GameResult {
    fn from(s: &str) -> Self {
        match s {
            "1-0" => GameResult::WhiteWin,
            "0-1" => GameResult::BlackWin,
            "1/2-1/2" => GameResult::Draw,
            _ => GameResult::Unfinished,
        }
    }
}
impl ToString for GameResult {
    fn to_string(&self) -> String {
        match self {
            GameResult::WhiteWin => "1-0".to_string(),
            GameResult::BlackWin => "0-1".to_string(),
            GameResult::Draw => "1/2-1/2".to_string(),
            GameResult::Unfinished => "*".to_string(),
        }
    }
}
pub fn parse_game_result(s: &str) -> GameResult {
    GameResult::from(s)
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum GamePhase {
    Opening,
    MiddleGame,
    EndGame,
}

impl std::fmt::Display for GamePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            GamePhase::Opening => "Opening",
            GamePhase::MiddleGame => "MiddleGame",
            GamePhase::EndGame => "EndGame",
        };
        write!(f, "{}", s)
    }
}
#[derive(Clone, Debug, TS, Serialize, Deserialize)]
#[ts(export)]
pub struct BoardMetaData {
    pub starting_position: String,
    pub date: String,
    pub move_list: Vec<MoveStruct>,
    pub termination: TerminationReason,
    pub result: GameResult,
    pub white_player_elo: u32,
    pub black_player_elo: u32,
    pub white_player_name: String,
    pub black_player_name: String,

    pub opening: Option<String>,

    // Additional optional PGN tags
    pub event: Option<String>,
    pub site: Option<String>,
    pub round: Option<String>,
    pub time_control: Option<String>,
    pub end_time: Option<String>,
    pub link: Option<String>,
    pub eco: Option<String>,
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
                meta_data: BoardMetaData::default(),
                move_cache: std::collections::HashMap::new(),
                game_phase: GamePhase::Opening,
                next_id: 0,
                ply_count: 0,
            },
        }
    }
}
impl Default for BoardUi {
    fn default() -> Self {
        Self {
            pov: DEFAULT_STARTING,
            white_taken: Vec::new(),
            black_taken: Vec::new(),
        }
    }
}
impl Default for BoardMetaData {
    fn default() -> Self {
        Self {
            starting_position: DEFAULT_FEN.to_string(),
            date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            move_list: Vec::new(),
            termination: TerminationReason::Draw,
            result: GameResult::Unfinished,
            white_player_elo: 0,
            black_player_elo: 0,
            white_player_name: String::new(),
            black_player_name: String::new(),
            opening: None,
            event: None,
            site: None,
            round: None,
            time_control: None,
            end_time: None,
            eco: None,
            link: None,
        }
    }
}

#[derive(Clone, TS, Serialize, Deserialize, Debug)]
#[ts(export)]

pub struct MoveStruct {
    pub move_number: u32,
    pub san: String,
    pub uci: String,
    pub promotion: Option<PieceType>,
    pub is_capture: bool,
    pub annotation: Option<String>,
    pub nag: Option<i32>,
    pub time_stamp: Option<u32>,
    pub clock: Option<String>,
}
impl Default for MoveStruct {
    fn default() -> Self {
        MoveStruct {
            move_number: 0,
            san: String::new(),
            uci: String::new(),
            promotion: None,
            is_capture: false,
            annotation: None,
            nag: None,
            time_stamp: None,
            clock: None,
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
//uci|san|clock|time_stamp|nag|annotation <-serialization
impl ToString for MoveStruct {
    fn to_string(&self) -> String {
        let uci = &self.uci;
        let san = &self.san;
        let clock = self.clock.as_deref().unwrap_or("_");
        let time_stamp = self.time_stamp.unwrap_or(0);
        let nag = self.nag.unwrap_or(-1);
        let annotation = self.annotation.as_deref().unwrap_or("_");
        format!(
            "{}|{}|{}|{}|{}|{}",
            uci, san, clock, time_stamp, nag, annotation
        )
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
        promotion: Option<PieceType>,
    ) -> Result<MoveStruct, MoveError> {
        let moving_piece = match self.squares[old_pos.0 as usize][old_pos.1 as usize] {
            Some(piece) => piece,
            None => {
                return Err(MoveError::NoAviailableMoves);
            }
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
        let san = self
            .compute_san_for_move(old_pos, new_pos, promotion, is_capture)
            .unwrap_or_else(|_| String::new());
        if is_quiet
            && moving_piece.kind == PieceType::King
            && self.is_player_castle(old_pos, new_pos)
        {
            // Clear en passant; castling does not set one
            self.en_passant_target = None;

            // Halfmove clock: castling is not a pawn move nor a capture, so +1
            self.halfmove_clock = self.halfmove_clock.saturating_add(1);

            // Debug: entering player castling

            // Execute castle (moves king and rook + updates rights)
            self.execute_player_castle(old_pos, new_pos);

            // Turn/Fullmove updates
            let was_black = self.turn == PieceColor::Black;
            self.change_turn();
            if was_black {
                self.fullmove_number = self.fullmove_number.saturating_add(1);
            }

            let mv = MoveStruct {
                move_number: self.ply_count,
                uci: self.encode_uci_move(old_pos, new_pos, promotion), // e1g1/e1c1/e8g8/e8c8
                san: san,
                promotion: None,
                is_capture: false,
                annotation: None,
                nag: None,
                time_stamp: None,
                clock: None,
            };

            return Ok(mv);
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
            .ok_or_else(|| MoveError::NoAviailableMoves)?;

        // Promotion detection: determine desired promotion kind (if any)
        let mut promotion_applied: Option<PieceType> = None;
        let is_pawn_promotion = moving_piece.kind == PieceType::Pawn
            && ((moving_piece.color == PieceColor::White && new_pos.0 == 0)
                || (moving_piece.color == PieceColor::Black && new_pos.0 == 7));
        if is_pawn_promotion {
            // if caller passed a promotion choose it, otherwise default to Queen
            promotion_applied = promotion.or(Some(PieceType::Queen));
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
                    (PieceColor::White, (7, 0)) => {
                        self.white_big_castle = false;
                    }
                    (PieceColor::White, (7, 7)) => {
                        self.white_small_castle = false;
                    }
                    (PieceColor::Black, (0, 0)) => {
                        self.black_big_castle = false;
                    }
                    (PieceColor::Black, (0, 7)) => {
                        self.black_small_castle = false;
                    }
                    _ => {}
                }
            }
        }

        let is_pawn_move = moving_piece.kind == PieceType::Pawn;

        // Apply promotion kind if needed before placing piece
        if let Some(prom_kind) = promotion_applied {
            moving_piece.kind = prom_kind;
        }
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

        self.update_gamephase();
        Ok(MoveStruct {
            move_number: self.ply_count,
            uci: self.encode_uci_move(old_pos, new_pos, promotion_applied),
            san: san,
            promotion: promotion_applied,
            is_capture,
            annotation: None,
            nag: None,
            time_stamp: None,
            clock: None,
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
                } else {
                }

                // Move rook from h-file to f-file
                if let Some(mut rook) = self.squares[r as usize][7].take() {
                    println!(
                        "[DEBUG][execute_player_castle] Moving rook from ({} ,7) to ({} ,5)",
                        r, r
                    );
                    rook.position = (r, 5);
                    rook.has_moved = true;
                    self.squares[r as usize][5] = Some(rook);
                } else {
                }

                // Update castling rights
                if r == 7 {
                    self.white_small_castle = false;
                    self.white_big_castle = false;
                } else {
                    self.black_small_castle = false;
                    self.black_big_castle = false;
                }
            }

            // Queen-side castle
            ((r, 4), (rr, 2)) if r == rr => {
                // Move king
                if let Some(mut king) = self.squares[r as usize][4].take() {
                    king.position = (r, 2);
                    king.has_moved = true;
                    self.squares[r as usize][2] = Some(king);
                } else {
                }

                // Move rook from a-file to d-file
                if let Some(mut rook) = self.squares[r as usize][0].take() {
                    rook.position = (r, 3);
                    rook.has_moved = true;
                    self.squares[r as usize][3] = Some(rook);
                } else {
                }

                // Update castling rights
                if r == 7 {
                    self.white_small_castle = false;
                    self.white_big_castle = false;
                } else {
                    self.black_small_castle = false;
                    self.black_big_castle = false;
                }
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
                } else {
                }
                // Move rook h1 -> f1
                if let Some(mut rook) = self.squares[7][7].take() {
                    rook.position = (7, 5);
                    rook.has_moved = true;
                    self.squares[7][5] = Some(rook);
                } else {
                }
                self.white_small_castle = false;
                self.white_big_castle = false;
            }
            // White queen-side: e1c1
            "e1c1" => {
                // Move king
                if let Some(mut king) = self.squares[7][4].take() {
                    king.position = (7, 2);
                    king.has_moved = true;
                    self.squares[7][2] = Some(king);
                } else {
                }
                // Move rook a1 -> d1
                if let Some(mut rook) = self.squares[7][0].take() {
                    rook.position = (7, 3);
                    rook.has_moved = true;
                    self.squares[7][3] = Some(rook);
                } else {
                }
                self.white_small_castle = false;
                self.white_big_castle = false;
            }
            // Black king-side: e8g8
            "e8g8" => {
                // Move king
                if let Some(mut king) = self.squares[0][4].take() {
                    king.position = (0, 6);
                    king.has_moved = true;
                    self.squares[0][6] = Some(king);
                } else {
                }
                // Move rook h8 -> f8
                if let Some(mut rook) = self.squares[0][7].take() {
                    rook.position = (0, 5);
                    rook.has_moved = true;
                    self.squares[0][5] = Some(rook);
                } else {
                }
                self.black_small_castle = false;
                self.black_big_castle = false;
            }
            // Black queen-side: e8c8
            "e8c8" => {
                println!("[DEBUG][execute_engine_castle] Black queen-side");
                // Move king
                if let Some(mut king) = self.squares[0][4].take() {
                    king.position = (0, 2);
                    king.has_moved = true;
                    self.squares[0][2] = Some(king);
                } else {
                    println!("[DEBUG][execute_engine_castle] WARNING: no black king at 0,4");
                }
                // Move rook a8 -> d8
                if let Some(mut rook) = self.squares[0][0].take() {
                    rook.position = (0, 3);
                    rook.has_moved = true;
                    self.squares[0][3] = Some(rook);
                } else {
                    println!("[DEBUG][execute_engine_castle] WARNING: no black rook at 0,0");
                }
                self.black_small_castle = false;
                self.black_big_castle = false;
            }
            _ => {
                println!(
                    "[DEBUG][execute_engine_castle] Unknown uci for castle: {}",
                    uci
                );
            }
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
    fn sq_to_coord(sq: &str) -> Result<(u8, u8), MoveError> {
        let s = sq.trim();

        let bytes = s.as_bytes();
        let file = bytes[0];
        let rank = bytes[1];

        if !(b'a'..=b'h').contains(&file) {
            println!("[DEBUG] sq_to_coord: file out of range '{}'", file as char);
            return Err(MoveError::IllegalMove);
        }
        if !(b'1'..=b'8').contains(&rank) {
            println!("[DEBUG] sq_to_coord: rank out of range '{}'", rank as char);
            return Err(MoveError::IllegalMove);
        }

        let col = file - b'a';
        let rank_digit = (rank - b'0') as u8; // '1' -> 1 .. '8' -> 8
        let row = 8u8 - rank_digit; // convert to board row (0 = rank 8)

        Ok((row, col))
    }

    pub fn update_gamephase(&mut self) {
        enum MoveCount {
            Low,
            High,
        }
        #[derive(PartialEq)]
        enum BackRank {
            Blocked,
            Open,
            Empty,
        }
        let move_count = {
            let moves = self.meta_data.move_list.len() / 2;
            if moves < 15 {
                MoveCount::Low
            } else {
                MoveCount::High
            }
        };
        let (material_sum, white_backrank, black_backrank) = {
            let mut material_sum = 0;
            let mut white_backrank_count = 0;
            let mut black_backrank_count = 0;
            let backrank_targets = &[PieceType::Bishop, PieceType::Queen, PieceType::Knight];
            for i in 0..8usize {
                for j in 0..8usize {
                    let square = &self.squares[i][j];
                    if let Some(piece) = square {
                        if i == 0 {
                            if backrank_targets.contains(&piece.kind)
                                && PieceColor::Black == piece.color
                            {
                                black_backrank_count += 1;
                            }
                        }
                        if i == 7 {
                            if backrank_targets.contains(&piece.kind)
                                && PieceColor::White == piece.color
                            {
                                white_backrank_count += 1;
                            }
                        }

                        material_sum += Self::material_value(&piece.kind);
                    }
                }
            }
            (
                material_sum,
                if white_backrank_count > 3 {
                    BackRank::Blocked
                } else {
                    if white_backrank_count > 0 {
                        BackRank::Open
                    } else {
                        BackRank::Empty
                    }
                },
                if black_backrank_count > 3 {
                    BackRank::Blocked
                } else {
                    if black_backrank_count > 0 {
                        BackRank::Open
                    } else {
                        BackRank::Empty
                    }
                },
            )
        };

        let kings_untouched = self.white_small_castle
            && self.white_big_castle
            && self.black_small_castle
            && self.black_big_castle;

        self.game_phase = match (material_sum, move_count, white_backrank, black_backrank) {
            // 1. ENDGAME (Absolute Priority)
            (m, _, _, _) if m <= 30 => GamePhase::EndGame,

            // 2. OPENING (Enhanced "Stickiness")
            // Stay in Opening if:
            // - Either side is "Blocked" (Your current logic)
            // - OR Move count is low (Under 15 moves)
            // - OR Both kings are still in the center (untouched castling rights)
            (_, MoveCount::Low, _, _)
            | (_, _, BackRank::Blocked, _)
            | (_, _, _, BackRank::Blocked)
            | (_, _, _, _)
                if kings_untouched =>
            {
                GamePhase::Opening
            }

            // 3. MIDDLE GAME
            // Only reached if material is high, moves > 15, and development is "Open" or "Empty"
            _ => GamePhase::MiddleGame,
        };
    }
    pub fn material_value(kind: &PieceType) -> u8 {
        match kind {
            PieceType::Bishop => 3,
            PieceType::Knight => 3,
            PieceType::Queen => 9,
            PieceType::Rook => 5,
            _ => 0,
        }
    }

    pub fn san_to_uci(&mut self, san: &str) -> Result<String, MoveError> {
        let mut promotion: Option<PieceType> = None;
        self.rerender_move_cache();
        // Check if san is castle:
        if san == "O-O" {
            match self.turn {
                PieceColor::Black => {
                    return Ok("e8g8".to_string()); // kingside castle move for black
                }
                PieceColor::White => {
                    return Ok("e1g1".to_string()); // kingside castle move for white
                }
            }
        }
        if san == "O-O-O" {
            match self.turn {
                PieceColor::Black => {
                    return Ok("e8c8".to_string()); // queenside castle move for black
                }
                PieceColor::White => {
                    return Ok("e1c1".to_string()); // queenside castle move for white
                }
            }
        }
        if san == "*" || san == "1-0" || san == "0-1" || san == "1/2-1/2" {
            return Err(MoveError::IllegalMove);
        }
        // Remove trailing characters
        let mut normalized_san = Self::normalize_san_token(san);

        if let Ok(res) = Self::consume_promotion(normalized_san) {
            (normalized_san, promotion) = res;
        } else {
            return Err(MoveError::IllegalMove);
        }

        // Transform normalized san into a vec
        // This means is a simple pawn move
        let mut san_chars: Vec<char> = normalized_san.chars().collect();

        let is_capture = san_chars.contains(&'x');
        if is_capture {
            san_chars.retain(|c| c != &'x');
        }

        // ---------- FIX: build dest from cleaned san_chars (not from normalized_san) ----------
        if san_chars.len() == 2 {
            // parse file and rank from cleaned chars (e.g. ['e','4'])
            let dest_str: String = san_chars.iter().collect();
            let dest = Self::sq_to_coord(&dest_str).map_err(|_e| MoveError::IllegalMove)?;

            // find a pawn of the side to move that has dest in its quiet or capture moves
            for i in 0..8u8 {
                for j in 0..8u8 {
                    if let Some(piece) = &self.squares[i as usize][j as usize] {
                        if piece.kind == PieceType::Pawn && piece.color == self.turn {
                            if let Some(pms) = self.move_cache.get(&piece.id) {
                                if pms.quiet_moves.contains(&dest)
                                    || pms.capture_moves.contains(&dest)
                                {
                                    return Ok(self.encode_uci_move((i, j), dest, promotion));
                                }
                            }
                        }
                    }
                }
            }
        }
        if san_chars.len() == 3 {
            // e.g. "exd5" cleaned -> ['e','d','5'] where [1..3] is dest
            let dest_str: String = san_chars[1..3].iter().collect();
            let dest = Self::sq_to_coord(&dest_str).map_err(|_e| MoveError::IllegalMove)?;

            let pawn_move = "abcdefgh".contains(san_chars[0]);
            for i in 0..8u8 {
                for j in 0..8u8 {
                    if let Some(piece) = &self.squares[i as usize][j as usize] {
                        let mut kind = PieceType::Pawn;
                        if !pawn_move {
                            kind = match san_chars[0] {
                                'Q' => PieceType::Queen,
                                'R' => PieceType::Rook,
                                'B' => PieceType::Bishop,
                                'N' => PieceType::Knight,
                                'K' => PieceType::King,
                                _ => return Err(MoveError::IllegalMove),
                            };
                        }
                        if pawn_move {
                            // For pawn moves like exd5, the first char is the file of the pawn
                            if piece.kind == PieceType::Pawn && piece.color == self.turn {
                                let file_char = san_chars[0];
                                if !(file_char >= 'a' && file_char <= 'h') {
                                    continue;
                                }
                                let file_idx = (file_char as u8 - b'a') as u8;
                                if j != file_idx {
                                    continue;
                                }
                                if let Some(pms) = self.move_cache.get(&piece.id) {
                                    if is_capture {
                                        if pms.capture_moves.contains(&dest) {
                                            return Ok(self.encode_uci_move(
                                                (i, j),
                                                dest,
                                                promotion,
                                            ));
                                        }
                                    } else {
                                        if pms.quiet_moves.contains(&dest) {
                                            return Ok(self.encode_uci_move(
                                                (i, j),
                                                dest,
                                                promotion,
                                            ));
                                        }
                                    }
                                }
                            }
                        } else {
                            // Piece moves (e.g., Nf3, Rxa7)
                            if piece.kind != kind || piece.color != self.turn {
                                continue;
                            }
                            if let Some(pms) = self.move_cache.get(&piece.id) {
                                if is_capture {
                                    if pms.capture_moves.contains(&dest) {
                                        return Ok(self.encode_uci_move((i, j), dest, promotion));
                                    }
                                } else {
                                    if pms.quiet_moves.contains(&dest) {
                                        return Ok(self.encode_uci_move((i, j), dest, promotion));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        //now we need to handle disambiguation moves
        let san_len = san_chars.len();
        let piece_c = san_chars[0];
        let dest_str: String = san_chars[san_len - 2..san_len].iter().collect();
        let dest = Self::sq_to_coord(&dest_str).map_err(|_e| MoveError::IllegalMove)?;
        for i in 0..8u8 {
            for j in 0..8u8 {
                let kind = match san_chars[0] {
                    'Q' => PieceType::Queen,
                    'R' => PieceType::Rook,
                    'B' => PieceType::Bishop,
                    'N' => PieceType::Knight,
                    'K' => PieceType::King,
                    _ => return Err(MoveError::IllegalMove),
                };
                match &self.squares[i as usize][j as usize] {
                    Some(piece) => {
                        let mut check_rank = false;
                        let mut check_rank_value = -1;
                        let mut check_file = false;
                        let mut check_file_value = -1;
                        let mut enforce_both = false;
                        if san_len == 4 {
                            // san_chars[1] is either a file or rank specifier
                            let c = san_chars[1];
                            if c.is_ascii_digit() {
                                // Rank specifier
                                check_rank = true;
                                check_rank_value = 8 - (c as u8 - b'0') as i8;
                            } else if c >= 'a' && c <= 'h' {
                                // File specifier
                                check_file = true;
                                check_file_value = (c as u8 - b'a') as i8;
                            }
                        }
                        if san_len == 5 {
                            // Both file and rank specifiers are present: [piece][file][rank][dest-file][dest-rank]
                            check_file = true;
                            check_file_value = (san_chars[1] as u8 - b'a') as i8;
                            if let Some(d) = san_chars[2].to_digit(10) {
                                check_rank = true;
                                check_rank_value = (8u8 - d as u8) as i8;
                            }
                            enforce_both = true;
                        }
                        let condition = if enforce_both {
                            (check_file && check_file_value == j as i8)
                                && (check_rank && check_rank_value == i as i8)
                        } else {
                            (check_file && check_file_value == j as i8)
                                || (check_rank && check_rank_value == i as i8)
                        };
                        if condition && piece.kind == kind {
                            if let Some(pms) = self.move_cache.get(&piece.id) {
                                if is_capture {
                                    if pms.capture_moves.contains(&dest) {
                                        return Ok(self.encode_uci_move((i, j), dest, promotion));
                                    }
                                } else {
                                    if pms.quiet_moves.contains(&dest) {
                                        return Ok(self.encode_uci_move((i, j), dest, promotion));
                                    }
                                }
                            }
                        }
                    }
                    None => {}
                }
            }
        }

        //if function still has not returned this means that the move involves either a pawn capture or a piece move

        Err(MoveError::IllegalMove)
    }

    /// Normalize SAN: strips trailing +/#/!/?, trims whitespace.
    fn normalize_san_token(s: &str) -> String {
        let mut t = s.trim().to_string();
        // Remove trailing check/mate/annotation, but keep 'x'
        while let Some(last) = t.chars().last() {
            if last == '+' || last == '#' || last == '!' || last == '?' {
                t.pop();
            } else {
                break;
            }
        }
        // Do not change case — return as-is (preserve original SAN casing)
        t
    }
    fn consume_promotion(san: String) -> Result<(String, Option<PieceType>), MoveError> {
        if san.len() < 2 {
            return Ok((san, None));
        }
        let san_len = san.len();
        let promotion_part = &san[san_len - 2..san_len];
        let mut chars = promotion_part.chars();
        if chars.next() == Some('=') {
            if let Some(c) = chars.next() {
                let piece_type = match c {
                    'Q' | 'q' => PieceType::Queen,
                    'R' | 'r' => PieceType::Rook,
                    'B' | 'b' => PieceType::Bishop,
                    'N' | 'n' => PieceType::Knight,
                    _ => return Err(MoveError::IllegalMove),
                };
                let normalized_san = san[..san_len - 2].to_string();
                return Ok((normalized_san, Some(piece_type)));
            } else {
                return Err(MoveError::IllegalMove);
            }
        }
        Ok((san, None))
    }

    /// Convert board coords (row,col) to algebraic square "e4"
    fn coord_to_sq(pos: (u8, u8)) -> String {
        let file = (b'a' + pos.1) as char;
        let rank = (8 - pos.0).to_string();
        format!("{}{}", file, rank)
    }

    /// Piece letter for SAN (pawn = "")
    fn piece_letter(kind: &PieceType) -> &'static str {
        match kind {
            PieceType::King => "K",
            PieceType::Queen => "Q",
            PieceType::Rook => "R",
            PieceType::Bishop => "B",
            PieceType::Knight => "N",
            PieceType::Pawn => "",
        }
    }

    /// Compute SAN for a move.
    /// - `from` and `to` are board coordinates in (row, col) (0..7)
    /// - `promotion` is optional promotion piece kind
    /// - `is_capture` indicates whether this move captures a piece (including en-passant)
    ///
    /// Note: call this on the board state BEFORE applying the move (so `self` represents the pre-move board).
    pub fn compute_san_for_move(
        &self,
        from: (u8, u8),
        to: (u8, u8),
        promotion: Option<PieceType>,
        is_capture: bool,
    ) -> Result<String, MoveError> {
        // validate moving piece exists
        let moving = self.squares[from.0 as usize][from.1 as usize]
            .as_ref()
            .ok_or(MoveError::IllegalMove)?;

        // Castling
        if moving.kind == PieceType::King {
            // King-side castle (move two files right)
            if from.1 + 2 == to.1 {
                return Ok("O-O".to_string());
            }
            // Queen-side castle (move two files left)
            if from.1 == 4 && to.1 == 2 {
                return Ok("O-O-O".to_string());
            }
        }

        let dest_sq = Self::coord_to_sq(to);
        // Pawn moves
        if moving.kind == PieceType::Pawn {
            let origin_file = (b'a' + from.1) as char;
            let mut san = String::new();
            if is_capture {
                // exd5 style
                san.push(origin_file);
                san.push('x');
                san.push_str(&dest_sq);
            } else {
                // e4 style
                san.push_str(&dest_sq);
            }
            // promotion
            if let Some(prom) = promotion {
                san.push('=');
                san.push_str(match prom {
                    PieceType::Queen => "Q",
                    PieceType::Rook => "R",
                    PieceType::Bishop => "B",
                    PieceType::Knight => "N",
                    _ => "Q",
                });
            }
            return Ok(san);
        }

        // Piece moves (N, B, R, Q, K)
        let piece_letter = Self::piece_letter(&moving.kind);
        // Find other pieces of same kind & color that can also move to `to`
        let mut ambiguous_positions: Vec<(u8, u8)> = Vec::new();
        for r in 0..8u8 {
            for c in 0..8u8 {
                if r == from.0 && c == from.1 {
                    continue;
                }
                if let Some(piece) = &self.squares[r as usize][c as usize] {
                    if piece.kind == moving.kind && piece.color == moving.color {
                        // check move_cache: if this piece can move to `to`, it's ambiguous
                        if let Some(pms) = self.move_cache.get(&piece.id) {
                            if pms.quiet_moves.contains(&to) || pms.capture_moves.contains(&to) {
                                ambiguous_positions.push((r, c));
                            }
                        }
                    }
                }
            }
        }

        // determine disambiguation
        let mut disamb = String::new();
        if !ambiguous_positions.is_empty() {
            // if any ambiguous piece shares the same file as 'from', include rank; otherwise include file
            let same_file_exists = ambiguous_positions.iter().any(|&(r, c)| c == from.1);
            if same_file_exists {
                // include rank (numeric) of origin
                let rank_char = (8 - from.0).to_string();
                disamb.push_str(&rank_char);
            } else {
                // include file letter of origin
                let file_char = (b'a' + from.1) as char;
                disamb.push(file_char);
            }

            // If still ambiguous (rare), include both file+rank
            // check if disamb chosen is unique; if not, fallback to file+rank
            let test = format!("{}{}", piece_letter, disamb);
            let mut matches = 0;
            for &(r, c) in ambiguous_positions.iter() {
                let mut candidate = String::new();
                if moving.kind != PieceType::Pawn {
                    candidate.push_str(Self::piece_letter(&moving.kind));
                }
                let file_c = (b'a' + c) as char;
                let rank_c = (8 - r).to_string();
                candidate.push_str(&format!("{}{}", file_c, rank_c));
                if candidate.contains(&to_cow_string(&dest_sq)) {
                    matches += 1;
                }
            }
            // If matches still > 0 (ambiguous), force full disambiguation file+rank
            // (Simpler approach: always include both file+rank if more than one ambiguous piece exists that would not be disambiguated by single char)
            if ambiguous_positions.len() > 1 && disamb.len() == 1 {
                // include both file and rank
                let file_char = (b'a' + from.1) as char;
                let rank_char = (8 - from.0).to_string();
                disamb = format!("{}{}", file_char, rank_char);
            }
        }

        // capture marker
        let capture_mark = if is_capture { "x" } else { "" };

        // promotion for non-pawn (rare) keep empty
        let promotion_suffix = if let Some(prom) = promotion {
            format!(
                "={}",
                match prom {
                    PieceType::Queen => "Q",
                    PieceType::Rook => "R",
                    PieceType::Bishop => "B",
                    PieceType::Knight => "N",
                    _ => "Q",
                }
            )
        } else {
            String::new()
        };

        let san = format!(
            "{}{}{}{}{}",
            piece_letter, disamb, capture_mark, dest_sq, promotion_suffix
        );

        Ok(san)
    }
}

// helper to satisfy small test above when checking match; convert &str to owned String
fn to_cow_string(s: &str) -> String {
    s.to_string()
}
