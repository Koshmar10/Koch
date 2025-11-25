use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub enum PieceType {
    Pawn,
    Rook,
    Queen,
    Bishop,
    Knight,
    King,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TS, serde::Serialize, Deserialize)]
#[ts(export)]
pub enum PieceColor {
    White,
    Black,
}
#[derive(Clone, Copy, Debug, TS, serde::Serialize, Deserialize)]
#[ts(export)]
pub struct ChessPiece {
    pub id: u32,
    pub kind: PieceType,
    pub color: PieceColor,
    pub position: (u8, u8),
    pub has_moved: bool,
}

impl Default for ChessPiece {
    fn default() -> Self {
        Self {
            id: 0,
            kind: PieceType::Pawn,
            color: PieceColor::Black,
            position: (0, 0),
            has_moved: false,
        }
    }
}
impl ToString for PieceColor {
    fn to_string(&self) -> String {
        match self {
            PieceColor::White => "White".to_owned(),
            PieceColor::Black => "Black".to_owned(),
        }
    }
}
impl ToString for PieceType {
    fn to_string(&self) -> String {
        match self {
            PieceType::King => "King".to_owned(),
            PieceType::Queen => "Queen".to_owned(),
            PieceType::Bishop => "Bishop".to_owned(),
            PieceType::Knight => "Knight".to_owned(),
            PieceType::Pawn => "Pawn".to_owned(),
            PieceType::Rook => "Rook".to_owned(),
        }
    }
}
