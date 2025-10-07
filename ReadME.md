# Koch

A desktop chess app written in Rust with egui/eframe. It renders a playable board, generates legal moves, integrates Stockfish for evaluation and engine moves, and saves games to a local SQLite database for later analysis.

- UI: egui/eframe board, menus, eval bar, move list, and charts
- Engine: custom move generation, FEN I/O, UCI helpers, basic metadata
- Stockfish: background evaluation worker and a command loop for best moves
- Database: rusqlite storage for games and per-move evaluations

## Project layout

- Engine
  - Board state, metadata, UI flags: [src/engine/board.rs](src/engine/board.rs) (e.g., [`Board::move_piece`](src/engine/board.rs), [`Board::try_castle`](src/engine/board.rs), [`Board::can_castle`](src/engine/board.rs), [`Board::is_square_attacked_by`](src/engine/board.rs))
  - Pieces and enums: [src/engine/piece.rs](src/engine/piece.rs)
  - FEN parsing/serialization: [src/engine/fen.rs](src/engine/fen.rs) (e.g., [`fen_parser`](src/engine/fen.rs))
  - Move generation and caching: [src/engine/move_gen.rs](src/engine/move_gen.rs) (e.g., [`Board::get_legal_moves`](src/engine/move_gen.rs), [`Board::rerender_move_cache`](src/engine/move_gen.rs)), plus [src/engine/capture.rs](src/engine/capture.rs), [src/engine/quiet.rs](src/engine/quiet.rs)
  - UCI helpers: [src/engine/uci.rs](src/engine/uci.rs) (e.g., [`Board::encode_uci_move`](src/engine/uci.rs), [`Board::decode_uci_move`](src/engine/uci.rs))
  - SAN placeholder: [src/engine/san.rs](src/engine/san.rs)
- Game orchestration
  - Controller and game mode: [src/game/controller.rs](src/game/controller.rs)
  - Stockfish command thread: [src/game/stockfish_engine.rs](src/game/stockfish_engine.rs) (e.g., [`StockfishCmd`](src/game/stockfish_engine.rs), [`StockfishResult`](src/game/stockfish_engine.rs), [`MyApp::start_stockfish`](src/game/stockfish_engine.rs))
  - Evaluation worker and API: [src/game/evaluator.rs](src/game/evaluator.rs) (e.g., [`EvalKind`](src/game/evaluator.rs), `EvaluationRequest`, [`MyApp::start_evaluator`](src/game/evaluator.rs))
- UI
  - App shell and state: [src/ui/app.rs](src/ui/app.rs)
  - Board rendering and inputs: [src/ui/render.rs](src/ui/render.rs) (e.g., [`MyApp::render_board`](src/ui/render.rs), [`MyApp::render_quiet_move`](src/ui/render.rs), [`MyApp::render_capture_move`](src/ui/render.rs), [`MyApp::render_attack_move`](src/ui/render.rs), [`MyApp::render_move_history`](src/ui/render.rs), [`MyApp::render_eval_bar`](src/ui/render.rs))
  - Board interactions: [src/ui/board_interaction.rs](src/ui/board_interaction.rs) (e.g., [`MyApp::handle_board_interaction_logic`](src/ui/board_interaction.rs))
  - Screens and layouts: [src/ui/screen_render.rs](src/ui/screen_render.rs)
  - Eval chart: [src/ui/chart_render.rs](src/ui/chart_render.rs) (e.g., [`MyApp::render_eval_chart`](src/ui/chart_render.rs))
  - Theme and assets: [src/ui/theme.rs](src/ui/theme.rs), [assets/](assets/)
- Database
  - Schema and CRUD: [src/database/create.rs](src/database/create.rs) (e.g., [`create_database`](src/database/create.rs), `insert_game_and_get_id`, `insert_single_move`, `get_game_list`)
  - Save workflow: [src/database/save_game.rs](src/database/save_game.rs) (e.g., [`MyApp::save_game`](src/database/save_game.rs))

## What it does now

- Play a local game on a rendered board
  - Click to select and move; right-click deselects (see [`MyApp::handle_board_interaction_logic`](src/ui/board_interaction.rs))
  - POV flip and multiple layouts (Sandbox, Versus, Analyzer) in [src/ui/screen_render.rs](src/ui/screen_render.rs)
- Legal move generation
  - Pseudo and legal moves per piece via [`Board::get_legal_moves`](src/engine/move_gen.rs)
  - Per-piece move cache rebuilt on changes via [`Board::rerender_move_cache`](src/engine/move_gen.rs)
- Stockfish integration
  - Background command loop for best move requests ([`MyApp::start_stockfish`](src/game/stockfish_engine.rs))
  - Separate evaluation worker that accepts BarEval (UI) and MoveEval (per-ply) requests ([`MyApp::start_evaluator`](src/game/evaluator.rs), [`EvalKind`](src/game/evaluator.rs))
- FEN/serialization
  - Load positions via [`fen_parser`](src/engine/fen.rs), serialize with `impl ToString for Board` in [src/engine/fen.rs](src/engine/fen.rs)
- Game history
  - Record moves (UCI, SAN placeholder), evaluation, and metadata in SQLite ([`insert_game_and_get_id`](src/database/create.rs), `insert_single_move`)
  - Replay and analyze past games (UI in [src/ui/screen_render.rs](src/ui/screen_render.rs), analyzer actions in [src/analyzer/board_interactions.rs](src/analyzer/board_interactions.rs))

## What it aims to achieve

- Stable PvE and Sandbox modes with smooth evaluation feedback
- Accurate, performant legal move generation (including castling, en passant, checks)
- Robust game saving and a simple analyzer with eval charts and step-through
- Extendable UI/UX and future multiplayer hooks

## Build and run

Prerequisites:

- Rust toolchain
- Stockfish binary available on PATH (used by the `stockfish` crate)
- Assets present under [assets/](assets/)

Build:

- `cargo build`

Run:

- `cargo run`

On first use, the app will create/use a local SQLite DB file named `chess.db` in the project root (see [src/database/create.rs](src/database/create.rs)).

## Controls and workflows

- Board
  - Click a piece to see moves; click a target square to move
  - Right-click to deselect (see [`MyApp::handle_board_interaction_logic`](src/ui/board_interaction.rs))
- Versus (PvE)
  - Start the Stockfish thread and request engine moves via UI (see [src/game/stockfish_engine.rs](src/game/stockfish_engine.rs))
- Saving
  - Save replays a copy of the game, requests a `MoveEval` per ply, and persists to DB (see [`MyApp::save_game`](src/database/save_game.rs))

## Key internals

- Move cache
  - Rebuilt when `Board.been_modified` is true in [`MyApp::render_board`](src/ui/render.rs) via [`Board::rerender_move_cache`](src/engine/move_gen.rs)
- Evaluation pipeline
  - UI sends BarEval each frame when appropriate ([`MyApp::render_board`](src/ui/render.rs))
  - Saving and one-off requests use MoveEval with reply channels ([`MyApp::save_game`](src/database/save_game.rs))
- UCI helpers
  - Encode/decode moves for DB and analyzer UI ([`Board::encode_uci_move`](src/engine/uci.rs), [`Board::decode_uci_move`](src/engine/uci.rs))

## Known issues and notes

- Receiving on closed channel during save
  - Cause: dropping `MoveEval` replies when coalescing requests in the evaluator leads to closed `reply_to` channels. Process all `MoveEval` and coalesce only the latest `BarEval` (see [`MyApp::start_evaluator`](src/game/evaluator.rs), sender usage in [`MyApp::render_board`](src/ui/render.rs) and [`MyApp::save_game`](src/database/save_game.rs)).
- Performance
  - BarEval every frame can be expensive. Throttle or gate while saving (see gating in [`MyApp::render_board`](src/ui/render.rs)).
- SAN encoding is a stub ([src/engine/san.rs](src/engine/san.rs)).

## Configuration

- Defaults: [src/etc.rs](src/etc.rs) (e.g., `DEFAULT_FEN`, `DEFAULT_STARTING`, `PLAYER_NAME`, `STOCKFISH_ELO`)
- Theme and textures: [src/ui/theme.rs](src/ui/theme.rs)
- UI sizing/padding: [src/ui/ui_setting.rs](src/ui/ui_setting.rs)

## Database schema

Created in [`create_database`](src/database/create.rs):

- games(id, played, starting_fen, black_player, white_player, black_elo, white_elo)
- moves(id, game_id, uci, san, eval_score, eval_type)

## Roadmap

- Solidify evaluator coalescing and cancellation for responsive UI
- Complete SAN and richer PGN export/import
- Analyzer tools (heatmaps, threats) and better history browsing
- Optional multiplayer and opening books

## License

Add a license
