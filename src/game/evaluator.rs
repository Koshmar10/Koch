use std::sync::{mpsc::{Receiver, Sender}, Arc};

use eframe::egui::mutex::Mutex;
use stockfish::Stockfish;

use crate::{engine::PieceColor, ui::app::MyApp};

pub struct EvaluatorQueue {
    pub eval_queue: Vec<EvaluationRequest>,
    // tx for transmiting a eval request to the evaluator
    pub eval_request_tx: Option<Sender<EvaluationRequest>>,
    //rx for receiving the evaluation in centipawns,
    pub eval_receiver_rx: Option<Receiver<f32>>
}
pub enum EvalKind {BarEval, MoveEval{ reply_to: Sender<EvalResponse>}}

pub struct EvaluationRequest{
    pub position: String,
    pub kind: EvalKind,
}
pub struct EvalResponse {
    pub value:f32,
}

impl Default for EvaluatorQueue {
    fn default() -> Self {
        Self { eval_queue: vec![], eval_request_tx: None, eval_receiver_rx: None,}
    }
}

pub struct Evaluator {
    pub stockfish_engine: Option<Arc<Mutex<Stockfish>>>,
    //reciever for the evaluations
    pub request_manager: EvaluatorQueue
}
pub struct Evaluation {
    pub centipawns: u32,
    pub color: PieceColor,
}
impl Evaluator  {
    pub fn new() -> Self {
       
            
    let ev = Stockfish::new("/usr/bin/stockfish")
        .ok()
        .and_then(|mut s| {
            s.setup_for_new_game().ok()?;
            s.set_elo(3500).ok()?;
            s.set_threads(10).ok()?;
            println!("started");
           
            Some(Arc::new(Mutex::new(s)))
        });
   
    Self {
        stockfish_engine: ev,
        request_manager: EvaluatorQueue::default()
    }
}
}

impl MyApp {
    pub fn start_evaluator(&mut self) {

        //asign the recievere and sender or the otside of the thread
        let stockfish_evaluator = match &self.evaluator.stockfish_engine {
            Some(s) => {
                Arc::clone(s)
            }
            None => {
                return;
            }
        };

        //we set up the channels
        let (eval_request_tx, eval_request_rx) = std::sync::mpsc::channel::<EvaluationRequest>();
        let (eval_receiver_tx,  eval_receiver_rx) = std::sync::mpsc::channel::<f32>();

        self.evaluator.request_manager.eval_request_tx = Some(eval_request_tx);
        self.evaluator.request_manager.eval_receiver_rx = Some(eval_receiver_rx);
        
        let _ = std::thread::Builder::new().name("evaluator_thread".to_string()).spawn(move || {
            let mut engine = stockfish_evaluator.lock();
            let mut request_queue:Vec<EvaluationRequest> = Vec::new();
            loop {
                match eval_request_rx.recv() {
                    Ok(fen) => {
                        request_queue.push(fen);
                    }
                    Err(e) => {}//eprintln!("Line {}: {}", line!(), e), // Added line number
                }
                if !request_queue.is_empty(){
                    let currrent_request = request_queue.remove(0);
                    match engine.set_fen_position(&currrent_request.position) {
                        Ok(_) => {}
                        Err(e) => {}//eprintln!("Line {}: {}", line!(), e)  // Added line number
                    }
                    match currrent_request.kind {
                        EvalKind::BarEval => {
                            for i in [2,  10,  12]{
                                engine.set_depth(i);
                                
                                match engine.go() {
                                    Ok(output) => {
                                        let eval = output.eval();
                                        let eval_type = eval.eval_type();
                                        println!("{eval_type}");
                                        let _ = eval_receiver_tx.send(eval.value() as f32);
                                    }
                                    Err(e) => {}//eprintln!("Line {}: {}", line!(), e)  // Added line number
                                }
                            }
                        }
                        EvalKind::MoveEval { reply_to: sender } => {

                                engine.set_depth(15);    
                                match engine.go() {
                                    Ok(output) => {
                                        let eval = output.eval();
                                        let eval_type = eval.eval_type();
                                        println!("{eval_type}");
                                        let _ = sender.send(EvalResponse { value: eval.value() as f32 });
                                    }
                                    Err(e) =>{}//eprintln!("Line {}: {}", line!(), e)  // Added line number
                                };
                            
                        }
                        
                    }
                    
                }
               
            
        }});
        
        
        
    }
    pub fn get_evaluation(&mut self) ->f32 {
        let rx = match &self.evaluator.request_manager.eval_receiver_rx {
            Some(rx) => rx,
            None => return 0.0,
        };
        match rx.try_recv() {
            Ok(eval) => {
                println!("{eval}");
                self.board.state.current_evaluation = eval;
                return self.board.state.current_evaluation;
            },   
            Err(e) => {
                return self.board.state.current_evaluation;
         
            }
        }
    }
    
}
impl Evaluator {
    pub fn send_eval_request(&mut self, board_position: String, eval_kind: EvalKind) {
        let  tx = match &self.request_manager.eval_request_tx{
            Some(tx) => tx,
            None => return,
        };
        match tx.send(EvaluationRequest { position: board_position, kind: eval_kind}) {
            Ok(_) => {}
            Err(e) => {}//eprintln!("Line {}: {}", line!(), e)  // Added line number
        }
    }
    
}