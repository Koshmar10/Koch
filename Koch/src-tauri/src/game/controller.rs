use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use stockfish::Stockfish;
use ts_rs::TS;

use crate::{
    database::create::{metadata_to_pgn, save_game},
    engine::{
        board::{BoardMetaData, GameResult},
        serializer::{serialize_board, SerializedBoard},
        Board, PieceColor, PieceType,
    },
    make_engine_move,
    server::server::ServerState,
};

#[derive(Clone, Debug)]

pub struct ChessClock {
    pub white_time_remaining: Duration,
    pub black_time_remaining: Duration,
    pub last_turn_start: Instant,
    pub is_active: bool,
    pub active_color: PieceColor,
}
impl From<GameControllerMode> for ChessClock {
    fn from(mode: GameControllerMode) -> Self {
        let duration = match mode {
            GameControllerMode::Bullet => Duration::from_secs(60),
            GameControllerMode::Blitz => Duration::from_secs(180),
            GameControllerMode::Rapid => Duration::from_secs(600),
            GameControllerMode::Classical => Duration::from_secs(1800),
        };
        ChessClock {
            white_time_remaining: duration,
            black_time_remaining: duration,
            last_turn_start: Instant::now(),
            active_color: PieceColor::White,
            is_active: false,
        }
    }
}
fn format_duration(duration: Duration) -> u32 {
    let total_millis = duration.as_millis();
    let minutes = (total_millis / 60000) % 60;
    let seconds = (total_millis / 1000) % 60;
    let millis = total_millis % 1000;
    (total_millis) as u32
}
impl ChessClock {
    pub fn format(&self) -> (u32, u32) {
        (
            format_duration(self.white_time_remaining),
            format_duration(self.black_time_remaining),
        )
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum GameControllerState {
    AwaitingStart,
    Ongoing,
    Ended,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum TerminationReason {
    Checkmate,
    StaleMate,
    Draw,
    Timeout,
    Resignation,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum GameControllerMode {
    Bullet,
    Blitz,
    Rapid,
    Classical,
}

#[derive(Clone)]
pub struct GameController {
    pub mode: GameControllerMode,
    pub player: PieceColor,
    pub white_name: String,
    pub black_name: String,
    pub white_elo: usize,
    pub black_elo: usize,
    pub board: Board,
    pub clock: ChessClock,
    pub state: GameControllerState,
    pub termination_reason: Option<TerminationReason>,
    pub result: Option<GameResult>,
    pub elo_gain: Option<i32>,
    pub can_be_abandoned: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SerializedGameController {
    //name cards with : name (elo)
    pub player: PieceColor,
    pub mode: GameControllerMode,
    pub player_card: String,
    pub engine_card: String,
    pub board: SerializedBoard,
    pub player_clock: u32,
    pub engine_clock: u32,
    pub state: GameControllerState,
    pub termination_reason: Option<TerminationReason>,
    pub result: Option<GameResult>,
    pub elo_gain: Option<i32>,
    pub can_be_abandoned: bool,
}

impl Default for GameController {
    fn default() -> Self {
        GameController::from(GameControllerMode::Rapid)
    }
}
impl From<GameControllerMode> for GameController {
    fn from(mode: GameControllerMode) -> Self {
        GameController {
            mode,
            player: PieceColor::White,
            white_name: "Petru".into(),
            black_name: "Stockfish".into(),
            white_elo: 500,
            black_elo: 500,
            board: Board::default(),
            clock: ChessClock::from(mode),
            state: GameControllerState::AwaitingStart,
            termination_reason: None,
            result: None,
            elo_gain: None,
            can_be_abandoned: true,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum UpdateType {
    EngineMove,
    Playermove {
        from: (u8, u8),
        to: (u8, u8),
        promotion: Option<PieceType>,
    },
}
impl GameController {
    pub fn new() -> Self {
        GameController::default()
    }
    pub fn start(&mut self) -> SerializedGameController {
        let player_color = if rand::random() {
            PieceColor::White
        } else {
            PieceColor::Black
        };
        self.player = player_color;
        self.clock = ChessClock::from(self.mode);
        self.clock.is_active = true;
        self.state = GameControllerState::Ongoing;
        self.board = Board::default();
        self.board.meta_data.site = Some("Koch".into());
        self.board.meta_data.time_control = match self.mode {
            GameControllerMode::Bullet => Some("60".to_string()),
            GameControllerMode::Blitz => Some("180".to_string()),
            GameControllerMode::Rapid => Some("600".to_string()),
            GameControllerMode::Classical => Some("1800".to_string()),
        };
        self.board.meta_data.black_player_elo = self.black_elo as u32;
        self.board.meta_data.black_player_elo = self.white_elo as u32;
        self.board.meta_data.black_player_name = self.black_name.clone();
        self.board.meta_data.white_player_name = self.white_name.clone();
        self.board.rerender_move_cache();
        self.can_be_abandoned = true;
        self.serialize()
    }
    pub fn export() {}
    pub fn end_game(
        &mut self,
        reason: TerminationReason,
        loser: PieceColor,
    ) -> SerializedGameController {
        self.termination_reason = Some(reason);
        self.state = GameControllerState::Ended;
        self.result = match reason {
            TerminationReason::Checkmate
            | TerminationReason::Timeout
            | TerminationReason::Resignation => match loser {
                PieceColor::White => Some(GameResult::BlackWin),
                PieceColor::Black => Some(GameResult::WhiteWin),
            },
            TerminationReason::StaleMate | TerminationReason::Draw => Some(GameResult::Draw),
        };
        if !(self.board.meta_data.move_list.len() < 2) {
            // Compute elo_gain using loser variable
            self.elo_gain = match reason {
                TerminationReason::Checkmate
                | TerminationReason::Timeout
                | TerminationReason::Resignation => {
                    if self.player != loser {
                        Some(10)
                    } else {
                        Some(-10)
                    }
                }
                TerminationReason::StaleMate | TerminationReason::Draw => Some(0),
            };
        }
        self.serialize()
    }
    pub fn update(
        &mut self,
        from: (u8, u8),
        to: (u8, u8),
        promotion: Option<PieceType>,
    ) -> SerializedGameController {
        let now = Instant::now();
        let elapsed = now.duration_since(self.clock.last_turn_start);
        if self.clock.is_active {
            match self.clock.active_color {
                PieceColor::White => {
                    if self.clock.white_time_remaining > elapsed {
                        self.clock.white_time_remaining -= elapsed;
                    } else {
                        self.clock.white_time_remaining = Duration::ZERO;
                    }
                }
                PieceColor::Black => {
                    if self.clock.black_time_remaining > elapsed {
                        self.clock.black_time_remaining -= elapsed;
                    } else {
                        self.clock.black_time_remaining = Duration::ZERO;
                    }
                }
            }
            // Format clocks as "mm:ss:ms"
            let format = {
                let ms = match self.clock.active_color {
                    PieceColor::White => self.clock.format().0,
                    PieceColor::Black => self.clock.format().1,
                };
                let minutes = (ms / 60000) % 60;
                let seconds = (ms / 1000) % 60;
                let millis = (ms % 1000) / 10; // 2-digit ms
                format!("{:02}:{:02}:{:02}", minutes, seconds, millis)
            };
            self.clock.last_turn_start = now;
            self.clock.active_color = match self.clock.active_color {
                PieceColor::White => PieceColor::Black,
                PieceColor::Black => PieceColor::White,
            };
            let move_result = self.board.move_piece(from, to, promotion);
            match move_result {
                Ok(mut mv_struct) => {
                    mv_struct.clock = Some(format);
                    self.board.meta_data.move_list.push(mv_struct.clone());
                }
                Err(e) => println!("Failed to update, invalid move"),
            }
            if self.board.has_lost() {
                self.termination_reason = self.board.get_termination_reason();
                self.state = GameControllerState::Ended;
                self.result = match self.termination_reason {
                    Some(TerminationReason::Checkmate) => {
                        if self.board.turn == PieceColor::White {
                            Some(GameResult::BlackWin)
                        } else {
                            Some(GameResult::WhiteWin)
                        }
                    }
                    Some(TerminationReason::StaleMate) | Some(TerminationReason::Draw) => {
                        Some(GameResult::Draw)
                    }
                    Some(TerminationReason::Timeout) => {
                        if self.board.turn == PieceColor::White {
                            Some(GameResult::BlackWin)
                        } else {
                            Some(GameResult::WhiteWin)
                        }
                    }
                    Some(TerminationReason::Resignation) => {
                        if self.board.turn == PieceColor::White {
                            Some(GameResult::BlackWin)
                        } else {
                            Some(GameResult::WhiteWin)
                        }
                    }
                    None => None,
                };
                // Compute elo_gain in a single line
                self.elo_gain = match self.result {
                    Some(GameResult::WhiteWin) if self.player == PieceColor::White => Some(10),
                    Some(GameResult::BlackWin) if self.player == PieceColor::Black => Some(10),
                    Some(GameResult::WhiteWin) | Some(GameResult::BlackWin) => Some(-10),
                    Some(GameResult::Draw) => {
                        let delta = (self.white_elo as i32 - self.black_elo as i32).abs();
                        if delta > 10 {
                            if (self.player == PieceColor::White && self.white_elo > self.black_elo)
                                || (self.player == PieceColor::Black
                                    && self.black_elo > self.white_elo)
                            {
                                Some(-1)
                            } else {
                                Some(1)
                            }
                        } else {
                            Some(0)
                        }
                    }
                    _ => Some(0),
                };
            }
        }
        self.board.rerender_move_cache();
        self.can_be_abandoned = false;
        self.serialize()
    }
    pub fn change_mode(&mut self, new_mode: GameControllerMode) {
        let new_controller = GameController::from(new_mode);
        self.mode = new_controller.mode;
        self.clock = new_controller.clock;
    }
    pub fn serialize(&self) -> SerializedGameController {
        let (white_clock, black_clock) = self.clock.format();
        let (player_card, engine_card, player_clock, engine_clock) = match self.player {
            PieceColor::White => (
                format!("{} ({})", self.white_name, self.white_elo),
                format!("{} ({})", self.black_name, self.black_elo),
                white_clock,
                black_clock,
            ),
            PieceColor::Black => (
                format!("{} ({})", self.black_name, self.black_elo),
                format!("{} ({})", self.white_name, self.white_elo),
                black_clock,
                white_clock,
            ),
        };
        SerializedGameController {
            player: self.player,
            mode: self.mode,
            player_card,
            engine_card,
            board: serialize_board(&self.board),
            player_clock,
            engine_clock,
            state: self.state.clone(),
            termination_reason: self.termination_reason.clone(),
            result: self.result.clone(),
            elo_gain: self.elo_gain,
            can_be_abandoned: self.can_be_abandoned,
        }
    }
    pub fn save() {}
}

#[tauri::command]
pub fn change_gamemode(
    state: tauri::State<'_, Mutex<ServerState>>,
    new_mode: GameControllerMode,
) -> SerializedGameController {
    let mut state = state.lock().unwrap();
    state.game_controller.change_mode(new_mode);
    return state.game_controller.serialize();
}
#[tauri::command]
pub fn start_game(state: tauri::State<'_, Mutex<ServerState>>) -> SerializedGameController {
    let mut state = state.lock().unwrap();
    if let Some(engine) = &mut state.engine {
        engine.setup_for_new_game().ok();
    };
    return state.game_controller.start();
}
#[tauri::command]
pub fn update_game_state(
    state: tauri::State<'_, Mutex<ServerState>>,
    payload: UpdateType,
) -> Result<SerializedGameController, String> {
    let (from, to, promotion) = match payload {
        UpdateType::EngineMove => {
            let mut state_guard = state.lock().unwrap();
            let fen = state_guard.game_controller.board.to_string();
            let board = state_guard.game_controller.board.clone();
            let engine_opt = state_guard.engine.as_mut();
            match engine_opt {
                Some(engine) => {
                    if let Err(e) = engine.set_fen_position(&fen) {
                        return Err(e.to_string());
                    }
                    engine.ensure_ready().ok();
                    match engine.go() {
                        Ok(out) => {
                            let best_move = out.best_move();
                            if let Some((from, to, promotion)) = board.decode_uci_move(best_move) {
                                (from, to, promotion)
                            } else {
                                return Err("failed uci ".into());
                            }
                        }
                        Err(e) => {
                            return Err(e.to_string());
                        }
                    }
                }
                None => {
                    return Err("No engine restart app".into());
                }
            }
        }
        UpdateType::Playermove {
            from,
            to,
            promotion,
        } => (from, to, promotion),
    };

    // Now perform the move and return the serialized game state
    let mut state_guard = state.lock().unwrap();
    let mut serialized = state_guard.game_controller.update(from, to, promotion);
    let mut opening: Option<String> = None;
    match &state_guard.opening_index {
        Some(oi) => match oi.get(&serialized.board.fen) {
            Some(re) => opening = Some(re.name.clone().into()),
            None => {}
        },
        None => {}
    }
    if opening.is_some() {
        state_guard.game_controller.board.meta_data.opening = opening.clone();
        serialized.board.meta_data.opening = opening;
    }
    println!(
        "{:#?}",
        state_guard
            .game_controller
            .board
            .meta_data
            .move_list
            .clone()
    );
    if serialized.elo_gain.is_some() {
        state_guard.update_elo(serialized.elo_gain.unwrap());
    }

    Ok(serialized)
}

#[tauri::command]
pub fn end_game(
    state: tauri::State<'_, Mutex<ServerState>>,
    reason: TerminationReason,
    loser: PieceColor,
) -> SerializedGameController {
    let mut state = state.lock().unwrap();
    let serialized = state.game_controller.end_game(reason, loser);
    if serialized.elo_gain.is_some() {
        state.update_elo(serialized.elo_gain.unwrap());
    }
    serialized
}
#[tauri::command]
pub fn new_game(state: tauri::State<'_, Mutex<ServerState>>) -> SerializedGameController {
    let mut state = state.lock().unwrap();

    let default_player_elo = "500".to_string();
    let default_engine_elo = "500".to_string();
    let (player_elo, engine_elo) = {
        (
            state
                .settings
                .map
                .get("PlayerElo")
                .unwrap_or(&default_player_elo)
                .clone(),
            state
                .settings
                .map
                .get("StockfishElo")
                .unwrap_or(&default_engine_elo)
                .clone(),
        )
    };
    state.game_controller = GameController::new();
    match state.game_controller.player {
        PieceColor::White => {
            state.game_controller.white_elo = player_elo.parse().unwrap_or(500);
            state.game_controller.black_elo = engine_elo.parse().unwrap_or(500);
        }
        PieceColor::Black => {
            state.game_controller.white_elo = engine_elo.parse().unwrap_or(500);
            state.game_controller.black_elo = player_elo.parse().unwrap_or(500);
        }
    };
    if let Some(ref mut engine) = &mut state.engine {
        engine.set_elo(engine_elo.parse().unwrap_or(500)).unwrap();
        engine.ensure_ready().ok();
    }
    state.game_controller.serialize()
}
#[tauri::command]
pub fn save_appgame(state: tauri::State<'_, Mutex<ServerState>>) -> Result<(), String> {
    let state = state.lock().unwrap();
    let metadata = BoardMetaData {
        starting_position: state
            .game_controller
            .board
            .meta_data
            .starting_position
            .clone(),
        date: chrono::Utc::now().naive_utc().date().to_string(),
        move_list: state.game_controller.board.meta_data.move_list.clone(),
        termination: state
            .game_controller
            .termination_reason
            .unwrap_or(TerminationReason::Timeout),
        result: state
            .game_controller
            .result
            .as_ref()
            .cloned()
            .unwrap_or(GameResult::Unfinished),
        white_player_elo: state.game_controller.white_elo as u32,
        black_player_elo: state.game_controller.black_elo as u32,
        white_player_name: state.game_controller.white_name.clone(),
        black_player_name: state.game_controller.black_name.clone(),
        opening: state.game_controller.board.meta_data.opening.clone(),
        event: state.game_controller.board.meta_data.event.clone(),
        site: state.game_controller.board.meta_data.site.clone(),
        round: state.game_controller.board.meta_data.round.clone(),
        time_control: state.game_controller.board.meta_data.time_control.clone(),
        end_time: state.game_controller.board.meta_data.end_time.clone(),
        link: state.game_controller.board.meta_data.link.clone(),
        eco: state.game_controller.board.meta_data.eco.clone(),
    };
    save_game(&metadata).map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
pub fn get_share_data(state: tauri::State<'_, Mutex<ServerState>>) -> (String, String) {
    let (fen, pgn) = {
        let state = state.lock().unwrap();
        let fen = state.game_controller.board.to_string();
        let pgn = metadata_to_pgn(&state.game_controller.board.meta_data);
        (fen, pgn)
    };
    (fen, pgn)
}
/*
#[allow(dead_code)]
fn try_move(
    state: tauri::State<'_, Mutex<ServerState>>,
    src_square: (u8, u8),
    dest_square: (u8, u8),
    promotion: Option<PieceType>,
) -> Option<SerializedBoard> {
    let mut state = state.lock().unwrap();

    // Ensure move cache exists (prevents missing entries for sliding pieces)
    if state.board.move_cache.is_empty() {
        state.board.rerender_move_cache();
    }

    // Capture target BEFORE moving (after move the destination holds the moving piece)
    let captured_before =
        state.board.squares[dest_square.0 as usize][dest_square.1 as usize].clone();

    match state.board.move_piece(src_square, dest_square, promotion) {
        Ok(mut move_struct) => {
            state.board.meta_data.move_list.push(move_struct);
            // Refresh move cache after a successful move
            state.board.rerender_move_cache();
            match &state.opening_index {
                Some(op_idx) => {
                    let fen = &state.board.to_string();
                    println!("{:?}", &fen);
                    match op_idx.get(fen) {
                        None => {}
                        Some(op) => {
                            state.board.meta_data.opening = Some(op.name.to_string());
                            println!("{:?}", &state.board.meta_data.opening)
                        }
                    }
                }
                None => {}
            }
            let sb = serialize_board(&state.board);
            dbg!(&sb);
            Some(sb)
        }
        Err(err) => {
            // Optional: log error
            eprintln!(
                "Illegal move {:?} -> {:?}: {:?}",
                src_square, dest_square, err
            );
            None
        }
    }
}
    */
