use crate::{
    analyzer::analyzer::{AnalyzerController, BoardState, MoveKind, UndoInfo},
    engine::{
        board::{MoveStruct, PieceMoves},
        move_gen::MoveError,
        serializer::{
            serialize_analyzer_controller, SerializedAnalyzerController, SerializedBoard,
        },
        Board, ChessPiece, PieceColor, PieceType,
    },
    server::server::{PvObject, ServerState},
};
use serde::{Deserialize, Serialize};
use std::{char, error::Error, sync::Mutex};
use ts_rs::TS;

pub enum AnalyzerWindowOprion {
    Emtpy,
    HeatMap,
}

// New struct to store irreversible FEN state

#[tauri::command]
pub fn get_fen(state: tauri::State<'_, Mutex<ServerState>>) -> String {
    let state = state.lock().unwrap();
    let fen = state.analyzer_controller.get_fen();
    return fen;
}

impl Board {
    pub fn move_piece_with_undo(
        &mut self,
        old_pos: (u8, u8),
        new_pos: (u8, u8),
        promotion: Option<PieceType>,
    ) -> Result<UndoInfo, MoveError> {
        let mut en_passant_move = false;
        let mut en_passant_enum = MoveKind::EnPassant {
            captured_at: (0, 0),
        };

        let mut normal_move: bool = true;
        let mut normal_enum = MoveKind::Normal {
            promotion: promotion,
        };

        let moving_piece = match self.squares[old_pos.0 as usize][old_pos.1 as usize] {
            Some(piece) => piece,
            None => {
                return Err(MoveError::NoAviailableMoves);
            }
        };

        if moving_piece.color != self.turn {
            return Err(MoveError::IllegalMove);
        }

        // Snapshot the irreversible board state BEFORE making any changes.
        let prev_state_snapshot = BoardState {
            turn: self.turn,
            white_big_castle: self.white_big_castle,
            white_small_castle: self.white_small_castle,
            black_big_castle: self.black_big_castle,
            black_small_castle: self.black_small_castle,
            en_passant_target: self.en_passant_target,
            halfmove_clock: self.halfmove_clock,
            fullmove_number: self.fullmove_number,
        };

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

            // Debug: entering player castling

            // Execute castle (moves king and rook + updates rights)
            self.execute_player_castle(old_pos, new_pos);

            // Turn/Fullmove updates
            let was_black = self.turn == PieceColor::Black;
            self.change_turn();
            if was_black {
                self.fullmove_number = self.fullmove_number.saturating_add(1);
            }

            self.been_modified = true;
            let rank: u8 = match moving_piece.color {
                PieceColor::White => 7,
                PieceColor::Black => 0,
            };
            let undodata = UndoInfo {
                from: old_pos,
                to: new_pos,
                captured: None,
                prev_state: prev_state_snapshot.clone(),
                kind: if new_pos.1 < old_pos.1 {
                    MoveKind::Castling {
                        rook_from: (rank, 0),
                        rook_to: (rank, 3),
                    }
                } else {
                    MoveKind::Castling {
                        rook_from: (rank, 7),
                        rook_to: (rank, 5),
                    }
                },
            };

            return Ok(undodata);
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
                en_passant_move = true;
                en_passant_enum = MoveKind::EnPassant {
                    captured_at: (captured_r, new_pos.1),
                }
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
                    println!();
                }
                if old_pos == (7, 7) {
                    self.white_small_castle = false;
                    println!();
                }
            }
            (PieceType::Rook, PieceColor::Black) => {
                if old_pos == (0, 0) {
                    self.black_big_castle = false;
                    println!();
                }
                if old_pos == (0, 7) {
                    self.black_small_castle = false;
                    println!();
                }
            }
            _ => {}
        }

        if let Some(captured) = &captured_piece {
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

        self.been_modified = true;
        // choose the correct move kind (use the snapshot taken before any mutation)
        let kind = if en_passant_move {
            en_passant_enum
        } else {
            normal_enum
        };
        self.update_gamephase();
        return Ok(UndoInfo {
            from: old_pos,
            to: new_pos,
            captured: captured_piece,
            prev_state: prev_state_snapshot,
            kind,
        });
    }

    pub fn apply_undo(&mut self, undo: UndoInfo) -> Result<(), MoveError> {
        // Helper to pop a piece from a square
        let take_square = |board: &mut Board, pos: (u8, u8)| -> Option<ChessPiece> {
            board.squares[pos.0 as usize][pos.1 as usize].take()
        };

        // Reverse the moved piece(s)
        match undo.kind {
            MoveKind::Castling { rook_from, rook_to } => {
                // King was at `to`, move it back to `from`
                let mut king =
                    take_square(self, undo.to).ok_or_else(|| MoveError::NoAviailableMoves)?;
                king.position = undo.from;
                // Conservative: mark as not moved when reverting (best-effort)
                king.has_moved = false;
                self.squares[undo.from.0 as usize][undo.from.1 as usize] = Some(king);

                // Rook was at `rook_to`, move it back to `rook_from`
                let mut rook =
                    take_square(self, rook_to).ok_or_else(|| MoveError::NoAviailableMoves)?;
                rook.position = rook_from;
                rook.has_moved = false;
                self.squares[rook_from.0 as usize][rook_from.1 as usize] = Some(rook);
            }

            MoveKind::EnPassant { captured_at } => {
                // Moving pawn currently at `to` -> move back to `from`
                let mut mover =
                    take_square(self, undo.to).ok_or_else(|| MoveError::NoAviailableMoves)?;
                // If the move was a promotion that resulted in a different kind, revert to pawn
                // (promotion info for en-passant is unlikely but be defensive)
                if let MoveKind::Normal { .. } = &undo.kind {
                    // noop here; keeping for symmetry
                }
                mover.position = undo.from;
                mover.has_moved = false;
                self.squares[undo.from.0 as usize][undo.from.1 as usize] = Some(mover);

                // Restore the captured pawn to its captured square
                if let Some(captured_piece) = undo.captured {
                    self.squares[captured_at.0 as usize][captured_at.1 as usize] =
                        Some(captured_piece);
                }
            }

            MoveKind::Normal { promotion } => {
                // Moving piece currently sits at `to`
                let mut mover =
                    take_square(self, undo.to).ok_or_else(|| MoveError::NoAviailableMoves)?;

                // If this was a promotion, revert kind back to pawn
                if promotion.is_some() {
                    mover.kind = PieceType::Pawn;
                }

                mover.position = undo.from;
                mover.has_moved = false;
                self.squares[undo.from.0 as usize][undo.from.1 as usize] = Some(mover);

                // If a piece was captured on `to`, restore it
                if let Some(captured_piece) = undo.captured {
                    self.squares[undo.to.0 as usize][undo.to.1 as usize] = Some(captured_piece);
                }
            }
        }

        // Restore irreversible state captured in the undo record
        self.white_big_castle = undo.prev_state.white_big_castle;
        self.white_small_castle = undo.prev_state.white_small_castle;
        self.black_big_castle = undo.prev_state.black_big_castle;
        self.black_small_castle = undo.prev_state.black_small_castle;
        self.en_passant_target = undo.prev_state.en_passant_target;
        self.halfmove_clock = undo.prev_state.halfmove_clock;
        self.fullmove_number = undo.prev_state.fullmove_number;
        self.turn = undo.prev_state.turn;

        self.been_modified = true;
        self.rerender_move_cache();

        Ok(())
    }
}
#[tauri::command]
pub fn get_board_at_index(
    state: tauri::State<'_, Mutex<ServerState>>,
    move_index: isize,
) -> Option<SerializedAnalyzerController> {
    let mut state = state.lock().unwrap();

    let game_moves = &state.analyzer_controller.board.meta_data.move_list;

    if move_index == -1 {
        let mut starting_board =
            Board::from(&state.analyzer_controller.board.meta_data.starting_position);
        starting_board.meta_data = state.analyzer_controller.board.meta_data.clone();
        let anal = AnalyzerController {
            game_id: state.analyzer_controller.game_id,
            board: starting_board,
            current_ply: -1,
            board_undo: state.analyzer_controller.board_undo.clone(),
            last_threat: state.analyzer_controller.last_threat.clone(),
            last_pv: state.analyzer_controller.last_pv.clone(),
            chat_history: state.analyzer_controller.chat_history.clone(),
        };
        //println!("FEN at index -1: {}", anal.get_fen());
        let serialized_anal = serialize_analyzer_controller(&anal);
        return Some(serialized_anal);
    }

    if move_index < -1 || (move_index as usize) >= game_moves.len() {
        return None;
    }

    // If requesting the currently loaded ply, return it immediately
    if move_index as i32 == state.analyzer_controller.current_ply {
        let serialized_anal = serialize_analyzer_controller(&state.analyzer_controller);
        return Some(serialized_anal);
    }

    //compute move delta
    if move_index > state.analyzer_controller.current_ply as isize {
        println!("did new move update");
        //this means the is a forward move
        let mut starting_board = state.analyzer_controller.board.clone();
        let mut new_undo = state.analyzer_controller.board_undo.clone();
        // start from the next ply after current_ply (handles current_ply == -1 safely)
        let start_index = (state.analyzer_controller.current_ply + 1) as usize;
        for i in start_index..=(move_index as usize) {
            let current_move = &game_moves[i];
            if let Some((from, to, promotion)) = &starting_board.decode_uci_move(&current_move.uci)
            {
                match starting_board.move_piece_with_undo(*from, *to, *promotion) {
                    Ok(undo) => {
                        //println!("{:#?}", &undo);
                        new_undo.push(undo);
                    }
                    Err(_) => {
                        return None;
                    }
                }
            } else {
                return None;
            }
        }
        starting_board.meta_data = state.analyzer_controller.board.meta_data.clone();
        starting_board.rerender_move_cache();
        let anal = AnalyzerController {
            game_id: state.analyzer_controller.game_id,
            board: starting_board,
            current_ply: move_index as i32,
            board_undo: new_undo,
            last_threat: state.analyzer_controller.last_threat.clone(),
            last_pv: state.analyzer_controller.last_pv.clone(),
            chat_history: state.analyzer_controller.chat_history.clone(),
        };
        let serialized_anal = serialize_analyzer_controller(&anal);
        state.analyzer_controller = anal;
        //println!("FEN at index {}: {}", move_index, anal.get_fen());
        return Some(serialized_anal);
    } else if move_index < state.analyzer_controller.current_ply as isize {
        let mut starting_board = state.analyzer_controller.board.clone();
        let mut new_undo = state.analyzer_controller.board_undo.clone();
        let count = (state.analyzer_controller.current_ply as isize - move_index) as usize;
        for _ in 0..count {
            let current_undo = new_undo.pop();
            if let Some(undo) = current_undo {
                match starting_board.apply_undo(undo) {
                    Ok(_) => {}
                    Err(_) => {
                        return None;
                    }
                }
            } else {
                return None;
            }
        }
        println!("did old move update");
        starting_board.meta_data = state.analyzer_controller.board.meta_data.clone();
        starting_board.rerender_move_cache();
        let anal = AnalyzerController {
            game_id: state.analyzer_controller.game_id,
            board: starting_board,
            current_ply: move_index as i32,
            board_undo: new_undo,
            last_threat: state.analyzer_controller.last_threat.clone(),
            last_pv: state.analyzer_controller.last_pv.clone(),
            chat_history: state.analyzer_controller.chat_history.clone(),
        };
        state.analyzer_controller = anal.clone();
        //println!("FEN at index {}: {}", move_index, anal.get_fen());
        let serialized_anal = serialize_analyzer_controller(&anal);
        return Some(serialized_anal);
    }
    None
}
