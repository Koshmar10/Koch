use std::sync::mpsc;

use eframe::egui::{self, Response};

use crate::{engine::{board::{CastleType, MoveInfo, MoveStruct}, ChessPiece, PieceType}, game::{controller::GameMode, evaluator::{EvalKind, EvalResponse}, stockfish_engine::{StockfishCmd, StockfishResult}}, ui::app::{MyApp, PopupType}};


impl MyApp{
    pub fn handle_board_interaction_logic(&mut self,  square :&(u8, u8), response :&Response) {
    match self.game.mode {
        GameMode::PvE => {
            if !self.game.game_over{
                if self.game.player != self.board.turn
                {
                    //wait for stockfish
                      
                } else {
        
                    if response.secondary_clicked() {
                        self.board.deselect_piece();
                    }
                if response.clicked() {
                    if self.board.ui.promtion_pending.is_some(){
                    //we need to handle promotion before doing anything lese
                    
                } else {
                    let old_piece: Option<ChessPiece> = self.board.ui.selected_piece;
                let new_piece: Option<ChessPiece> = self.board.squares[square.0 as usize][square.1 as usize];
                
                let is_selected_piece: bool = old_piece.is_some();
                let has_piece: bool = new_piece.is_some();
                 //CASE 1: there is no selected piece so we must set the seleced piece to the square we interact with
                if !is_selected_piece && has_piece {
                    //get the piece from the square
                    
                    let piece: ChessPiece = self.board.squares[square.0 as usize][square.1 as usize].unwrap(); 
                    self.board.select_piece(piece);
                    
                    //checking if the piece we try to select matches the player s turn
                }
                if !is_selected_piece && has_piece {
                    //get the piece from the square
                    if let Some(piece) = new_piece {
                        //checking if the piece we try to select matches the player's turn
                        if piece.color == self.board.turn {
                            //We select the new piece
                            self.board.select_piece(piece);   
                        }
                    }
                }
                if is_selected_piece && !has_piece {
                    // movement performed
                    if let Some(piece) = old_piece {
                        if let Ok(mut ms) = self.board.move_piece(piece.position, *square) {
                            // ensure UCI is set
                            if ms.uci.is_empty() {
                                ms.uci = self.board.encode_uci_move(piece.position, *square, None);
                            }
                            // push the move now (don't wait for eval)
                            self.board.meta_data.move_list.push(ms.clone());

                            // optionally kick off eval (non-blocking)
                            let (tx, rx) = mpsc::channel::<EvalResponse>();
                            self.evaluator
                                .send_eval_request(self.board.to_string(), EvalKind::MoveEval { reply_to: tx });
                            // try to update evaluation if ready this frame
                            if let Ok(resp) = rx.try_recv() {
                                if let Some(last) = self.board.meta_data.move_list.last_mut() {
                                    last.evaluation = resp;
                                    self.board.ui.bar_eval = last.evaluation.value;
                                }
                            }
                        }
                    }
                }
                if is_selected_piece && has_piece {
                    //in this case we handle two cases
                    //CASE 1: if the piece we are trying to select is of the same color as the already selected one 
                    //we select it if it's not a castle 
                    if let (Some(old), Some(new)) = (old_piece, new_piece) {
                        if old.color == new.color {
                            //check for rook special case
                            //if castle possible we castle else just select it
                            if new.kind == PieceType::Rook && old.kind == PieceType::King{
                                let casteled = self.board.try_castle(old.position, new.position);
                                
                                if !casteled {self.board.select_piece(new);}
                                else {self.board.deselect_piece(); self.board.change_turn();}
                            } else {
                                self.board.select_piece(new);
                            }
                        } 
                        // In the branch where is_selected_piece && has_piece and colors differ (capture)
                        else {
                            if let Some(old) = old_piece {
                                if let Ok(mut ms) = self.board.move_piece(old.position, *square) {
                                    if ms.uci.is_empty() {
                                        ms.uci = self.board.encode_uci_move(old.position, *square, None);
                                    }
                                    self.board.meta_data.move_list.push(ms.clone());

                                    let (tx, rx) = mpsc::channel::<EvalResponse>();
                                    self.evaluator
                                        .send_eval_request(self.board.to_string(), EvalKind::MoveEval { reply_to: tx });
                                    if let Ok(resp) = rx.try_recv() {
                                        if let Some(last) = self.board.meta_data.move_list.last_mut() {
                                            last.evaluation = resp;
                                            self.board.ui.bar_eval = last.evaluation.value;
                                        }
                                    }
                                }
                            }
                        }   
                            }
                        
        
                    }
                
                }   
                }
                }
                if self.board.has_lost() { 
                    self.game.game_over = true;
                    self.popup = Some(PopupType::GameLostPopup("ai perdut".to_owned()));
                    let king_pos = self
                        .board
                        .squares
                        .iter()
                        .enumerate()
                        .find_map(|(r, row)| {
                            row.iter().enumerate().find_map(|(c, &cell)| {
                                if let Some(piece) = cell {
                                    if piece.kind == PieceType::King && piece.color == self.board.turn {
                                        Some((r as u8, c as u8))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                        });
                        self.board.ui.checkmate_square = king_pos;
                   
                }
            }
            
        
        }
        GameMode::Sandbox => {
            //If we right click the selected piece loses focus
            if response.secondary_clicked() {
                self.board.deselect_piece();
            }
            if response.clicked() {
                //First we check if a piece is already selected
                let old_piece: Option<ChessPiece> = self.board.ui.selected_piece;
                let new_piece: Option<ChessPiece> = self.board.squares[square.0 as usize][square.1 as usize];
             
                let is_selected_piece: bool = old_piece.is_some();
                let has_piece: bool = new_piece.is_some();
                 //CASE 1: there is no selected piece so we must set the seleced piece to the square we interact with
                if !is_selected_piece && has_piece {
                    //get the piece from the square
                    
                    let piece: ChessPiece = self.board.squares[square.0 as usize][square.1 as usize].unwrap(); 
                    self.board.select_piece(piece);
                    
                    //checking if the piece we try to select matches the player s turn
                }
                if !is_selected_piece && has_piece {
                    //get the piece from the square
                    if let Some(piece) = new_piece {
                        //checking if the piece we try to select matches the player's turn
                        if piece.color == self.board.turn {
                            //We select the new piece
                            self.board.select_piece(piece);   
                        }
                    }
                }
                if is_selected_piece && !has_piece { 
                    //in this case a movement is performed
                    if let Some(piece) = old_piece {
                        if let Ok(mut ms) = self.board.move_piece(piece.position, *square){
                              self.board.meta_data.move_list.push(ms);

                        
                        };
                    }
                }
                if is_selected_piece && has_piece {
                    //in this case we handle two cases
                    //CASE 1: if the piece we are trying to select is of the same color as the already selected one 
                    //we select it if it's not a castle 
                    if let (Some(old), Some(new)) = (old_piece, new_piece) {
                        if old.color == new.color {
                            //check for rook special case
                            //if castle possible we castle else just select it
                            if new.kind == PieceType::Rook && old.kind == PieceType::King{
                                let casteled = self.board.try_castle(old.position, new.position);
                                
                                if !casteled {self.board.select_piece(new);}
                            } else {
                                self.board.select_piece(new);
                            }
                        } 
                        else {
                            if let Some(piece) = old_piece {
                        if let Ok(mut ms) = self.board.move_piece(piece.position, *square){
                          self.board.meta_data.move_list.push(ms);

                        };
                    }
                            }   
                            }
                        
        
                    }

                }
                
            }
            _ => {}
        }
        }
        
        pub fn poll_stockfish(&mut self, fen: String) {
            if !matches!(self.game.mode, GameMode::PvE) || self.game.game_over {
                return;
            }
            if self.game.player != self.board.turn {
                if !self.game.stockfish_move_pending {
                    self.game.stockfish_move_pending = true;
                    if let Some(tx) = &self.game.stockfish_tx {
                        let _ = tx.send(StockfishCmd::Go(fen));
                    }
                } else if let Some(rx) = &self.game.stockfish_rx {
                    if let Ok(StockfishResult::Move(mv)) = rx.try_recv() {
                        if let Some((from, to)) = self.board.decode_uci_move(mv.clone()) {
                            if let Ok(mut ms) = self.board.move_piece(from, to) {
                                // set UCI from engine reply
                                ms.uci = mv;
                                self.board.meta_data.move_list.push(ms);
                            }
                        }
                        self.game.stockfish_move_pending = false;
                    }
                }
            }
        }

            }

