use eframe::egui::{self, pos2, vec2, Color32, CornerRadius, Pos2, Rect, Stroke, Ui, UiBuilder, Vec2};

use crate::{engine::PieceType, etc::STOCKFISH_ELO, ui::app::MyApp};

impl MyApp{
    pub fn render_game_info(&mut self, top_left: Pos2, ui : &mut Ui) {
        
        let unit =  &self.ui_settings.square_size;
        //settings
        let timer_size = Vec2::new(2.0*unit, *unit*2.0 /3.0);
        let pad = 12.0;
        let pfp_size = 1.5*unit;
        let label_size = unit *2.0;
        
        

        let white_box = Rect::from_min_size(
            pos2(top_left.x + 6.0 * unit, top_left.y - timer_size.y - pad),
            timer_size,
        );
        
        let black_box = Rect::from_min_size(
            pos2(top_left.x + 6.0 * unit, top_left.y + 8.0*unit + pad),
            timer_size,
        );
        let white_player_pfp = Rect::from_min_size (
            pos2(top_left.x, top_left.y - pfp_size - pad),
            Vec2::new(pfp_size, pfp_size)
        );
        let white_player_label = Rect::from_min_size(
            pos2(top_left.x + pfp_size + pad, top_left.y - pfp_size - pad), 
            Vec2::new(label_size*1.5, label_size/3.0));
        
        let white_player_pieces = Rect::from_min_size(
            pos2(top_left.x + pfp_size + pad, top_left.y - label_size/3.0 - pad), 
            Vec2::new(label_size*2.0, label_size/3.0));

        for &rect in &[white_box, black_box, white_player_pfp, white_player_label] {
            ui.painter().rect_filled(rect, CornerRadius::same(4), self.ui_settings.timer_inside);
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(4),
                Stroke::new(1.0, self.ui_settings.timer_outside),
                egui::StrokeKind::Outside
            );
        }
        ui.painter().image(
            self.theme.white_pfp.as_ref().unwrap_or(&self.theme.empty_texture).id(), white_player_pfp, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
        
        ui.allocate_new_ui(UiBuilder::new().max_rect(white_player_label), |ui| {
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(16.0, egui::FontFamily::Proportional)
            );
            ui.horizontal_centered(|ui| {
                ui.label(format!("{}({})", self.board.meta_data.white_player_name, self.board.meta_data.white_player_elo));
            });
        });
            ui.allocate_new_ui(UiBuilder::new().max_rect(white_box), |ui| {
                ui.style_mut().text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(18.0, egui::FontFamily::Proportional)
                );
                ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new("10:00").strong());
                });
            });
            ui.allocate_new_ui(UiBuilder::new().max_rect(white_player_pieces), |ui| {
                let initial_render_position = white_player_pieces.min;
                let mut x_offset = 0.0;
                let max_piece_size = (white_player_pieces.height().min(white_player_pieces.width() / 4.0)) * 0.8 ; // Made 30% smaller
                ui.horizontal_centered(|ui| {

                    if !self.board.ui.white_taken.is_empty() {
                        self.board.ui.white_taken.sort_by_key(|p| {
                            match p.0 {
                                PieceType::Bishop => 31,
                                PieceType::King => 120,
                                PieceType::Knight => 30,
                                PieceType::Pawn => 10,
                                PieceType::Queen => 90,
                                PieceType::Rook => 50,
                            }
                        });
                        for piece in &self.board.ui.white_taken {

                            let piece_rect = Rect::from_min_size(
                                pos2(
                                    initial_render_position.x + x_offset, 
                                    initial_render_position.y + (white_player_pieces.height() - max_piece_size) / 2.0
                                ),
                                vec2(max_piece_size, max_piece_size)
                            );
                            
                        
                        if let Some(texture) = self.theme.piece_map.get(&(piece.0, piece.1)) {
                            match texture {
                                Ok(tex) => {
                                  
                                    // Render the original piece on top
                                    ui.painter().image(
                                        tex.id(),
                                        piece_rect,
                                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                        Color32::WHITE
                                    );
                                }
                                Err(_) => {
                                    ui.painter().image(
                                        self.theme.empty_texture.id(),
                                        piece_rect,
                                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                        Color32::WHITE
                                    );
                                }
                            }
                        }
                        
                        
                        x_offset += piece_rect.width()/2.5;
                    }
                }
            });
            });
            ui.allocate_new_ui(UiBuilder::new().max_rect(black_box), |ui| {
                ui.style_mut().text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(18.0, egui::FontFamily::Proportional)
                );
                ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new("10:00").strong());
                });
            });
            let black_player_pfp = Rect::from_min_size (
                pos2(top_left.x, top_left.y + 8.0*unit + pad),
                Vec2::new(pfp_size, pfp_size)
            );

            let black_player_label = Rect::from_min_size(
                pos2(top_left.x + pfp_size + pad, top_left.y + 8.0*unit + pad), 
                Vec2::new(label_size*1.5, label_size/3.0));

            let black_player_pieces = Rect::from_min_size(
                pos2(top_left.x + pfp_size + pad, top_left.y + 8.0*unit + label_size/3.0 + pad), 
                Vec2::new(label_size*2.0, label_size/3.0));

            for &rect in &[black_player_pfp, black_player_label] {
                ui.painter().rect_filled(rect, CornerRadius::same(4), self.ui_settings.timer_inside);
                ui.painter().rect_stroke(
                    rect,
                    CornerRadius::same(4),
                    Stroke::new(1.0, self.ui_settings.timer_outside),
                    egui::StrokeKind::Outside
                );
            }

            ui.painter().image(
                self.theme.black_pfp.as_ref().unwrap_or(&self.theme.empty_texture).id(), 
                black_player_pfp, 
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), 
                Color32::WHITE
            );

            ui.allocate_new_ui(UiBuilder::new().max_rect(black_player_label), |ui| {
                ui.style_mut().text_styles.insert(
                    egui::TextStyle::Body,
                    egui::FontId::new(16.0, egui::FontFamily::Proportional)
                );
                ui.horizontal_centered(|ui| {
                    ui.label(format!("{}({})", self.board.meta_data.black_player_name, self.board.meta_data.black_player_elo));
                });
            });

            ui.allocate_new_ui(UiBuilder::new().max_rect(black_player_pieces), |ui| {
                let initial_render_position = black_player_pieces.min;
                let mut x_offset = 0.0;
                let max_piece_size = (black_player_pieces.height().min(black_player_pieces.width() / 4.0)) * 0.8;
                ui.horizontal_centered(|ui| {
                    if !self.board.ui.black_taken.is_empty() {
                        self.board.ui.black_taken.sort_by_key(|p| {
                            match p.0 {
                                PieceType::Bishop => 31,
                                PieceType::King => 120,
                                PieceType::Knight => 30,
                                PieceType::Pawn => 10,
                                PieceType::Queen => 90,
                                PieceType::Rook => 50,
                            }
                        });
                        for piece in &self.board.ui.black_taken {
                            let piece_rect = Rect::from_min_size(
                                pos2(
                                    initial_render_position.x + x_offset, 
                                    initial_render_position.y + (black_player_pieces.height() - max_piece_size) / 2.0
                                ),
                                vec2(max_piece_size, max_piece_size)
                            );
                        
                            if let Some(texture) = self.theme.piece_map.get(&(piece.0, piece.1)) {
                                match texture {
                                    Ok(tex) => {
                                        ui.painter().image(
                                            tex.id(),
                                            piece_rect,
                                            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                            Color32::WHITE
                                        );
                                    }
                                    Err(_) => {
                                        ui.painter().image(
                                            self.theme.empty_texture.id(),
                                            piece_rect,
                                            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                            Color32::WHITE
                                        );
                                    }
                                }
                            }
                            
                            x_offset += piece_rect.width()/2.5;
                        }
                    }
                });
            });
        
        

}
}