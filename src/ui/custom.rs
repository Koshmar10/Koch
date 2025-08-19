use eframe::egui::{vec2, Color32, Response, Sense, TextureHandle, Ui, Vec2, Rect, Stroke, pos2};
use eframe::epaint::StrokeKind;
use std::error::Error;

use crate::ui::app::MyApp;

impl MyApp{
    pub fn colourd_image_button(&self, size: Vec2, texture: &Result<TextureHandle, Box<dyn Error>>, color: Color32, ui: &mut Ui) -> Response {
        let tex = match texture {
            Ok(t) => t,
            Err(_) => &self.theme.empty_texture.clone(),
        };
    
                // Define button appearance
                let button_size = vec2(size.x, size.y - 2.0);
                let normal_color = color;
                
                // Make hover color slightly lighter
                let hover_color = Color32::from_rgb(
                    (normal_color.r() as f32 * 1.2).min(255.0) as u8,
                    (normal_color.g() as f32 * 1.2).min(255.0) as u8,
                    (normal_color.b() as f32 * 1.2).min(255.0) as u8,
                );
                
                // Make pressed color darker
                let pressed_color = Color32::from_rgb(
                    (normal_color.r() as f32 * 0.7) as u8,
                    (normal_color.g() as f32 * 0.7) as u8,
                    (normal_color.b() as f32 * 0.7) as u8,
                );

                // Reserve space for button and create interaction
                let (rect, response) = ui.allocate_exact_size(button_size, Sense::click());

                // Determine button color based on interaction state
                let fill_color = if response.is_pointer_button_down_on() {
                    pressed_color
                } else if response.hovered() {
                    hover_color
                } else {
                    normal_color
                };

                // Draw button background with rounded corners
                ui.painter().rect_filled(rect, 4.0, fill_color);

                // Draw button border
                ui.painter().rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(100, 100, 50)), StrokeKind::Middle);

                // Calculate container for image with padding
                let image_container = rect.shrink(4.0);
                
                // Get texture dimensions and calculate aspect ratio
                let tex_size = tex.size_vec2();
                let tex_aspect_ratio = tex_size.x / tex_size.y;
                
                // Calculate image dimensions to maintain aspect ratio
                let mut image_width = image_container.width();
                let mut image_height = image_width / tex_aspect_ratio;
                
                // If height exceeds container, scale down
                if image_height > image_container.height() {
                    image_height = image_container.height();
                    image_width = image_height * tex_aspect_ratio;
                }
                
                // Center the image in the container
                let image_pos = pos2(
                    image_container.min.x + (image_container.width() - image_width) / 2.0,
                    image_container.min.y + (image_container.height() - image_height) / 2.0
                );
                
                // Create image rect with proper aspect ratio
                let image_rect = Rect::from_min_size(image_pos, vec2(image_width, image_height));
                
                // Draw image centered in button
                ui.painter().image(
                    tex.id(),
                    image_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE
                );
                
                // Handle click
                return response;
}
}