use crate::engine::{Board, ChessPiece, PieceColor, PieceType};

impl Board{
    pub fn simulate_move(&self, piece: &ChessPiece, new_pos: &(u8, u8)) -> bool {
        // Clone the board to simulate the move
        let old_pos = piece.position;
        let mut board = self.clone();

        // Make the move on the cloned board
        board.squares[old_pos.0 as usize][old_pos.1 as usize] = None;
        // Create a new piece with updated position for accurate checking
        let mut moved_piece = *piece;
        moved_piece.position = *new_pos;
        board.squares[new_pos.0 as usize][new_pos.1 as usize] = Some(moved_piece);

        // Check if the king would NOT be in check after this move
        !board.is_in_check(piece.color)
     }
    pub fn is_in_check(&self, color: PieceColor) -> bool {
        // Find king position
        let mut king_pos = None;
        for (r, row) in self.squares.iter().enumerate() {
            for (c, square) in row.iter().enumerate() {
                if let Some(piece) = square {
                    if piece.kind == PieceType::King && piece.color == color {
                        king_pos = Some((r as u8, c as u8));
                        break;
                    }
                }
            }
            if king_pos.is_some() { break; }
        }
        
        if let Some(king_position) = king_pos {
            // Check if any enemy piece attacks the king square
            for row in &self.squares {
                for square in row {
                    if let Some(piece) = square {
                        if piece.color != color {
                            let attacks = self.get_attack_squares(piece);
                            if attacks.contains(&king_position) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
    
    
    pub fn is_chackmate(&mut self) -> bool{
        let mut mate =true;
        if !self.is_in_check(self.turn){
            return false;
        }else{
            let squares = self.squares.clone();
            for rank in squares{
                for file in rank{
                    match  file {
                        Some(piece) if piece.color == self.turn => {
                            let (q, c) = self.get_legal_moves(&piece);
                            if !q.is_empty() || !c.is_empty(){
                                mate = false;
                            }
                        }
                        Some(_) => {}
                        None => {}
                    }
                }
            }
        }
        mate
    }
    pub fn is_stale_mate(&mut self) -> bool{
        let  mut stale = true;        
        if self.is_in_check(self.turn){
            return false;
        }else {
            let squares = self.squares.clone();
            for rank in squares{
                for file in rank{
                    match  file {
                        Some(piece) if piece.color == self.turn => {
                            let (q, c) = self.get_legal_moves(&piece);
                            if !q.is_empty() || !c.is_empty(){
                                stale = false;
                            }
                        }
                        Some(_) => {}
                        None => {}
                    }
                }
            }
            
        }
        stale
    }
    pub fn has_lost(&mut self) -> bool{
        return self.is_chackmate() || self.is_stale_mate() || self.halfmove_clock == 50;
    }
}