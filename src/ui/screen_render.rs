use eframe::{egui::{self, vec2, Button, CentralPanel, Color32, CornerRadius, Frame, RichText, SidePanel, Stroke}, App};

use crate::{analyzer::board_interactions::AnalyzerController, database::{create::{destroy_database, get_game_list}, save_game::SaveType}, engine::{board::{BoardMetaData, MoveInfo}, fen::fen_parser, Board, PieceColor, PieceType}, etc::{PLAYER_NAME, STOCKFISH_ELO}, game::{self, controller::GameMode, stockfish_engine::StockfishCmd}, ui::{app::{AppScreen, HistoryScreenVariant, MyApp, PopupType}, render::BoardLayout, DEFAULT_FEN}};
use crate::engine::board::MoveStruct;


impl MyApp{
    pub fn render_main_menu(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame){
            
            // Take up one third of the available width:
            let third = ctx.available_rect().width() / 6.0;
            SidePanel::left("left-separator")
                .min_width(third)
                .max_width(third)
                .show(ctx, |_| {});
            SidePanel::right("right-separator")
            .min_width(third)
            .max_width(third)
            .show(ctx, |_| {});
        
            
            CentralPanel::default()
                .frame(Frame::default().fill(Color32::BLACK))
                .show(ctx, |ui| {
                    // Center both horizontally and vertically:
                    
                    ui.add_space(ui.available_height()* 0.2);
                    let scale = if ui.available_width() < 400.0 { 0.8 } else { 1.0 };
                        ctx.request_repaint();
                        let title_size = self.ui_settings.title_size * scale;
                        let subtitle_size = self.ui_settings.subtitle_size * scale;
                    ui.vertical_centered(|ui| {
                        // Scale down to 80% if the available width is small
                       

                        // Title: bold & large
                        ui.label(
                            egui::RichText::new("Koch")
                                .heading()
                                .strong()
                                .size(title_size),
                        );
                        ui.add_space(12.0 * scale);

                        // Subtitle: smaller & fainter
                        // Before calling ui.label, compute a dynamic size based on the quote’s length:
                        let quote = match &self.ui_settings.menu_quote {
                            None => {
                                self.ui_settings.menu_quote = Some(self.get_quote());
                                self.ui_settings.default_subtitle.clone()
                            }
                            Some(s) => s.clone(),
                        };
                        let len = quote.chars().count().max(1) as f32;
                        // we cap the width we expect it to occupy
                        let max_text_width = ui.available_width() * 0.8;
                        // derive a font size that shrinks for longer text, clamped to a reasonable range
                        let computed_size = (max_text_width / len * 1.5).clamp(12.0, subtitle_size);

                        ui.label(
                            egui::RichText::new(quote)
                                .weak()
                                .size(computed_size),
                        );
                        ui.add_space(12.0 * scale);
                    });
                    
                    ui.vertical_centered(|ui|{
                        let button_width = ui.available_width()*0.48;
                        
                        let train_btn = Button::new(egui::RichText::new("Train").raised().strong().size(18.0))
                        .corner_radius(CornerRadius::from(5.0))
                        .min_size(vec2(button_width, 40.0)); 
                        
                        ui.add_space(12.0 * scale);

                        let history_btn = Button::new(egui::RichText::new("Game History").raised().strong().size(18.0))
                        .corner_radius(CornerRadius::from(5.0))
                        .min_size(vec2(button_width, 40.0)); 

                        ui.add_space(12.0 * scale);

                        if ui.add(train_btn).clicked() {
                            self.screen = AppScreen::TrainWithAi;
                        }
                        ui.add_space(4.0);
                        if ui.add(history_btn).clicked() {
                            self.screen = AppScreen::History(HistoryScreenVariant::PastGameSelectionView);
                        }
                            
                    } );
                    
                   
                });
                }
            
            

    
    pub fn render_train_with_ai(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame){
        SidePanel::left("menu")
            .resizable(true)
            .min_width(250.0)
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Chess");
                ui.label(&self.board.to_string());
                ui.label(format!(
                    "piesa: {:?}",
                    match self.board.ui.selected_piece {
                        Some(p) => self.board.squares[p.position.0 as usize][p.position.1 as usize],
                        None => None,
                    }
                ));
                ui.label(format!(
                    "passant square: {:?}",
                    match self.board.en_passant_target {
                        Some(p) => self.board.squares[p.0 as usize][p.1 as usize],
                        None => None,
                    }
                ));
                if ui.button("menu").clicked(){
                    self.screen = AppScreen::MainMenu;
                }
                if ui.button("flip").clicked() {
                    self.board.ui.pov = match self.board.ui.pov {
                        PieceColor::White => PieceColor::Black,
                        PieceColor::Black => PieceColor::White,
                    };
                    ctx.request_repaint();
                };
                if ui.button("save game").clicked() {
                    self.popup = Some(PopupType::SavingGamePopup);
                    match self.game.mode {
                        GameMode::PvE => {
                            self.start_save_game_sequence(SaveType::VersusSave{player: self.game.player});
                        }   
                        GameMode::Sandbox =>  {self.start_save_game_sequence(SaveType::SandboxSave);}
                        _ => {}
                    }
                    
                }
                if ui.button("reset-board").clicked() {
                    self.board = Board::from(&DEFAULT_FEN.to_owned());
                };
                ui.separator();
                if ui.button("gameMode: Sandbox").clicked() {
                    self.game.mode = GameMode::Sandbox;
                    self.ui_controller.board_layout = BoardLayout::SandboxLayout;
                }
                if ui.button("gameMode: PvE").clicked() {
                    self.game.mode = GameMode::PvE;
                    self.ui_controller.board_layout = BoardLayout::VersusLayout;
                }
                if ui.button("start-game").clicked() {
                   
                    self.board = Board::from(&DEFAULT_FEN.to_owned());
                    self.board.meta_data.starting_position = DEFAULT_FEN.to_owned();
                    self.board.meta_data.date = chrono::Local::now().format("%Y.%m.%d").to_string();
                    self.game.game_over = false;
                    let colors = [PieceColor::White, PieceColor::Black];
                    let player_color = colors[rand::random::<i32>() as usize % 2];
                    self.game.player = player_color;
                    self.game.enemey = match player_color {
                        PieceColor::White => {
                            self.board.meta_data.white_player_elo  = 800; // to be modified
                            self.board.meta_data.white_player_name = PLAYER_NAME.to_owned(); 
                            self.board.meta_data.black_player_elo = STOCKFISH_ELO;
                            self.board.meta_data.black_player_name = "Stockfish".to_owned();
                            PieceColor::Black}
                            ,
                        PieceColor::Black => {
                            
                            self.board.meta_data.white_player_elo  = STOCKFISH_ELO; // to be modified
                            self.board.meta_data.white_player_name = "Stockfish".to_owned(); 
                            self.board.meta_data.black_player_elo = 800;
                            self.board.meta_data.black_player_name = PLAYER_NAME.to_owned();
                            
                            PieceColor::White},
                    };
                    if self.board.ui.pov != self.game.player {
                        self.board.ui.pov = match self.board.ui.pov {
                            PieceColor::White => PieceColor::Black,
                            PieceColor::Black => PieceColor::White,
                        };
                        ctx.request_repaint();
                    }
                    
                    self.start_stockfish();       // ← start the cmd_rx loop right away
                    
                    
                    
                };
                ui.label(format!("{:?}", self.game.game_over));
                ui.label(format!("{:?}{:?}", self.game.player, self.game.enemey));
                if ui.button("end-game").clicked() {
                    self.board = Board::from(&DEFAULT_FEN.to_owned());
                    self.game.game_over = true;
                    if let Some(tx) = &self.game.stockfish_tx {
                        if let Err(e) = tx.send(StockfishCmd::Stop) {
                            eprintln!("failed to send `stop` to stockfish: {}", e);
                        }
                    }
                }
                
                ui.label(format!("current eval: {}", self.get_evaluation()));

                ui.vertical(|ui| {
                    let check = if self.board.is_in_check(PieceColor::White) {"true"} else {"false"};
                    ui.label(format!("white_check: {}", check));
                    let check = if self.board.is_in_check(PieceColor::Black) {"true"} else {"false"};
                    ui.label(format!("black_check: {}", check));
                    let stale:bool = self.board.is_stale_mate();
                    ui.label(format!("satelmate {:?}", stale));
                    let mate :bool = self.board.is_chackmate();
                    ui.label(format!("checkmate {:?}", mate));

                });
                ui.label(format!("{:?}", self.board.ui.pov));
                
            
            });
            CentralPanel::default() .frame(
        Frame::default()
            .fill(Color32::from_rgb(0x30, 0x30, 0x30))        // your background color            // optional corner rounding
            .stroke(Stroke::new(0.2, Color32::WHITE))   // optional border
    ).show(ctx, |ui|{
                    //UI SETUP
                    ui.spacing_mut().item_spacing= vec2(0.0, 0.0);
                    
                    let board_size = &self.ui_settings.square_size * 8.0;
                    // Find top-left of centered board
                    let top_left = egui::pos2(
                        ui.min_rect().center().x - board_size / 2.0,
                        ui.min_rect().center().y - board_size / 2.0,
                    );
                    self.render_board_layout(top_left, ui, ctx, self.ui_controller.board_layout.clone());
                    
                    ctx.request_repaint();
                    if let Some(popup) = &self.popup {
                        self.poll_save_game_worker();
                        ctx.request_repaint();
                    }
                    

                });
    }
    pub fn render_past_game_selection_view(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
       let games_to_render: Vec<BoardMetaData> = match get_game_list(){
        Ok(x) => x,
        Err(e) => {println!("{}", e); Vec::new()},
       };
       
       let pad = self.ui_settings.padding as f32;
       //get games TODO
       CentralPanel::default()
                .frame(Frame::default().fill(Color32::BLACK))
                .show(ctx, |ui| {
                    ui.vertical(|ui |{
                        ui.add_space(pad);
                        ui.horizontal(|ui|{
                            ui.add_space(pad);
                            if ui.button("Back").clicked() {
                                self.screen = AppScreen::MainMenu
                            }
                            if ui.button("Delete DB").clicked() {
                                destroy_database();
                            }
                        });
                        if !games_to_render.is_empty() {
                            for game in games_to_render {
                                if ui.label(format!(
                                    "{} vs {} on {}",
                                    game.white_player_name, game.black_player_name, game.date
                                )).clicked(){
                                    self.screen=AppScreen::History(HistoryScreenVariant::GameAnalyzerView(game));
                                }
                            }
                        
                            
                        }
                        else{
                            ui.label(
                                RichText::new("No chess games played")
                                    .size(16.0),
                            );
                        }

                    })
                });
    }
    pub fn render_analyzer_view(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame, metadata: &BoardMetaData){
        let pad = self.ui_settings.padding;
        self.board.meta_data = metadata.clone();
        SidePanel::right("cha_panel").frame(Frame::default().fill(Color32::DARK_GRAY))
        .resizable(true)
        .min_width(350.0)
        .show(ctx, |ui| {
            ui.label("this where the chat  box will be");
        });
        CentralPanel::default()
                .frame(Frame::default().fill(Color32::BLACK))
                .show(ctx, |ui| {
                    
                    ui.vertical_centered_justified(|ui|{
                        ui.horizontal(|ui| {
                            if ui.button("back").clicked() {
                                self.analyzer = AnalyzerController::default();
                                self.board.set_fen((&DEFAULT_FEN).to_string());
                                self.screen = AppScreen::History(HistoryScreenVariant::PastGameSelectionView);
                            }
                        });
                        ui.add_space(pad as f32);
                        ui.label(RichText::new(format!("Date: {}", metadata.date)).size(16.0));
                        ui.add_space(pad as f32);

                        ui.label(RichText::new(format!("White: {} (ELO: {})", 
                            metadata.white_player_name, metadata.white_player_elo)).size(16.0));
                        ui.add_space(pad as f32);

                        ui.label(RichText::new(format!("Black: {} (ELO: {})",
                            metadata.black_player_name, metadata.black_player_elo)).size(16.0));
                        ui.add_space(pad as f32);

                        ui.label(RichText::new("Starting Position:").size(16.0));
                        ui.label(RichText::new(metadata.starting_position.clone()).size(14.0));
                        ui.add_space(pad as f32);

                        // For the move list, we'll use a wrappable horizontal container
                        ui.label(RichText::new("Moves:").size(16.0));
                        ui.add_space(pad as f32);

                        // Wrappable horizontal container for move list
                        egui::ScrollArea::horizontal().show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                // Adjust this based on how moves are stored in your BoardMetaData
                                if !metadata.move_list.is_empty() {
                                    for (i, chess_move) in metadata.move_list.iter().enumerate() {
                                        ui.label(RichText::new(format!("{:?}. {}, eval:{}{}", i+1, chess_move.uci, chess_move.evaluation.kind,chess_move.evaluation.value, )).size(14.0));
                                        ui.add_space(5.0);
                                    }
                                } else {
                                    ui.label(RichText::new("No moves recorded").size(14.0));
                                }
                            });
                        });
                    });
                    
                    let board_size = &self.ui_settings.square_size * 8.0;
                    let top_left = egui::pos2(
                        ui.min_rect().center().x - board_size / 2.0,
                        ui.min_rect().center().y - board_size / 2.0,
                    );
                    self.render_board_layout(top_left, ui, ctx, BoardLayout::AnalyzerLayout);
                    egui::Window::new("floating window").collapsible(true).resizable(true).show(ctx, |ui|{
                        ui.label("window");
                        ui.label(self.analyzer.current_ply.to_string());
                    });
                });

    }
}