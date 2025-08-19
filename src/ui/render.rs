use eframe::egui::{self, menu, pos2, vec2, Color32, Context, Painter, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Ui, UiBuilder};
use rand::rngs::SmallRng;
use egui_plot::{Line, Plot, PlotPoints};
use crate::{engine::{ChessPiece, PieceColor, PieceType}, game::evaluator::EvalKind, ui::app::MyApp};

#[derive(Clone)]
pub enum BoardLayout {SandboxLayout, VersusLayout, AnalyzerLayout}

impl MyApp{
    pub fn render_board_layout(&mut self, top_left: Pos2, ui: &mut Ui, ctx:&Context, layout: BoardLayout){
        
        match layout {
            BoardLayout::SandboxLayout => {
                self.render_eval_bar(top_left, ui, true);
                self.render_move_history(top_left, ui, true);
                self.render_board(top_left, ui);
                ctx.request_repaint();
        
                if let Some((new_pos, old_pos)) = self.board.ui.promtion_pending {
                    egui::Window::new("Promote Pawn")
                        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                        .collapsible(false)
                        .resizable(false)
                        .show(ctx, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label("Choose piece to promote to:");
                                for kind in [
                                    PieceType::Bishop,
                                    PieceType::Knight,
                                    PieceType::Queen,
                                    PieceType::Rook,
                                ] {
                                    if ui.button(kind.to_string()).clicked() {
                                        //3self.after_move_logic(&MoveInfo {old_pos:old_pos, new_pos:new_pos, promotion:Some(kind), is_capture: false});
                                        self.board.promote_pawn((old_pos, new_pos), kind);
                                        let last_move = self.board.meta_data.move_list.last_mut().unwrap();
                                        last_move.promotion = Some(kind);
                                        
                                        match kind {
                                            PieceType::Queen => last_move.uci.push('q'),
                                            PieceType::Rook => last_move.uci.push('r'),
                                            PieceType::Bishop => last_move.uci.push('b'),
                                            PieceType::Knight => last_move.uci.push('n'),
                                            _ => {}
                                        }
                                        
                                        self.board.ui.promtion_pending = None;
                                        
                                    }
                                }
                            });
                        });
                    }
            }
            BoardLayout::VersusLayout => {
                
            self.render_move_history(top_left, ui, true);
            self.render_board(top_left, ui);
            ctx.request_repaint();
            self.render_game_info(top_left, ui);
        
            if let Some((new_pos, old_pos)) = self.board.ui.promtion_pending {
                egui::Window::new("Promote Pawn")
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label("Choose piece to promote to:");
                            for kind in [
                                PieceType::Bishop,
                                PieceType::Knight,
                                PieceType::Queen,
                                PieceType::Rook,
                            ] {
                                if ui.button(kind.to_string()).clicked() {
                                    //3self.after_move_logic(&MoveInfo {old_pos:old_pos, new_pos:new_pos, promotion:Some(kind), is_capture: false});
                                    self.board.promote_pawn((old_pos, new_pos), kind);
                                    let last_move = self.board.meta_data.move_list.last_mut().unwrap();
                                    last_move.promotion = Some(kind);
                                    
                                    match kind {
                                        PieceType::Queen => last_move.uci.push('q'),
                                        PieceType::Rook => last_move.uci.push('r'),
                                        PieceType::Bishop => last_move.uci.push('b'),
                                        PieceType::Knight => last_move.uci.push('n'),
                                        _ => {}
                                    }
                                    
                                    self.board.ui.promtion_pending = None;
                                    
                                }
                            }
                        });
                    });
                }
                }
                BoardLayout::AnalyzerLayout =>  {
                    self.render_eval_bar(top_left, ui, true);
                    self.render_board_menu(top_left, ui);
                    self.render_move_history(top_left, ui, false);
                    self.render_eval_chart(top_left, ui);
                    self.render_board(top_left, ui);


                    ctx.request_repaint();
                }
        }
        
    }
    pub fn render_board(&mut self, top_left: Pos2, ui: &mut Ui) {
        let avail = ui.available_size();
        self.ui_settings.square_size = (avail.x / 14.0).min(60.0);
        let s = self.ui_settings.square_size;
        let painter = ui.painter();
        
        if self.board.been_modified {
            self.board.rerender_move_cache();
            self.board.been_modified = false;
        }
        // iterate raw geometry 0..7
        let fen = self.board.to_string();
        self.poll_stockfish(fen.clone());   
        self.board.ui.bar_eval = self.get_evaluation();

        for raw_rank in 0..8 {
            for raw_file in 0..8 {
                // map raw -> board coords
                let board_rank = if self.board.ui.pov == PieceColor::Black {
                    7 - raw_rank
                } else {
                    raw_rank
                };
                let board_file = if self.board.ui.pov == PieceColor::Black {
                7 - raw_file
            } else {
                raw_file
            };
            
            // compute screen‐position from the *raw* coords
            let x = top_left.x + (raw_file as f32) * s;
            let y = top_left.y + (raw_rank as f32) * s;
            let rect = Rect::from_min_size(pos2(x, y), vec2(s, s));
            let id = ui.make_persistent_id((raw_rank, raw_file));
            let response = ui.interact(rect, id, Sense::click_and_drag());
            
            // checker‐pattern based on raw geometry
            let base = if (raw_rank + raw_file) % 2 == 0 {
                self.theme.light_square
            } else {
                self.theme.dark_square
            };
            let color = if response.hovered() {
                base.to_opaque().linear_multiply(1.1)
            } else {
                base.to_opaque()
            };
            painter.rect_filled(rect, 0.0, color);
            
            // pull the piece out of the mapped board cell
            let piece = self.board.squares[board_rank][board_file];
            self.render_selected(&piece, &rect, painter);
            self.render_piece(&piece, &rect, &painter);
            self.render_quiet_move(&(board_rank as u8, board_file as u8), &rect, &painter);
            self.render_capture_move(&(board_rank as u8, board_file as u8), &rect, &painter);
            self.handle_board_interaction_logic(
                &(board_rank as u8, board_file as u8),
                &response);
        }
    }
    // Only enqueue BarEval when not saving (avoid starving MoveEval)
    if !self.ui_controller.saving_game || self.board.been_modified{
        self.evaluator.send_eval_request(fen.clone(), EvalKind::BarEval);
    }
}

pub fn render_quiet_move(&self, poz :&(u8, u8), rect: &Rect, painter: &Painter){
    match &self.board.ui.selected_piece{
        Some(piece) => {
            let moves = &self.board.move_cache.get(&piece.id);
            match moves{
                Some(moves) => {
                    // use quiet_moves for quiet indicators
                    if moves.quiet_moves.contains(poz) {
                        let center = rect.center();
                        let radius = self.ui_settings.square_size * 0.225;
                        painter.circle_filled(
                            center,
                            radius, 
                            if (poz.0 + poz.1) % 2 == 0 {
                                self.theme.light_pseudo_move_highlight
                            } else {
                                self.theme.dark_pseudo_move_highlight
                            }
                        );
                    } else { return; }
                }
                None => {}
            }
        }
        None => { return; }
    }
}
pub fn render_capture_move(&self, poz :&(u8, u8), rect: &Rect, painter: &Painter){
    match &self.board.ui.selected_piece{
        Some(piece) => {
            let moves = &self.board.move_cache.get(&piece.id);
            match moves{
                Some(moves) => {
                    if moves.capture_moves.contains(poz) {
                        let center = rect.center();
                        let radius = self.ui_settings.square_size * 0.225;
                        painter.circle_stroke(
                            center,
                            radius,
                            Stroke::from((
                                2.5,
                                if (poz.0 + poz.1) % 2 == 0 {
                                    self.theme.light_pseudo_move_highlight
                                } else {
                                    self.theme.dark_pseudo_move_highlight
                                },
                            )),
                        );
                    }
                    else {return;}
                }
                None => {}
            }
        }
        None => {return;}
    }
}
pub fn render_selected(&self, piece: &Option<ChessPiece>, rect: &Rect, painter: &Painter){
    match piece {    
        Some(p) =>{
            if let Some(selected_piece) = self.board.ui.selected_piece {
                if p.position == selected_piece.position {
                    painter.rect_filled(*rect, 0.0, self.theme.square_select_highlight);
                }
            }

            if let Some(pos) = self.board.ui.moved_piece {
                if p.position == pos {
                    painter.rect_filled(*rect, 0.0, self.theme.moved_from_highlight.to_opaque());
                }
            }

            // highlight king in check or checkmate
            if let Some(pos) = self.board.ui.checkmate_square {
                if p.position == pos {
                    painter.rect_filled(*rect, 0.0, self.theme.checkmate_square);
                }
            }


        }
        None => {

        }
    }
}
pub fn render_piece(&self, piece: &Option<ChessPiece>, rect: &Rect, painter: &Painter){
    match piece {
        Some(p) =>{
            painter.image(
            match self.theme.piece_map.get(&(p.kind, p.color)) {
                Some(rez) => {
                    match rez {
                        Ok(texture) => {
                            texture.id()
                        }
                        Err(err) =>{
                            self.theme.empty_texture.id()
                        }
                    }
                }
                None => {
                    self.theme.empty_texture.id()
                }
            },
            *rect,
            Rect { min: Pos2 { x: 0.0, y: 0.0 }, max: Pos2{ x: 1.0, y:1.0}},
            Color32::WHITE
        );
    }
    None => {

    }
}
}
pub fn render_eval_bar(&self, top_left: Pos2, ui: &mut Ui, is_visible: bool){
    if !is_visible {return;}
    let painter = ui.painter();
    let bar_height = 8.0 * self.ui_settings.square_size;
    let bar_width = 20.0;

    let bar_color = Color32::BLACK;
    let eval_color = Color32::WHITE;

    let eval_x = top_left.x - 20.0 - self.ui_settings.padding as f32;

    // POV-adjusted centipawns, clamped
    let mut cp = self.board.ui.bar_eval;
    if self.board.ui.pov == PieceColor::Black {
        cp = -cp;
    }
    let cp = cp.clamp(-1000.0, 1000.0); // -10 to +10 pawns

    // Map [-1000..1000] to [-bar_height/2..bar_height/2]
    let delta = (cp / 1000.0) * (bar_height / 2.0);
    let eval_height = (bar_height / 2.0) + delta;

    // For White POV, the white area grows from bottom; for Black, from top
    let eval_y = if self.board.ui.pov == PieceColor::White {
        top_left.y + bar_height - eval_height
    } else {
        top_left.y
    };

    // Base bar
    let bar_rect = Rect::from_min_size(
        pos2(top_left.x - 20.0 - self.ui_settings.padding as f32, top_left.y),
        vec2(bar_width, bar_height),
    );
    painter.rect_filled(bar_rect, 5.0, bar_color);

    // Eval fill
    let eval_rect = Rect::from_min_size(pos2(eval_x, eval_y), vec2(bar_width, eval_height));
    painter.rect_filled(eval_rect, 5.0, eval_color);

    // Midline
    painter.rect_filled(
        Rect::from_min_size(
            pos2(top_left.x - 40.0 - self.ui_settings.padding as f32, top_left.y + bar_height/2.0),
            vec2(bar_width, bar_height/8.0),
        ),
        0.0,
        Color32::RED,
    );
}
pub fn render_move_history(&self, top_left: Pos2, ui: &mut Ui, is_visible: bool){
    if !is_visible {return;}
    let mut k=0;
    let test_moves = &self.board.meta_data.move_list;
      
    let painter = ui.painter();
    let bar_height = self.ui_settings.square_size * 8.0;
    let bar_width = self.ui_settings.square_size*2.0;
    let pad = self.ui_settings.padding as f32;
    let move_text_size = (bar_width * 0.15).min(12.0);
    let (history_x, history_y) = (top_left.x + self.ui_settings.square_size*8.0+pad, top_left.y);
    let rect = Rect::from_min_size(pos2(history_x, history_y), vec2(bar_width, bar_height));
    painter.rect_filled(rect, 0.0, Color32::BLACK);
    ui.allocate_new_ui(
        UiBuilder::default().max_rect(rect),
        |ui| {
            ui.vertical(|ui| {
                
                let move_rect_width = ui.available_width()*0.4;

                let move_rect_height = bar_width * 0.2;
                
                let square_row_count = (bar_height / move_rect_height).round() as usize;
                
                for row in 0..square_row_count {
                    let row_num_rect = Rect::from_min_size(
                        pos2(history_x, history_y+pad + (move_rect_height * row as f32)), 
                        vec2(bar_width*0.2, move_rect_height));

                    let ply1_rect = Rect::from_min_size(
                        pos2(history_x+bar_width*0.2, history_y+ pad + (move_rect_height * row as f32)), 
                        vec2(move_rect_width, move_rect_height));
                    let ply2_rect = Rect::from_min_size(
                        pos2(history_x +bar_width*0.2+ move_rect_width, history_y+ pad + (move_rect_height * row as f32)), 
                        vec2(move_rect_width, move_rect_height));
                   
                    let painter = ui.painter();
                    painter.rect_filled(row_num_rect, 0.0, Color32::BLACK);
                    painter.rect_filled(ply1_rect, 0.0, Color32::BLACK);
                    painter.rect_filled(ply2_rect, 0.0, Color32::BLACK);
                    ui.allocate_new_ui(UiBuilder::default().max_rect(ply1_rect), |ui|{
                        ui.horizontal_centered(|ui|{
                            ui.add_space(pad);
                            if k < test_moves.len() {
                                ui.label(RichText::new(test_moves[k].uci.clone()).size(move_text_size));
                            }
                        })
                    });
                    k += 1;
                    ui.allocate_new_ui(UiBuilder::default().max_rect(ply2_rect), |ui|{
                        ui.horizontal_centered(|ui|{
                            ui.add_space(pad);
                            if k < test_moves.len() {
                                ui.label(RichText::new(test_moves[k].uci.clone()).size(move_text_size));
                            }
                        })
                    });
                    ui.allocate_new_ui(UiBuilder::default().max_rect(row_num_rect), |ui|{
                        ui.horizontal_centered(|ui|{
                            ui.add_space(pad);
                            if k <= test_moves.len() {
                                ui.label(RichText::new(format!("{}.", row)).size(move_text_size));
                            }
                        })
                    });
                    k+=1;
                    
                    
                }
                
                
            });
        },
    );
    

}
    pub fn render_board_menu(&mut self, top_left: Pos2, ui: &mut Ui) {
        let board_square = self.ui_settings.square_size;
        let board_size = board_square* 8.0;
        let pad = self.ui_settings.padding as f32;
        let menu_size = board_square * 0.8;
        let menu_x =top_left.x;
        let menu_y = top_left.y - pad - menu_size;

        let menu_area = Rect::from_min_size(pos2(menu_x,menu_y), vec2(board_size, menu_size));
        ui.painter().rect_filled(menu_area, 2.0, Color32::RED);
        ui.allocate_new_ui(UiBuilder::default().max_rect(menu_area), |ui| {
            ui.scope(|ui| {
            ui.style_mut().spacing.item_spacing.x = 0.0;
            ui.horizontal_centered(|ui| {
                 
                if self.colourd_image_button(vec2(menu_size, menu_size*0.9), &self.theme.ui.prev_button, Color32::DARK_GRAY, ui).clicked(){
                    match self.analyzer.current_ply.checked_sub(1) {
                        Some(ply_num) => {
                            self.analyzer.current_ply = ply_num;
                            match self.board.meta_data.move_list.get(ply_num as usize) {
                                Some(ply) => {  
                                    let res = self.board.undo_move(ply.uci.clone());
                                    
                                }
                                None => {}
                            }
                        }
                        None => {

                        }
                    }
                };
                if self.colourd_image_button(vec2(menu_size, menu_size*0.9), &self.theme.ui.next_button, Color32::DARK_GRAY, ui).clicked(){
                     let ply = self.board.meta_data.move_list.get(self.analyzer.current_ply as usize);
                    match ply {
                        Some(ply) => {
                            let res = self.board.do_move(ply.uci.clone());
                            if res.is_ok() {
                                self.analyzer.current_ply += 1;
                            }
                        }
                        _ => {}
                    }
                };
                if self.colourd_image_button(vec2(menu_size, menu_size*0.9), &self.theme.ui.heat_button, Color32::DARK_GRAY, ui).clicked(){

                };
                if self.colourd_image_button(vec2(menu_size, menu_size*0.9), &self.theme.ui.danger_button, Color32::DARK_GRAY, ui).clicked(){

                };
                
            });
            })
        });
        
    }

pub fn render_eval_chart(&self, top_left: Pos2, ui: &mut eframe::egui::Ui) {
        let board_square = self.ui_settings.square_size;
        let board_size = board_square* 8.5;
        let pad = self.ui_settings.padding as f32;
        let chart_x =top_left.x - board_square*0.5;
        let chart_y = top_left.y + pad+ board_size;
        let chart_height = board_square * 2.5;
        
        let chart_area = Rect::from_min_size(pos2(chart_x, chart_y), vec2(board_size, chart_height));
        
        let chart_color = Color32::BLACK;
        
        ui.painter().rect_filled(chart_area, 1.0, Color32::DARK_GRAY);
        ui.allocate_new_ui(UiBuilder::default().max_rect(chart_area), |ui| {
            let ox_start = pos2(chart_area.left()+pad, chart_area.center().y);
            let ox_stop: Pos2 = pos2(chart_area.right()-pad, chart_area.center().y);

            let oy_start = pos2(chart_area.left()+ 20.0, chart_area.center().y - chart_height/2.0+pad);

            let oy_stop = pos2(chart_area.left()+20.0, chart_area.center().y + chart_height/2.0 - pad);


            ui.painter().line_segment([ox_start, ox_stop], Stroke::new(2.0, chart_color));

            ui.painter().line_segment([oy_start, oy_stop], Stroke::new(2.0, chart_color));
        });
        
    }

}