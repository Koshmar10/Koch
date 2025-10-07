use eframe::{egui::{self, pos2, vec2, Color32, Pos2, Rect, Sense, Stroke, Ui, UiBuilder}, epaint::PathShape};

use crate::ui::app::MyApp;

impl MyApp {
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
            //determine how much space i have to draw the graph,
            let plot_area_start = chart_area.left()+25.0;
            let graph_draw_area = ((chart_area.left()+ 20.0)-(chart_area.right()-pad)).abs();
            let space_between_plots = graph_draw_area / self.board.meta_data.move_list.len() as f32;

            // First calculate all points positions
            let mut points = Vec::new();

            // Calculate point positions for all moves
            for (i, mv) in self.board.meta_data.move_list.iter().enumerate() {
                let score = mv.evaluation.value/100.0;
                let usable_height = chart_height/2.0 - pad * 2.0;
                let normalized_score = (score / 3.0).tanh();
                let final_offset = normalized_score * usable_height;
                let min_offset = 10.0;
                let final_offset = if final_offset.abs() < min_offset && score != 0.0 {
                    min_offset * final_offset.signum()
                } else {
                    final_offset
                };
                let mapped_y = chart_area.center().y - final_offset;
                let point_x = plot_area_start + i as f32 * space_between_plots;

                points.push((point_x, mapped_y, score));
            }

            // Draw smooth curve through points
            if points.len() >= 2 {
                let raw_pts: Vec<Pos2> = points.iter().map(|(x, y, _)| pos2(*x, *y)).collect();
                let smoothed = Self::catmull_rom_polyline(&raw_pts, 8);
                ui.painter().add(PathShape::line(smoothed, Stroke::new(1.5, Color32::BLACK)));
            }

            // Draw points
            for (i, (x, y, score)) in points.iter().enumerate() {
                let color = if i == self.analyzer.current_ply.saturating_sub(1) as usize {
                    Color32::RED
                } else {
                    Color32::ORANGE
                };
                self.plot_point(*x, *y, *score, color, ui);
            }

            // Draw axes
            ui.painter().line_segment([ox_start, ox_stop], Stroke::new(2.0, chart_color));

            ui.painter().line_segment([oy_start, oy_stop], Stroke::new(2.0, chart_color));
        });
    }

    pub fn plot_point(&self, x: f32, y: f32, value: f32, color: Color32, ui: &mut Ui) {
        let circle_radius = 1.4;
        let circle_position = pos2(
            x,  // X position - a bit to the right of the y-axis
            y   // Centered on the x-axis
        );

        // Draw a filled circle
        ui.painter().circle_filled(
            circle_position,
            circle_radius,
            color
        );
        
        // Create an interactive area around the point
        let point_rect = Rect::from_center_size(
            circle_position,
            vec2(circle_radius * 4.0, circle_radius * 4.0), // Larger hit area for easier hovering
        );
        
        // Make the area interactive
        let id = ui.make_persistent_id(format!("plot_point_{:.1}_{:.1}", x, y));
        let response = ui.interact(point_rect, id, Sense::hover());
        
        // Show tooltip when hovering
        if response.hovered() {
            // Show tooltip with value above the point
            let tooltip_height = 20.0;
            let tooltip_width = 30.0;
            let tooltip_rect = Rect::from_min_size(
                pos2(x - tooltip_width / 2.0 + 5.0, y - tooltip_height - 10.0), // Position above the point
                vec2(tooltip_width, tooltip_height),
            );
            ui.painter().rect_filled(
                tooltip_rect,
                3.0, // Corner radius
                Color32::from_rgba_premultiplied(50, 50, 50, 230), // Semi-transparent background
            );
            // Add the text value
            ui.painter().text(
                tooltip_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{:.2}", value),
                egui::TextStyle::Small.resolve(ui.style()),
                Color32::WHITE,
            );
        }
    }

    // Smooth Catmull–Rom polyline (uniform), sampling each segment
    pub fn catmull_rom_polyline(pts: &[Pos2], samples_per_segment: usize) -> Vec<Pos2> {
        if pts.len() < 2 {
            return pts.to_vec();
        }
        let mut out = Vec::with_capacity((pts.len() - 1) * samples_per_segment + 1);
        for i in 0..(pts.len() - 1) {
            let p0 = if i == 0 { pts[i] } else { pts[i - 1] };
            let p1 = pts[i];
            let p2 = pts[i + 1];
            let p3 = if i + 2 < pts.len() { pts[i + 2] } else { pts[i + 1] };

            for j in 0..samples_per_segment {
                let t = j as f32 / samples_per_segment as f32;
                let t2 = t * t;
                let t3 = t2 * t;

                let x = 0.5 * ((2.0 * p1.x)
                    + (-p0.x + p2.x) * t
                    + (2.0 * p0.x - 5.0 * p1.x + 4.0 * p2.x - p3.x) * t2
                    + (-p0.x + 3.0 * p1.x - 3.0 * p2.x + p3.x) * t3);

                let y = 0.5 * ((2.0 * p1.y)
                    + (-p0.y + p2.y) * t
                    + (2.0 * p0.y - 5.0 * p1.y + 4.0 * p2.y - p3.y) * t2
                    + (-p0.y + 3.0 * p1.y - 3.0 * p2.y + p3.y) * t3);

                out.push(pos2(x, y));
            }
        }
        out.push(*pts.last().unwrap());
        out
    }
}