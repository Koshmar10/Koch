use rusqlite::types::Value;
use std::collections::HashMap;
use std::error::Error;
use std::io::{BufRead, BufReader};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;
use std::{fs, io, thread};
use stockfish::Stockfish;

// --- Data Structures ---

#[derive(Clone, Debug, Default)]
struct PvLine {
    moves: String,
    score_type: String, // "cp" (centipawns) or "mate"
    score_value: i32,
}

#[derive(Clone, Debug, Default)]
struct PvObject {
    fen: String,
    depth: u32,
    // MultiPV Index -> Line Data
    lines: std::collections::HashMap<u8, PvLine>,
}

// Commands the UI can send to the Engine
enum EngineCommand {
    SetFen(String),
    GoInfinite,
    Stop,
    Quit,
}


fn main() {
}
