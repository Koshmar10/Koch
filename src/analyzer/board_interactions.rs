
use crate::engine::{Board, PieceType};
use std::{char, error::Error};

pub enum AnalyzerWindowOprion{ Emtpy, HeatMap}

pub struct AnalyzerController{
    pub current_ply: u32,
    pub window_mode: AnalyzerWindowOprion,

}
impl Default for AnalyzerController {
    fn default() -> Self {
        Self {
            current_ply: 0,
            window_mode: AnalyzerWindowOprion::Emtpy,
        }
    }
}

impl Board{
    pub fn do_move(&mut self, uci: String) -> Result<(), Box<dyn Error>> {
        
        let promotion: Option<PieceType> = if uci.len() == 5{
            match uci.chars().nth(4) {
                Some(char) => {
                    match char {
                        'q' => {Some(PieceType::Queen)}
                        'b' => { Some(PieceType::Bishop)}
                        'n' => { Some(PieceType::Knight)}
                        'r' => { Some(PieceType::Rook)}
                        _ => { None}
                    }
                }
                None => {None}
            }
        } else { None};
        
        let decoded = self.decode_uci_move(uci);
        let (from,to) = match decoded {
            Some(x) => x,
            None => return Err("bad uci".into()),
        };
        
        let mut piece = match self.squares[from.0 as usize][from.1 as usize]{
            Some(piece) => piece,
            None => return Err("no piece on from square".into()),
        };
        match promotion {
            Some(pro) => piece.kind = pro,
            None =>{}
        }
        self.squares[from.0 as usize][from.1 as usize]= None;
        self.squares[to.0 as usize][to.1 as usize] = Some(piece);
        Ok(())
    }   
    pub fn undo_move(&mut self, uci: String) -> Result<(), Box<dyn Error>>{    
        println!("{}", uci.clone());    
        let decoded = self.decode_uci_move(uci.clone());
        let (from,to) = match decoded {
            Some(x) => x,
            None => {println!("bad uci {}", uci.clone()); return Err("bad uci".into())},
        };
        
        let mut piece = match self.squares[to.0 as usize][to.1 as usize]{
            Some(piece) => piece,
            None =>{println!("no piece square"); return Err("no piece on from square".into())},
        };
        if uci.len() == 5 {piece.kind  = PieceType::Pawn;}
        self.squares[from.0 as usize][from.1 as usize]= Some(piece);
        self.squares[to.0 as usize][to.1 as usize] =None;
        Ok(())
    } 
}