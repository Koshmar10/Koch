
use crate::engine::{board::{BoardMetaData, BoardState}, Board, ChessPiece, PieceColor, PieceType};

pub enum FenError{
    InvalidChar(char)
}

pub fn fen_parser(fen: &String) -> Result<Board, FenError>{
    // Implementation goes here
    let mut pieces = Vec::<ChessPiece>::new();
    // split into exactly six fields
    let parts: Vec<&str> = fen.split_whitespace().collect();
    let board_representation    = parts[0];
    let to_move                 = parts[1];
    let castling_rights         = parts[2];
    let en_passant_targets      = parts[3];
    let halfmove_clock: u32     = parts[4].parse().unwrap();
    let fullmove_number: u32    = parts[5].parse().unwrap();
    let fen_files : Vec<&str> = board_representation.split("/").collect();
    let mut i: u8 = 0;
    for file in fen_files {
        let mut j: u8 =0;
        for elem in file.chars(){
            if elem.is_numeric(){
                j+=elem.to_digit(10).unwrap() as u8;

            }
            else{
                let (kind, color, pos, move_count) = match elem {
                    'r' => {(PieceType::Rook, PieceColor::Black, (i, j), 0)}
                    'n' => {(PieceType::Knight, PieceColor::Black, (i, j), 0)}
                    'b' => {(PieceType::Bishop, PieceColor::Black, (i, j), 0)}
                    'k' => {(PieceType::King, PieceColor::Black, (i, j), 0)}
                    'q' => {(PieceType::Queen, PieceColor::Black, (i, j), 0)}
                    'p' => {(PieceType::Pawn, PieceColor::Black, (i, j), 0)}
                    'R' => {(PieceType::Rook, PieceColor::White, (i, j), 0)}
                    'N' => {(PieceType::Knight, PieceColor::White, (i, j), 0)}
                    'B' => {(PieceType::Bishop, PieceColor::White, (i, j), 0)}
                    'K' => {(PieceType::King, PieceColor::White, (i, j), 0)}
                    'Q' => {(PieceType::Queen, PieceColor::White, (i, j), 0)}
                    'P' => {(PieceType::Pawn, PieceColor::White, (i, j), 0)}
                    c => return Err(FenError::InvalidChar(c))
                };
                pieces.push(
                    ChessPiece { 
                        kind, 
                        color, 
                        position: pos, 
                        times_moved: move_count }
                );
                j+=1;
            }
        }
        i+=1;
    }
    let mut board: [[Option<ChessPiece>; 8]; 8] = [[None; 8]; 8];
    for piece in pieces {
        let (x, y) = piece.position;
        board[x as usize][y as usize] = Some(piece);
    }

    let en_passant_target = {
        if en_passant_targets == "-" {
            None
        } else {
            // e.g. "e3" → file 'e', rank '3'
            let bytes = en_passant_targets.as_bytes();
            let file = bytes[0].wrapping_sub(b'a');          // 'a' → 0, …, 'h' → 7
            let rank_digit = bytes[1].wrapping_sub(b'0');    // '1' → 1, …, '8' → 8
            let row = 8 - rank_digit;                        // convert chess‐rank to 0–7
            Some((row as u8, file as u8))
        }
    };
    Ok(Board { squares: board, turn: 
        if to_move == "w" {PieceColor::White} else {PieceColor::Black}, 
        white_big_castle: if castling_rights.contains("Q") {true} else {false}, 
        black_big_castle: if castling_rights.contains("q") {true} else {false},
        white_small_castle: if castling_rights.contains("K") {true} else {false}, 
        black_small_castle: if castling_rights.contains("k") {true} else {false}, 
        halfmove_clock,
        fullmove_number,
        en_passant_target: en_passant_target,
        state: BoardState::default(),
        meta_data: BoardMetaData::default(),
    })
}

impl ToString for Board {
    fn to_string(&self) -> String {
        let mut board_string = "".to_owned();
        let to_move: &str = if self.turn ==PieceColor::White {"w"} else {"b"};
        let mut castleing_rights = 
            (if self.white_small_castle {"K"} else {""}).to_string() +
            if self.white_big_castle {"Q"} else {""} +
            if self.black_small_castle {"k"} else {""} +
            if self.black_big_castle {"q"} else {""};
        if castleing_rights == ""{castleing_rights = "-".to_owned()}
        for i in 0..8 {
            let mut empty_squares = 0;
            for j in 0..8 {
                let piece = &self.squares[i as usize][j as usize];
                match &piece{
                    Some(p) => {
                        if empty_squares != 0 {board_string+= &format!("{}", empty_squares)};
                        board_string += match (p.kind, p.color) {
                            (PieceType::King, PieceColor::White) => "K",
                            (PieceType::Queen, PieceColor::White) => "Q",
                            (PieceType::Rook, PieceColor::White) => "R",
                            (PieceType::Knight, PieceColor::White) => "N",
                            (PieceType::Bishop, PieceColor::White) => "B",
                            (PieceType::Pawn, PieceColor::White) => "P",
                            (PieceType::King, PieceColor::Black) => "k",
                            (PieceType::Queen, PieceColor::Black) => "q",
                            (PieceType::Rook, PieceColor::Black) => "r",
                            (PieceType::Knight, PieceColor::Black) => "n",
                            (PieceType::Bishop, PieceColor::Black) => "b",
                            (PieceType::Pawn, PieceColor::Black) => "p",

                            
                        };
                        empty_squares=0;
                    }
                    None => {empty_squares+=1;}
                }
            }
            if empty_squares !=0 {
                board_string+=&empty_squares.to_string();
            }
            board_string+="/";

        }
        board_string = board_string.trim_end_matches("/").to_string();
        let en_passant = "-"; // placeholder if no en-passant target
        return board_string
            + " "
            + &to_move
            + " "
            + &castleing_rights
            + " "
            + en_passant
            + " "
            + &self.halfmove_clock.to_string()
            + " "
            + &self.fullmove_number.to_string();
    }
}