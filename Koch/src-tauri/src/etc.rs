use crate::engine::PieceColor;


// make sure this file is included by your lib.rs/bin.rs via `mod etc;`
pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const DEFAULT_STARTING: PieceColor = PieceColor::White;
pub const PLAYER_NAME: &str = "Koshmar";
pub const STOCKFISH_ELO: u32 = 2500;
