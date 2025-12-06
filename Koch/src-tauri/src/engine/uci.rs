use std::collections::HashMap;

use crate::engine::{Board, PieceColor, PieceType};

impl Board {
    pub fn decode_uci_move(&mut self, uci_move: String) -> Option<((u8, u8), (u8, u8))> {
        let uci_move = uci_move.as_str();

        // Mapping for algebraic notation to board coordinates
        let file_map: HashMap<char, u8> = [
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

        // Parse standard coordinates from UCI move
        let chars: Vec<char> = uci_move.chars().collect();
        let from_file = file_map[&chars[0]];
        let from_rank = 8 - (chars[1].to_digit(10).unwrap() as u8);
        let to_file = file_map[&chars[2]];
        let to_rank = 8 - (chars[3].to_digit(10).unwrap() as u8);
        let from = (from_rank, from_file);
        let to = (to_rank, to_file);

        // 1) Handle promotions: e.g. "e7e8q"
        if uci_move.len() == 5 {
            // Execute the move first
            let result = self.move_piece(from, to);
            if result.is_err() {
                return None;
            }

            // Then handle the promotion
            let promotion_char = chars[4];
            let promotion_piece = match promotion_char {
                'q' | 'Q' => PieceType::Queen,
                'r' | 'R' => PieceType::Rook,
                'b' | 'B' => PieceType::Bishop,
                'n' | 'N' => PieceType::Knight,
                _ => panic!("Invalid promotion piece: {}", promotion_char),
            };

            // Apply the promotion
            self.promote_pawn(to, promotion_piece);

            // Mark that promotion is handled
            self.ui.promtion_pending = None;

            return Some((from, to));
        }

        // 2) Handle castling (no changes needed to your castling code)
        match uci_move {
            "e1g1" => Some(((7, 4), (7, 6))),
            "e1c1" => Some(((7, 4), (7, 2))),
            "e8g8" => Some(((0, 4), (0, 6))),
            "e8c8" => Some(((0, 4), (0, 2))),
            // 3) Normal move (no changes needed)
            _ => Some((from, to)),
        }
    }

    /// Encodes a move from board coordinates to UCI format (e.g., "e2e4" or "a7a8q")
    pub fn encode_uci_move(
        &self,
        from: (u8, u8),
        to: (u8, u8),
        promotion: Option<PieceType>,
    ) -> String {
        // Map board coordinates to algebraic notation
        let file_chars = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

        // Convert from coordinates (rank, file) to algebraic
        let from_file = file_chars[from.1 as usize];
        let from_rank = 8 - from.0; // 0 -> 8, 7 -> 1

        // Convert to coordinates to algebraic
        let to_file = file_chars[to.1 as usize];
        let to_rank = 8 - to.0; // 0 -> 8, 7 -> 1

        // Start building the UCI move string
        let mut uci = format!("{}{}{}{}", from_file, from_rank, to_file, to_rank);

        // Check if this is a promotion move
        if let Some(piece_type) = promotion {
            // Get the promotion character and append it
            let promotion_char = match piece_type {
                PieceType::Queen => 'q',
                PieceType::Rook => 'r',
                PieceType::Bishop => 'b',
                PieceType::Knight => 'n',
                _ => panic!("Invalid promotion piece type"),
            };
            uci.push(promotion_char);
        } else if let Some(piece) = &self.squares[from.0 as usize][from.1 as usize] {
            // Auto-detect promotion if we're moving a pawn to the end rank
            if piece.kind == PieceType::Pawn {
                let is_promotion = (piece.color == PieceColor::White && to.0 == 0)
                    || (piece.color == PieceColor::Black && to.0 == 7);

                if is_promotion {
                    // Default to queen promotion if not specified
                    uci.push('q');
                }
            }
        }

        uci
    }
}
