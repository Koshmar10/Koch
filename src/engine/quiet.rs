use crate::engine::{Board, ChessPiece, PieceType};

impl Board{
pub fn filter_quiet_moves(&self, piece: &ChessPiece, moves:&Vec<(u8,u8)>) -> Vec<(u8,u8)>{
    moves.iter().filter(|pos| {
      match self.squares[pos.0 as usize][pos.1 as usize] {
        None => {
           if piece.kind == PieceType::Pawn{
               if piece.position.1 == pos.1 {
                true }
                else { false}
            } 
            else {true }
        
        },
        _ => false,
      }  
    }).cloned().collect()
}

pub fn legalize_quiet_moves(&self, piece: &ChessPiece, quiet_moves: Vec<(u8,u8)>) -> Vec<(u8,u8)>{
        let mut valid_quiet_moves: Vec<(u8,u8)> = Vec::new();
        for mv in quiet_moves {
            // simulate_move returns true if the king is safe after the move
            if self.simulate_move(piece, &mv){
                if piece.kind == PieceType::King {
                    if piece.position.1.abs_diff(mv.1) == 1 || piece.position.0.abs_diff(mv.0) == 1{
                        valid_quiet_moves.push(mv);
                    } 
                } else {
                    valid_quiet_moves.push(mv);   
                }
            }
        }
        valid_quiet_moves
    }
}