use std::sync::{mpsc::{Receiver, Sender}, Arc};
use std::time::Duration;

use rusqlite::ToSql;
use stockfish::{EvalType, Stockfish};

use crate::{engine::PieceColor};

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
    pub kind: EvalKind
}
#[derive(Clone)]

impl EvalResponse {
    // Convert EvalResponse to SQL-compatible values
    pub fn to_sql_params(&self) -> (f32, String) {
        let eval_type_str = match self.kind {
            EvalType::Centipawn => "cp".to_string(),
            EvalType::Mate => "mate".to_string(),
            // Add other variants as needed
            _ => "unknown".to_string(),
        };
        
        (self.value, eval_type_str)
    }
    
    // Create an EvalResponse from SQL-retrieved values
    
}
pub fn from_sql_params(value: f32, eval_type_str: &str) -> EvalResponse {
        let kind = match eval_type_str {
            "cp" => EvalType::Centipawn,
            "mate" => EvalType::Mate,
            // Add other variants as needed
            _ => EvalType::Centipawn,
        };
        
        EvalResponse { value, kind }
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

            use std::collections::VecDeque;

            loop {
                // Block briefly for the first request
                let first = match eval_request_rx.recv_timeout(Duration::from_millis(200)) {
                    Ok(req) => Some(req),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => None,
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                };

                // Accumulate: keep only the latest BarEval; keep ALL MoveEval
                let mut latest_bar: Option<String> = None;           // FEN
                let mut move_reqs: VecDeque<EvaluationRequest> = VecDeque::new();

                let mut push_req = |req: EvaluationRequest| {
                    match req.kind {
                        EvalKind::BarEval => {
                            latest_bar = Some(req.position);
                        }
                        EvalKind::MoveEval { reply_to } => {
                            move_reqs.push_back(EvaluationRequest {
                                position: req.position,
                                kind: EvalKind::MoveEval { reply_to },
                            });
                        }
                    }
                };

                if let Some(req) = first { push_req(req); }
                while let Ok(req) = eval_request_rx.try_recv() {
                    push_req(req);
                }

                // Process all MoveEval first; never drop them
                while let Some(req) = move_reqs.pop_front() {
                    let (pos, reply_to) = match req.kind {
                        EvalKind::MoveEval { reply_to } => (req.position, reply_to),
                        _ => unreachable!(),
                    };

                    if let Err(e) = engine.set_fen_position(&pos) {
                        let _ = reply_to.send(EvalResponse { value: 0.0, kind: EvalType::Centipawn });
                        eprintln!("{}:{} set_fen_position failed: {}", file!(), line!(), e);
                        continue;
                    }

                    let _ = engine.set_depth(15);
                    match engine.go() {
                        Ok(output) => {
                            let eval = output.eval();
                            let kind = eval.eval_type();
                            let value = eval.value() as f32;
                            let _ = reply_to.send(EvalResponse { value, kind });
                        }
                        Err(e) => {
                            eprintln!("{}:{} engine.go() failed: {}", file!(), line!(), e);
                            let _ = reply_to.send(EvalResponse { value: 0.0, kind: EvalType::Centipawn });
                        }
                    }
                }

                // Then process the latest BarEval (optional UI-only)
                if let Some(pos) = latest_bar {
                    if engine.set_fen_position(&pos).is_ok() {
                        let _ = engine.set_depth(15);
                        if let Ok(output) = engine.go() {
                            let eval = output.eval();
                            let _ = eval_receiver_tx.send(eval.value() as f32);
                        }
                    }
                }
            }
        });
    }
    pub fn get_evaluation(&mut self) -> f32 {
        let Some(rx) = &self.evaluator.request_manager.eval_receiver_rx else {
            return self.board.ui.bar_eval;
        };
        // Drain to newest value this frame
        let mut last = self.board.ui.bar_eval;
        while let Ok(v) = rx.try_recv() {
            last = v;
        }
        self.board.ui.bar_eval = last;
        last
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