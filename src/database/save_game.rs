use std::sync::mpsc;

use crate::{database::create::{insert_game_and_get_id, insert_single_move}, engine::Board, game::evaluator::{EvalKind, EvalResponse, EvaluationRequest}, ui::app::MyApp};


impl MyApp{
    pub fn save_game(&mut self){
        //we get the reference of the board that was plaed, we reset it to the starting position
          let save_info = insert_game_and_get_id(&self.board.meta_data);
        let save_info = match save_info {
            Err(e) => {print!("Save Error {e}"); return;}
            Ok(save) => {save}
        };
        
        let mut  sample_board = Board::from(&self.board.meta_data.starting_position);
        sample_board.rerender_move_cache();
        
        let mut made_moves = self.board.meta_data.move_list.clone();
        self.start_evaluator();
        let eval_sender = match self.evaluator.request_manager.eval_request_tx.clone(){
            Some(x) => x,
            None => {println!("No eval ender");return},
        };
        let (tx, rx) = mpsc::channel::<u8>();
        self.ui_controller.save_game_rx = Some(rx);
        let _ = std::thread::Builder::new().name("save_worler_thread".to_string()).spawn(
            move || {
            for mv in made_moves.iter_mut() {
                let (old_pos, new_pos) = sample_board.decode_uci_move(mv.uci.clone()).unwrap();
                if let Err(e) = sample_board.move_piece(old_pos, new_pos) {
                    println!("Apply move error for '{}': {:?}", mv.uci, e);
                    break;
                }
                let (os_tx, os_rx) = mpsc::channel::<EvalResponse>();
                let _ = eval_sender.send(EvaluationRequest {
                    position: sample_board.to_string(),
                    kind: EvalKind::MoveEval { reply_to: os_tx }
                });
                match os_rx.recv_timeout(std::time::Duration::from_secs(3)) {
                    Ok(response) => {
                        mv.evaluation = response;
                        if let Err(e) = insert_single_move(save_info, mv) {
                            println!("Save move error '{}': {e}", mv.uci);
                            break;
                        }
                    }
                    Err(e) => {
                        println!("Evaluation timeout/closed for '{}': {e}", mv.uci);
                        break;
                    }
                }
                sample_board.rerender_move_cache();
                }
            tx.send(0)
            }
        );
        
       
    }

    pub fn start_save_game_sequence(&mut self){
        if !self.ui_controller.saving_game {
            self.save_game();
            self.ui_controller.saving_game = true;
        }
    }
    pub fn poll_save_game_worker(&mut self){
        match &self.ui_controller.save_game_rx {
            Some(rx) => {
                //save_game_worker active
                match rx.try_recv() {
                    Ok(code)=> {
                        match code {
                            0 => {
                                //game was saved
                                self.popup = None;
                            }
                            1 => {
                                //error occured
                            }
                            _ => { // unknown code}
                        }
                        }
                    }
                    Err(e) => {}

                }
            }
            None => {}
         }
    }

}

