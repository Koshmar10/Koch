use std::collections::HashMap;

use crate::engine::{Board, PieceColor, PieceType};

impl Board {
    /// Decode a UCI move string into board coordinates (from, to).
    /// This function no longer mutates the board; caller should apply the move.
    pub fn decode_uci_move(
        &self,
        uci_move: &str,
    ) -> Option<((u8, u8), (u8, u8), Option<crate::engine::PieceType>)> {
        if uci_move.len() < 4 {
            return None;
        }

        // Mapping for algebraic file to board file index
        let file_map: std::collections::HashMap<char, u8> = [
            ('a', 0),
            ('b', 1),
            ('c', 2),
            ('d', 3),
            ('e', 4),
            ('f', 5),
            ('g', 6),
            ('h', 7),
        ]
        .iter()
        .cloned()
        .collect();

        let chars: Vec<char> = uci_move.chars().collect();

        // safe lookups for from/to squares
        let from_file = *file_map.get(&chars[0].to_ascii_lowercase())?;
        let from_rank_digit = chars[1].to_digit(10)? as u8;
        if !(1..=8).contains(&from_rank_digit) {
            return None;
        }
        let from_rank = 8u8 - from_rank_digit;

        let to_file = *file_map.get(&chars[2].to_ascii_lowercase())?;
        let to_rank_digit = chars[3].to_digit(10)? as u8;
        if !(1..=8).contains(&to_rank_digit) {
            return None;
        }
        let to_rank = 8u8 - to_rank_digit;

        // Parse optional promotion. Accept both "e7e8q" and "e7e8=Q"
        let mut promotion: Option<crate::engine::PieceType> = None;
        if chars.len() > 4 {
            let mut idx = 4;
            if chars[idx] == '=' {
                idx += 1;
            }
            if idx < chars.len() {
                promotion = match chars[idx].to_ascii_lowercase() {
                    'q' => Some(crate::engine::PieceType::Queen),
                    'r' => Some(crate::engine::PieceType::Rook),
                    'b' => Some(crate::engine::PieceType::Bishop),
                    'n' => Some(crate::engine::PieceType::Knight),
                    _ => None,
                };
            }
        } else {
            // Also detect promotion implicitly for pawns moving to last rank (in case UCI omitted promotion piece)
            if let Some(piece) = &self.squares[from_rank as usize][from_file as usize] {
                if piece.kind == crate::engine::PieceType::Pawn {
                    let is_promotion = (piece.color == crate::engine::PieceColor::White && to_rank == 0)
                        || (piece.color == crate::engine::PieceColor::Black && to_rank == 7);
                    if is_promotion {
                        promotion = Some(crate::engine::PieceType::Queen); // default to queen
                    }
                }
            }
        }

        let from = (from_rank, from_file);
        let to = (to_rank, to_file);

        Some((from, to, promotion))
    }

    /// Encodes a move from board coordinates to UCI format (e.g., "e2e4" or "a7a8q")
    /// (unchanged, but keep callers responsible for executing moves)
    pub fn encode_uci_move(
        &self,
        from: (u8, u8),
        to: (u8, u8),
        promotion: Option<crate::engine::PieceType>,
    ) -> String {
        let file_chars = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        let from_file = file_chars[from.1 as usize];
        let from_rank = 8u8 - from.0; // 0 -> 8, 7 -> 1
        let to_file = file_chars[to.1 as usize];
        let to_rank = 8u8 - to.0; // 0 -> 8, 7 -> 1
        let mut uci = format!("{}{}{}{}", from_file, from_rank, to_file, to_rank);
        if let Some(piece_type) = promotion {
            let pc = match piece_type {
                crate::engine::PieceType::Queen => 'q',
                crate::engine::PieceType::Rook => 'r',
                crate::engine::PieceType::Bishop => 'b',
                crate::engine::PieceType::Knight => 'n',
                _ => 'q',
            };
            uci.push(pc);
        } else if let Some(piece) = &self.squares[from.0 as usize][from.1 as usize] {
            if piece.kind == crate::engine::PieceType::Pawn {
                let is_promotion = (piece.color == crate::engine::PieceColor::White && to.0 == 0)
                    || (piece.color == crate::engine::PieceColor::Black && to.0 == 7);
                if is_promotion {
                    uci.push('q');
                }
            }
        }
        uci
    }
}
