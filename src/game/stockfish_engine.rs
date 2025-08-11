use std::{sync::{ mpsc::{self, Receiver, Sender}, Arc, Mutex}, thread, time::Duration};

use stockfish::Stockfish;

use crate::ui::app::MyApp;

pub enum StockfishCmd {
    NewGame,
    GoDepth(usize),
    Go(String),
    Stop,
    Eval,
}
pub enum StockfishResult {
    Succes,
    Fail,
    Move(String),
}

pub enum StockfishOutput { InvalidOutput, ValidOutput(String)}
impl MyApp{

    pub fn start_stockfish(&mut self){

        self.game.stockfish = match Stockfish::new("/usr/bin/stockfish") {
            
            Ok(mut s) => {
                match s.setup_for_new_game(){
                    Ok(_) => {
    
                        match s.set_skill_level(16){
                            Ok(_) => {
                                println!("started");
                                Some(Arc::new(Mutex::new(s)))
                            }
                            Err(e) => {None}
                        }
                    }
                    Err(e) => {None}
                }
    
            }
            Err(_) => { None }
        };
        let stockfish: Arc<Mutex<Stockfish>> = match &self.game.stockfish {
            Some(s) => Arc::clone(s),
            None => return,
            
        };
        let search_dept = (self.game.search_depth as u32).clone();
        println!("cloned stockfish");
        let (cmd_tx, cmd_rx): (Sender<StockfishCmd>, Receiver<StockfishCmd>) =
            mpsc::channel();
        let (res_tx, res_rx): (Sender<StockfishResult>, Receiver<StockfishResult>) = mpsc::channel(); // assume you stored a Sender for results earlier
        self.game.stockfish_rx = Some(res_rx);
        
                // … inside your spawn:
                let _ = thread::Builder::new().name("player_stockfish".to_string()).spawn(move || {
                    println!("Spawned thread");
                    let mut engine = stockfish.lock().unwrap();
                    // make sure engine is fresh
                    let _ = engine.setup_for_new_game();
                    engine.set_depth(search_dept);
                    loop {
                        match cmd_rx.recv() {
                            Ok(cmd) => {
                                match cmd {
                                    StockfishCmd::NewGame => {
                                        match  engine.setup_for_new_game(){
                                            Err(e)=> println!("{:?}", e),
                                            Ok(_) => println!("Setup for new game succesfull"),
                                        }
                                        let _ = res_tx.send(StockfishResult::Succes);
                                    }
                                    StockfishCmd::GoDepth(depth) => {
                                        engine.set_depth(depth as u32);
                                    }
                                    StockfishCmd::Go(mv) => {
                                        // before calling go(), tell Stockfish about the current position (the FEN string in `mv`)
                                        let _ = engine.set_fen_position(&mv.clone());
                                        println!("recived go");
                                        match engine.go() {
                                            
                                            Ok(output) => {
                                                let mv = output.best_move().clone();
                                                println!("got mv");
                                                let _ = res_tx.send(StockfishResult::Move(mv));
                                            }
                                            Err(e) => {
                                                eprintln!("engine.go() error: {:?}", e);
                                                let _ = res_tx.send(StockfishResult::Fail);
                                            }
                                        }
                                    }
                                    StockfishCmd::Stop => {
                                        match  engine.setup_for_new_game(){
                                            Err(e)=> println!("{:?}", e),
                                            Ok(_) => println!("Setup for new game succesfull"),
                                        }
                                        let _ = res_tx.send(StockfishResult::Succes);
                                        break;
                                    }
                                    StockfishCmd::Eval =>{
                                        match engine.go() {
                                            
                                            Ok(output) => {
                                               let evaluation_score = output.eval();
                                            }
                                            Err(e) => {
                                                eprintln!("engine.go() error: {:?}", e);
                                                let _ = res_tx.send(StockfishResult::Fail);
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // channel closed -> exit
                                break;
                            }
                        }
                        // small sleep to yield if you ever decide to switch back to try_recv
                        thread::sleep(Duration::from_millis(1));
                    }
                });
    
        self.game.stockfish_tx = Some(cmd_tx);
        
    }
    pub fn get_stockfish_move(&self) -> Option<String>{
        let uci_move = String::new();
        
        match &self.game.stockfish_tx {
            Some(tx) =>{
               
                match tx.send(StockfishCmd::Go(self.board.to_string())) {
                    Ok(_) =>{}
                    Err(e) => {
                        println!("{:?}", e);
                        return None;
                    }
                }
                
            } 
            
            None => {println!("nu e transmitor stockfish")}
        }
    
        Some(uci_move)
    }
    
    pub fn make_stockfish_move(&mut self) {
        // ask the engine for a move
        if let Some(uci_move) = self.get_stockfish_move() {
            // UCI string is at least 4 chars long, e.g. "e2e4"
            match self.board.decode_uci_move(uci_move){
                Some((from, to)) => {
                    let _ = self.board.move_piece(from, to);
                }
                None => {}
            }
                
            }
    
            // we're done with this engine move
            self.game.stockfish_move_pending = false;
        
    }

}