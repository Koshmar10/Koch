use std::{error::Error, path::Display};

use crate::{engine::{fen::fen_parser, move_gen::MoveError, ChessPiece, PieceColor, PieceType}, etc::{DEFAULT_FEN, DEFAULT_STARTING}, game::controller::LostBy};
use chrono::Local;

#[derive(Clone)]
pub struct Board{
    pub squares: [[Option<ChessPiece>; 8]; 8],
    pub turn: PieceColor,
    pub white_big_castle: bool,
    pub black_big_castle: bool,
    pub white_small_castle: bool,
    pub black_small_castle: bool,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub en_passant_target: Option<(u8,u8)>,
    pub state: BoardState,
    pub meta_data: BoardMetaData,

}
pub struct MoveInfo{
    pub old_pos: (u8, u8),
    pub new_pos: (u8, u8), 
    pub promotion: Option<PieceType>,
    pub is_capture: bool,
}

#[derive(Clone)]
pub struct BoardState {
    pub selected_piece: Option<ChessPiece>,
    pub moved_from: Option<(u8, u8)>,
    pub moved_to: Option<(u8, u8)>,
    pub quiet_moves: Option<Vec<(u8, u8)>>,
    pub capture_moves: Option<Vec<(u8, u8)>>,
    pub pov: PieceColor,
    pub white_taken:     Vec<ChessPiece>,
    pub black_taken:     Vec<ChessPiece>,
    pub promtion_pending: Option<((u8, u8), (u8,u8))>,
    pub checkmate_square: Option<(u8, u8)>,
    pub past_evaluation: f32, 
    pub current_evaluation: f32,

}
#[derive(Clone)]
pub enum GameResult {WhiteWin, BlackWin, Draw, Unfinished}
#[derive(Clone)]
pub struct BoardMetaData{
    pub starting_position: String,
    pub date: String,
    pub move_list: Vec<MoveStruct>,
    pub termination: LostBy,
    pub result: GameResult,
    pub white_player_elo: u32,
    pub black_player_elo: u32,
    pub white_player_name: String,
    pub black_player_name: String,
    

}
#[derive(Clone)]
pub struct MoveStruct{
    pub move_number: usize,
    pub san: String,
    pub uci: String,
    pub from: String,
    pub to: String,
    pub promotion: Option<PieceType>,
    pub is_capture:bool,
    pub evaluation: f32,
    pub time_stamp: f32,
}

#[derive(Clone)]
pub enum CastleType {QueenSide, KingSide}
    

impl Default for Board{
    fn default() -> Self {
    match fen_parser(&DEFAULT_FEN.to_owned()){
        Ok(board) => return board,
        Err( e) => return Board{
            squares: [[None; 8]; 8],
            turn: PieceColor::White,
            white_big_castle: true,
            white_small_castle: true,
            black_big_castle: true, 
            black_small_castle: true, 
            halfmove_clock: 0,
            fullmove_number: 1,
            en_passant_target: None,
            state:BoardState::default(),
            meta_data:BoardMetaData::default(),

        },   
    }
}      
}
impl Default for BoardState{
    fn default() -> Self {
        Self{
            selected_piece: None,
            quiet_moves: None,
            capture_moves: None, 
            pov: DEFAULT_STARTING,
            moved_from: None,
            moved_to: None,
            white_taken: Vec::new(),
            black_taken: Vec::new(),
            promtion_pending: None,
            checkmate_square: None,
            past_evaluation: 0.0,
            current_evaluation:0.0,
        }
    }
}
impl Default for BoardMetaData {
    fn default() -> Self {
        Self{
            starting_position: DEFAULT_FEN.to_string(),
            date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            move_list: Vec::new(),
            termination: LostBy::Draw,
            result: GameResult::Unfinished,
            white_player_elo: 0,
            black_player_elo: 0,
            white_player_name: String::new(),
            black_player_name: String::new(),
        }
    }
        }
  
impl Default for MoveStruct {
    fn default() -> Self {
        MoveStruct {
            move_number: 0,
            san: String::new(),
            uci: String::new(),
            from: String::new(),
            to: String::new(),
            promotion: None,
            is_capture: false,
            evaluation: 0.0,
            time_stamp: 0.0,
        }
    }
}

impl From<&str> for MoveStruct {
    fn from(uci: &str) -> Self {
        let mut mv = MoveStruct::default();
        mv.uci = uci.to_string();
        mv.from = uci.get(0..2).unwrap_or_default().to_string();
        mv.to = uci.get(2..4).unwrap_or_default().to_string();
        if let Some(p) = uci.chars().nth(4) {
            mv.promotion = Some(match p.to_ascii_lowercase() {
                'q' => PieceType::Queen,
                'r' => PieceType::Rook,
                'b' => PieceType::Bishop,
                'n' => PieceType::Knight,
                _ => return mv,
            });
        }
        mv
    }
}

impl From<&String> for Board{
    fn from(fen: &String) -> Self {
        match fen_parser(fen){
            Ok(b) => b,
            Err(e) => Board::default(),
        }
    }
}
impl Board {
    pub fn move_piece(&mut self, old_pos:(u8,u8), new_pos: (u8, u8)) -> Result<MoveInfo, MoveError> {
        let (old_rank, old_file) = old_pos;
        let (new_rank, new_file) = new_pos;
        
        
        // Clone the piece to compute legal moves without holding a mutable borrow on self
        let piece = match self.squares[old_rank as usize][old_file as usize].as_ref() {
            Some(p) => p.clone(),
            None => return Err(MoveError::IllegalMove),
        };
        let (quiets, captures) = self.get_legal_moves(&piece);
        let capture_piece = self.squares[new_rank as usize][new_file as usize].clone();
        // Now borrow the moving piece mutably to apply the move
        match self.squares[old_rank as usize][old_file as usize].as_mut() {
            Some(moving_piece) => {

                
                println!("Quiet moves: {:?}", quiets);
                println!("Capture moves: {:?}", captures);
                let is_quiet = quiets.contains(&new_pos);
                let is_capture = captures.contains(&new_pos);
                if is_capture {
                    self.halfmove_clock = 0;
                    //first we check for en passant because is a special capture
                    match moving_piece.kind {
                        PieceType::Pawn => {
                            //pawn capture decide
                            match moving_piece.color {
                                PieceColor::Black => {
                                    if new_pos.0 == 7 {self.state.promtion_pending = Some((new_pos, old_pos));}
                                }
                                PieceColor::White => {
                                    
                                    if new_pos.0 == 0 {self.state.promtion_pending = Some((new_pos, old_pos));}
                                }
                            }

                            match capture_piece {
                                Some(_) => {
                                    //normal capture
                                    moving_piece.times_moved+=1;
                                    moving_piece.position = new_pos;
                                    self.state.moved_to = Some(moving_piece.position);
                                    self.squares[new_rank as usize][new_file as usize] = Some(*moving_piece);
                                    self.squares[old_rank as usize][old_file as usize] = None;
                                    let capture = capture_piece.unwrap();
                                    match capture.color {
                                        PieceColor::Black => {
                                            self.state.white_taken.push(capture);
                                        }
                                        PieceColor::White => {
                                            self.state.black_taken.push(capture);
                                        }
                                    }
                                    self.en_passant_target = None;
                                    if !self.state.promtion_pending.is_some() {
                                        self.change_turn();
                                        self.deselect_piece();
                                        return Ok(MoveInfo {
                                            old_pos: (old_rank, old_file),
                                            new_pos: (new_rank, new_file),
                                            promotion: None,
                                            is_capture: true,
                                        });
                                    }
                                    self.change_turn();
                                    self.deselect_piece();
                                }
                                None => {
                                    //en passant
                                    moving_piece.times_moved+=1;
                                    moving_piece.position = new_pos;
                                    self.state.moved_to = Some(moving_piece.position);
                                    
                                    self.squares[new_rank as usize][new_file as usize] = Some(*moving_piece);
                                    self.squares[old_rank as usize][old_file as usize] = None;
                                    let (epr, epf) = self.en_passant_target.unwrap();
                                    let capture = self.squares[epr as usize][epf as usize].unwrap();
                                    match capture.color {
                                        PieceColor::Black => {
                                            self.state.white_taken.push(capture);
                                        }
                                        PieceColor::White => {
                                            self.state.black_taken.push(capture);
                                        }
                                    }
                                    self.squares[epr as usize][epf as usize]=None;
                                    self.en_passant_target = None;
                                    if !self.state.promtion_pending.is_some() {
                                        self.change_turn();
                                        self.deselect_piece();
                                        return Ok(MoveInfo {
                                            old_pos: (old_rank, old_file),
                                            new_pos: (new_rank, new_file),
                                            promotion: None,
                                            is_capture: true,
                                        });
                                    }
                                    self.change_turn();
                                    self.deselect_piece();
                                }
                            }
                            
                        }
                        _ =>{
                            moving_piece.times_moved+=1;
                            moving_piece.position = new_pos;
                            self.state.moved_to = Some(moving_piece.position);
                            self.squares[new_rank as usize][new_file as usize] = Some(*moving_piece);
                            self.squares[old_rank as usize][old_file as usize] = None;
                            
                            self.en_passant_target = None;
                            let capture = capture_piece.unwrap();
                            match capture.color {
                                PieceColor::Black => {
                                    self.state.white_taken.push(capture);
                                }
                                PieceColor::White => {
                                    self.state.black_taken.push(capture);
                                }
                            }
                            self.change_turn();
                            self.deselect_piece();
                            return Ok(MoveInfo {
                                old_pos: (old_rank, old_file),
                                new_pos: (new_rank, new_file),
                                promotion: None,
                                is_capture: true,
                            });
                        }
                    }
                    
                }else 
                if is_quiet  {
                    self.en_passant_target = None;
                    if moving_piece.kind == PieceType::Pawn {
                        let mv_delta = old_rank.abs_diff(new_rank);
                        if mv_delta == 2 {
                            //set en ppassant target 
                            self.en_passant_target = Some(new_pos);
                        }
                        
                    }
                    moving_piece.times_moved+=1;
                    moving_piece.position = new_pos;
                    self.state.moved_to = Some(moving_piece.position);
                    self.squares[new_rank as usize][new_file as usize] = Some(*moving_piece);
                    self.squares[old_rank as usize][old_file as usize] = None;

                    if piece.color == PieceColor::Black {
                        self.fullmove_number+=1;
                    }
                    if piece.kind == PieceType::Pawn {
                        self.halfmove_clock = 0;
                    }
                                   
                    self.change_turn();
                    self.deselect_piece();
                    
                    return Ok(MoveInfo {
                        old_pos: (old_rank, old_file),
                        new_pos: (new_rank, new_file),
                        promotion: None,
                        is_capture: false,
                    });
                }
                else {
                    return Err(MoveError::IllegalMove);
                }
                
            }

            None =>{
                return Err(MoveError::IllegalMove);
            }
        }
        
        // This return is for cases where we have promotion pending
        // or other special cases that don't return earlier
        Ok(MoveInfo {
            old_pos: (old_rank, old_file),
            new_pos: (new_rank, new_file),
            promotion: None,
            is_capture: captures.contains(&new_pos),
        })
    }
    
    pub fn change_turn(&mut self){
        self.turn = match self.turn {
            PieceColor::Black => PieceColor::White,
            PieceColor::White => PieceColor::Black,
        };
    }
    
    pub fn execute_castle(&mut self, king_pos:(u8,u8), rook_pos: (u8,u8))-> Result<(), Box<dyn Error>>{
        //get mut king rook
        let mut king = self.squares[king_pos.0 as usize][king_pos.1 as usize].unwrap();
        let mut rook: ChessPiece = self.squares[rook_pos.0 as usize][rook_pos.1 as usize].unwrap();

        //determine if is big castleif piece.kind == PieceType::Rook || piece.kind == PieceType::King {
        let king_rook_distance = king.position.1.abs_diff(rook.position.1);
        let castle_type = if king_rook_distance == 4 { CastleType::QueenSide} else {CastleType::KingSide};       
        //befora castle we need to check that all the squares between rook and king ar not in check
        if self.can_castle(castle_type.clone(), king.color){

            match castle_type {
                CastleType::KingSide => {
                    king.position.1 = (king.position.1 as i8 + 2) as u8;
                    rook.position.1 = (rook.position.1 as i8 + -2) as u8;
                    
                }
                CastleType::QueenSide => {
                    king.position.1 = (king.position.1 as i8 + -2) as u8;
                    rook.position.1 = (rook.position.1 as i8 + 3) as u8;
                }
                
            };
            
            self.state.moved_to = Some(king.position);
            //update_ing board
        
            self.squares[king_pos.0 as usize][king_pos.1 as usize] = None;
            self.squares[rook_pos.0 as usize][rook_pos.1 as usize] = None;
            
            self.squares[king.position.0 as usize][king.position.1 as usize] = Some(king);
            self.squares[rook.position.0 as usize][rook.position.1 as usize] = Some(rook); 
        
        match king.color {
            PieceColor::Black => {
                self.black_big_castle = false;  
                self.black_small_castle = false;
            }
            PieceColor::White => {
                self.white_big_castle = false;
                self.white_small_castle = false;
            }
            
            
            
        }
        self.change_turn();
        self.deselect_piece();
        return Ok(());
    
    }
    else {
        return Err("Cannot castle".into());
    }


    }
    pub fn promote_pawn(&mut self, pos: (u8,u8), kind: PieceType){
        if let Some(pawn) = self.squares[pos.0 as usize][pos.1 as usize].as_mut() {
            pawn.kind = kind;
        }
    }
    pub fn select_piece(&mut self, piece: ChessPiece) {
        self.state.selected_piece = Some(piece);
        self.set_legal_moves(&piece);
    }
    pub fn deselect_piece(&mut self) {
        self.state.capture_moves = None;
        self.state.selected_piece = None;
        self.state.quiet_moves = None;
    }
    pub fn record_move(&mut self, from: (u8, u8), to: (u8, u8), promotion: Option<PieceType>, is_capture: bool, eval_score: f32) {
        let uci = self.encode_uci_move(from, to, promotion);
        
        // Convert board coordinates to algebraic notation
        let file_chars = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        let from_file = file_chars[from.1 as usize];
        let from_rank = 8 - from.0;
        let to_file = file_chars[to.1 as usize];
        let to_rank = 8 - to.0;
        let mut move_record = MoveStruct {
            move_number: self.fullmove_number as usize,
            uci: uci,
            from: format!("{}{}", from_file, from_rank),
            to: format!("{}{}", to_file, to_rank),
            promotion: promotion,
            is_capture: is_capture,
            evaluation: eval_score,
            time_stamp: 0.0,
            san: "".to_string(), // Could use std::time::SystemTime if needed
        };
        
        // For SAN notation you would need more complex logic
        // This is a simplified version that just shows the piece movement
        if let Some(piece) = self.squares[to.0 as usize][to.1 as usize].as_ref() {
            let piece_letter = match piece.kind {
                PieceType::Pawn => "",
                PieceType::Knight => "N",
                PieceType::Bishop => "B",
                PieceType::Rook => "R",
                PieceType::Queen => "Q",
                PieceType::King => "K",
            };
            
            let capture_symbol = if is_capture { "x" } else { "" };
            
            move_record.san = format!(
                "{}{}{}{}{}",
                piece_letter,
                move_record.from,
                capture_symbol,
                move_record.to,
                promotion.map_or("".to_string(), |p| match p {
                    PieceType::Queen => "=Q",
                    PieceType::Rook => "=R",
                    PieceType::Bishop => "=B",
                    PieceType::Knight => "=N",
                    _ => "",
                }.to_string())
            );
        }
        
        self.meta_data.move_list.push(move_record);
    }
    
}