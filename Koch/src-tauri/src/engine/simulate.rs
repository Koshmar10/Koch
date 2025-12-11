use crate::engine::{Board, ChessPiece, PieceColor, PieceType};

impl Board {
    pub fn simulate_move(&self, piece: &ChessPiece, new_pos: &(u8, u8)) -> bool {
        // Minimize cloning: copy only squares we touch
        let old_pos = piece.position;
        let mut board = self.clone(); // consider a "light clone" if you add bitboards later

        let is_en_passant = piece.kind == PieceType::Pawn
            && old_pos.1 != new_pos.1
            && board.squares[new_pos.0 as usize][new_pos.1 as usize].is_none()
            && board.en_passant_target == Some(*new_pos);

        // Remove from old square
        board.squares[old_pos.0 as usize][old_pos.1 as usize] = None;

        // Update piece position (no reallocation)
        let mut moved_piece = *piece;
        moved_piece.position = *new_pos;

        if is_en_passant {
            // Use precomputed direction
            let dir = if piece.color == PieceColor::White {
                1i8
            } else {
                -1i8
            };
            let captured_r = (new_pos.0 as i8 + dir) as usize;
            board.squares[captured_r][new_pos.1 as usize] = None;
        }

        // Place on new square
        board.squares[new_pos.0 as usize][new_pos.1 as usize] = Some(moved_piece);

        // Rely on get_attack_squares for check detection
        !board.is_in_check(piece.color)
    }

    pub fn is_in_check(&self, color: PieceColor) -> bool {
        // Early exit by tracking king square once
        let mut king_pos: Option<(u8, u8)> = None;
        'outer: for (r, row) in self.squares.iter().enumerate() {
            for (c, square) in row.iter().enumerate() {
                if let Some(k) = square {
                    if k.kind == PieceType::King && k.color == color {
                        king_pos = Some((r as u8, c as u8));
                        break 'outer;
                    }
                }
            }
        }
        let Some(king_position) = king_pos else {
            return false;
        };

        // Short-circuit on first attack hit; avoid allocations where possible
        for row in &self.squares {
            for square in row {
                if let Some(p) = square {
                    if p.color != color {
                        // get_attack_squares must be fast; if it allocates, consider switching to iter + callback later
                        let attacks = self.get_attack_squares(p);
                        if attacks.contains(&king_position) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn is_checkmate(&mut self) -> bool {
        // spelling fix from is_chackmate
        if !self.is_in_check(self.turn) {
            return false;
        }
        let squares = &self.squares;
        for rank in squares {
            for file in rank {
                if let Some(piece) = file {
                    if piece.color == self.turn {
                        let (q, c) = self.get_legal_moves(piece);
                        if !q.is_empty() || !c.is_empty() {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    pub fn is_stalemate(&mut self) -> bool {
        if self.is_in_check(self.turn) {
            return false;
        }
        let squares = &self.squares;
        for rank in squares {
            for file in rank {
                if let Some(piece) = file {
                    if piece.color == self.turn {
                        let (q, c) = self.get_legal_moves(piece);
                        if !q.is_empty() || !c.is_empty() {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    pub fn has_lost(&mut self) -> bool {
        // route through corrected names
        self.is_checkmate() || self.is_stalemate() || self.halfmove_clock == 50
    }
}
