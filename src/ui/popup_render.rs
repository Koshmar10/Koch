use eframe::egui;

use crate::{database::{create::{insert_game_and_get_id, insert_single_move}, save_game}, engine::{Board, PieceType}, ui::{app::{MyApp, PopupType}, DEFAULT_FEN}};
use crate::database::create;


impl MyApp {
    pub fn popup_handler(&mut self, popup: &PopupType, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match popup {
            PopupType::GameLostPopup(msg) =>{
                egui::Window::new("Game lost")
                            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                            .collapsible(false)
                            .resizable(false)
                            .show(ctx, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(msg);
                                    if ui.button("x").clicked(){
                                        self.popup = None;
                                        self.board = Board::from(&DEFAULT_FEN.to_owned());
                                        self.game.game_over = true;
                                    }
                                    let mut saved = false;
                                    if ui.button("save game").clicked() {
                                        if !saved {
                                            self.save_game();
                                        }
                                        saved = true; 
                                    }
                                });
                            });
            }
            PopupType::SavingGamePopup => {
                 egui::Window::new("Saving game")
                            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                            .collapsible(false)
                            .resizable(false)
                            .show(ctx, |ui| {
                                ui.vertical_centered(|ui| {
                                    
                                    ui.label("Saving Game");
                                    self.poll_save_game_worker();
                                });
                            });
            }
        }
    }

}