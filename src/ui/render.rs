use eframe::egui::{pos2, vec2, Color32, Painter, Pos2, Rect, RichText, Sense, Stroke, Ui, UiBuilder};

use crate::{engine::{ChessPiece, PieceColor}, ui::app::MyApp};

impl MyApp{

    pub fn render_board(&mut self, top_left: Pos2, ui: &mut Ui) {
        let avail = ui.available_size();
        self.ui.square_size = (avail.x / 14.0).min(60.0);
        let s = self.ui.square_size;
        let painter = ui.painter();
        
        // iterate raw geometry 0..7
        for raw_rank in 0..8 {
            for raw_file in 0..8 {
                // map raw -> board coords
                let board_rank = if self.board.state.pov == PieceColor::Black {
                    7 - raw_rank
                } else {
                    raw_rank
                };
                let board_file = if self.board.state.pov == PieceColor::Black {
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
                &piece,
                &(board_rank as u8, board_file as u8),
                &response,
            );
        }
    }
}
pub fn render_quiet_move(&self, poz :&(u8, u8), rect: &Rect, painter: &Painter){
    match &self.board.state.quiet_moves {
        Some(moves) => {
            if moves.contains(poz) {
                let center = rect.center();
                let radius = self.ui.square_size * 0.225;
                painter.circle_filled(center, radius, 
                 if (poz.0 + poz.1) % 2 ==0 {self.theme.light_pseudo_move_highlight} else {self.theme.dark_pseudo_move_highlight});
            }
            else {return;}
        }
        None => {return;}
    }
}
pub fn render_capture_move(&self, poz :&(u8, u8), rect: &Rect, painter: &Painter){
        match &self.board.state.capture_moves {
            Some(moves) => {
                
                if moves.contains(poz) {
                                    let center = rect.center();
                                    let radius = self.ui.square_size * 0.225;
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
                            
                        }
                        None => return,
                    }
                    
                
}
pub fn render_selected(&self, piece: &Option<ChessPiece>, rect: &Rect, painter: &Painter){
    match piece {    
        Some(p) =>{
            match self.board.state.selected_piece{
                Some (selected_piece) => {

                    if p.position == selected_piece.position {
                        painter.rect_filled(*rect, 0.0, self.theme.square_select_highlight);
                    }
                    
                }
                None => {

                }
            }
            match self.board.state.moved_from{
                Some (pos) => {

                    if p.position == pos {
                        painter.rect_filled(*rect, 0.0, self.theme.moved_from_highlight.to_opaque());
                    }
                    
                }
                None => {

                }
            }
            match self.board.state.moved_to{
                Some (pos) => {

                    if p.position == pos {
                        painter.rect_filled(*rect, 0.0, self.theme.moved_from_highlight);
                    }
                    
                }
                None => {

                }
            }
            // highlight king in check or checkmate
            match self.board.state.checkmate_square {
                Some(pos) => {
                    if p.position == pos {
                        painter.rect_filled(*rect, 0.0, self.theme.checkmate_square);
                    }
                }
                None => {}
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
    let bar_height = 8.0 * self.ui.square_size;
    let bar_width = 20.0;

    let bar_color = Color32::BLACK;
    let eval_color = Color32::WHITE;

    let eval_x= top_left.x -20.0 - self.ui.padding as f32;
    let mut eval_y = top_left.y;
    
    let mut eval_side: f32 =1.0;
    
    if self.board.state.pov != PieceColor::White {
        eval_side*=-1.0;
    }
    

    let eval_height = bar_height/2.0 + (bar_height* eval_side*self.board.state.current_evaluation/1000.0).min(self.ui.square_size*3.0);
    if self.board.state.pov == PieceColor::White {
        eval_y = top_left.y + self.ui.square_size*8.0 - eval_height;
    }
   
    
    
    //base of the bar
    let bar_rect = Rect::from_min_size(
        pos2(top_left.x - 20.0 - self.ui.padding as f32, top_left.y),
        vec2(bar_width, bar_height),
    );
    painter.rect_filled(bar_rect, 5.0, bar_color);
    //render the part that displays evaluation
    //we assume that the white part always starts from the top
    
let eval_rect = Rect::from_min_size(
        pos2(eval_x, eval_y),
        vec2(20.0, eval_height),
    );
    painter.rect_filled(eval_rect, 5.0, eval_color);
    painter.rect_filled(
        Rect::from_min_size(
            pos2(top_left.x - 40.0 - self.ui.padding as f32, top_left.y+ bar_height/2.0),
            vec2(bar_width, bar_height/8.0),
        ), 0.0, Color32::RED);
    
}
pub fn render_move_history(&self, top_left: Pos2, ui: &mut Ui, is_visible: bool){
    if !is_visible {return;}
    let mut k=0;
    let test_moves = &self.board.meta_data.move_list;
      
    let painter = ui.painter();
    let bar_height = self.ui.square_size * 8.0;
    let bar_width = self.ui.square_size*2.0;
    let pad = self.ui.padding as f32;
    let move_text_size = (bar_width * 0.15).min(12.0);
    let (history_x, history_y) = (top_left.x + self.ui.square_size*8.0+pad, top_left.y);
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

}