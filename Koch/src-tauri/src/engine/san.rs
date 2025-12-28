use crate::engine::{Board, PieceType};

impl Board {
    pub fn encode_san_move(&self, from: (u8, u8), to: (u8, u8), promotion: Option<PieceType>) {}
}
