use std::collections::HashMap;

use std::error::Error;
use std::path::Path;

use eframe::egui::load;
use eframe::egui::Color32;
use eframe::egui::ColorImage;
use eframe::egui::TextureHandle;
use eframe::egui::TextureOptions;



use eframe::CreationContext;

use crate::engine::PieceColor;
use crate::engine::PieceType;





pub struct ThemeLoader{
    pub dark_square: Color32,
    pub light_square: Color32,
    pub dark_square_hover: Color32,
    pub light_square_hover: Color32,
    pub moved_from_highlight: Color32,
    pub moved_to_highlight: Color32,
    pub empty_texture: TextureHandle,
    pub piece_map: HashMap<(PieceType, PieceColor), Result<TextureHandle, Box<dyn Error>>>, 
    pub square_select_highlight: Color32,
    pub dark_pseudo_move_highlight: Color32,  
    pub light_pseudo_move_highlight: Color32,
    pub checkmate_square: Color32,

    pub white_pfp: Result<TextureHandle, Box<dyn Error>>,
    pub black_pfp: Result<TextureHandle, Box<dyn Error>>
            
}

impl From<&CreationContext<'_>> for ThemeLoader {
    fn from(cc : &CreationContext<'_>) -> Self {
        ThemeLoader { 
            dark_square: Color32::from_rgb(181, 136, 99).to_opaque(), 
            light_square: Color32::from_rgb(230, 207, 171).to_opaque(), 
            dark_square_hover: Color32::from_rgb(191, 146, 119), 
            light_square_hover: Color32::from_rgb(240, 217, 181),
            square_select_highlight: Color32::from_rgba_unmultiplied(255, 255, 0, 128),
            light_pseudo_move_highlight: Color32::from_rgba_unmultiplied(70, 70, 70, 128),
            dark_pseudo_move_highlight: Color32::from_rgba_unmultiplied( 80,  80,  80, 128),
            moved_from_highlight:Color32::from_rgb(255, 170, 0),
            moved_to_highlight: Color32::from_rgb(0, 204, 255),
            checkmate_square: Color32::from_rgb(255, 0, 0),
            empty_texture: cc.egui_ctx.load_texture(
                "empty_piece",
                ColorImage::new([1, 1], Color32::TRANSPARENT),
                TextureOptions::default(),
            ),
            white_pfp: load_texture(cc, "assets/images/killua.png"),
            black_pfp: load_texture(cc, "assets/images/gon.png"),
            piece_map: {
                let mut map = HashMap::with_capacity(12);
                for &color in &[PieceColor::White, PieceColor::Black] {
                    for &piece in &[
                        PieceType::Pawn,
                        PieceType::Bishop,
                        PieceType::King,
                        PieceType::Knight,
                        PieceType::Queen,
                        PieceType::Rook,
                    ] {
                        let image_path = format!(
                            "assets/pieces/{}_{}.png",
                            color.to_string().to_lowercase(),
                            piece.to_string().to_lowercase()
                        );
                        println!("{}", image_path.clone());
                        map.insert((piece, color), load_texture(cc, &image_path));
                    }
                }
                  map
            },

        }
    }
}
pub fn load_texture(cc: &CreationContext, image_path: &str) -> Result<TextureHandle, Box<dyn Error>> {
    // take the directory where Cargo.toml lives:
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    // join our relative asset path to it:
    let full_path = Path::new(manifest_dir).join(image_path);

    let image = image::open(&full_path)
        .map_err(|e| format!("Failed to open {:?}: {}", full_path, e))?
        .to_rgba8();
    let (w, h) = image.dimensions();
    let pixels = image
        .chunks_exact(4)
        .map(|rgba| Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3]))
        .collect();
    let color_image = ColorImage { size: [w as usize, h as usize], pixels };

    let texture_name = full_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(image_path);
    let texture = cc.egui_ctx.load_texture(texture_name, color_image, TextureOptions::default());
    Ok(texture)
}

