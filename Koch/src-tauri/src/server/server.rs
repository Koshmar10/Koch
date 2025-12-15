use crate::analyzer::analyzer::EngineCommand;
use crate::analyzer::board_interactions::AnalyzerController;
use crate::{database, engine::Board, game::controller::GameController};

use serde::{Deserialize, Serialize};
use sysinfo::System;

use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;
use stockfish::{EngineEval, Stockfish};
use ts_rs::TS;

#[derive(Clone, Debug, TS, Serialize)]
#[ts(export)]
pub enum EvalKind {
    Mate,
    Centipawn,
}
#[derive(Clone, Debug, TS, Serialize)]
#[ts(export)]
pub struct PvLineData {
    pub moves: String,
    pub eval_kind: EvalKind,
    pub eval_value: i32,
}

#[derive(Clone, Debug, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PvObject {
    pub fen: String,
    pub depth: u32,
    // Changed: lines now map to PvLineData instead of just String
    pub lines: HashMap<u8, PvLineData>,
}

impl Default for PvObject {
    fn default() -> Self {
        Self {
            fen: String::new(),
            depth: 0,
            lines: std::collections::HashMap::new(),
        }
    }
}

// Commands the UI can send to the Engine
/*
"rnbqkbnr/ppp1pppp/8/3p4/3PP3/8/PPP2PPP/RNBQKBNR b KQkq - 0 2": {
    "name": "Blackmar-Diemer Gambit",
    "eco": "D00",
    "moves": "1. d4 d5 2. e4",
        "src": "eco_tsv",
        "scid": "D00l",
        "aliases": {
            "scid": "Blackmar-Diemer Gambit (BDG): 2.e4",
            "ct": "Blackmar-Diemer Gambit, General",
            "chronos": "Blackmar gambit"
        }
    }, */
#[derive(Debug)]
pub struct Settings {
    pub corrupted: bool,
    pub map: HashMap<String, String>,
}
impl Settings {
    pub fn update(&mut self, key: String, val: String) {
        let sv = self.map.entry(key).or_insert("".to_string());
        *sv = val;
    }
    pub fn save(&mut self) -> Result<(), Box<dyn Error>> {
        let mut settings_string = String::new();
        // FIX: Iterate by reference (&self.map) instead of draining (moving) the values
        for (key, value) in &self.map {
            let line = format!("{}={}\n", key, value);
            settings_string.push_str(&line);
        }
        settings_string = settings_string.trim().to_string();

        // FIX: Write to ../koch.config to avoid triggering the src-tauri file watcher
        fs::write("../koch.config", settings_string)?;
        Ok(())
    }
}
#[derive(Deserialize)]
pub struct OpeningEntry<'a> {
    pub name: Cow<'a, str>,
    pub moves: Vec<Cow<'a, str>>,
}

pub struct ServerState<'a> {
    pub board: Board,
    pub engine: Option<Stockfish>,
    pub opening_index: Option<HashMap<String, OpeningEntry<'a>>>,
    pub game_controller: Option<GameController>,
    pub analyzer_controller: AnalyzerController,
    pub analyzer_tx: Option<Sender<EngineCommand>>,
    pub analyzer_rx: Option<Receiver<PvObject>>,
    pub total_memory: f64,
    pub nbcpu: usize,
    pub settings: Settings,
}
impl<'a> Default for ServerState<'a> {
    fn default() -> Self {
        let mut board = Board::default();
        board.rerender_move_cache();
        let engine = match Stockfish::new("/usr/bin/stockfish") {
            Ok(mut s) => {
                if s.setup_for_new_game().is_ok() && s.set_skill_level(16).is_ok() {
                    Some(s)
                } else {
                    None
                }
            }
            Err(_) => None,
        };
        let game_controller = None;
        let analyzer_controller = AnalyzerController::default();

        let mut opening_index: Option<HashMap<String, OpeningEntry<'a>>> = None;
        let path =
            Path::new("/home/petru/storage/Projects/chess_app/Koch/src-tauri/src/openings.json");
        let mut file = match File::open(path) {
            Err(why) => {
                println!("could not open openings : {why}");
                None
            }
            Ok(file) => Some(file),
        };
        match file {
            Some(mut f) => {
                let mut buf = String::new();
                f.read_to_string(&mut buf);
                opening_index =
                    match serde_json::from_str::<HashMap<String, OpeningEntry<'a>>>(&buf) {
                        Ok(res) => Some(res),
                        Err(e) => {
                            eprintln!("{e}");
                            None
                        }
                    };
            }
            None => {}
        }

        database::create::create_database()
            .inspect_err(|e| eprintln!("{e}"))
            .ok();
        return ServerState {
            board,
            engine,
            game_controller,
            analyzer_controller,
            analyzer_rx: None,
            analyzer_tx: None,
            opening_index: opening_index,
            total_memory: 0.0,
            nbcpu: 1,
            settings: load_settings().unwrap_or_else(|_| Settings {
                corrupted: true,
                map: HashMap::new(),
            }),
        };
    }
}
#[tauri::command]
pub fn get_system_information(state: tauri::State<'_, Mutex<ServerState>>) -> (f64, usize) {
    let state = state.lock().unwrap();
    println!("RAM capacity: {} GB", &state.total_memory);
    println!("Number of CPUs: {}", &state.nbcpu);
    (state.total_memory, state.nbcpu)
}

pub fn load_settings() -> Result<Settings, io::Error> {
    let mut settings_map = HashMap::new();
    let mut corrupted = false;

    // FIX: Read from ../koch.config
    let settings = fs::File::open("../koch.config")?;
    let reader = BufReader::new(settings);
    for line in reader.lines() {
        match line {
            Ok(line) => {
                let split_line: Vec<&str> = line.split("=").collect();
                settings_map.insert(split_line[0].to_string(), split_line[1].to_string());
            }
            Err(_) => {
                corrupted = true;
            }
        }
    }

    Ok(Settings {
        corrupted: corrupted,
        map: settings_map,
    })
}
