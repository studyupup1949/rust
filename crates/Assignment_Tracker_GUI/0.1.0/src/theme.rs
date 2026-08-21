use eframe::egui::Color32;

pub struct ThemeColors {
    pub base_text: Color32,
    pub remove_btn: Color32,
    pub remove_text: Color32,
    pub add_btn: Color32,
    pub add_text: Color32,
}

pub fn get_theme_colors(dark_mode: bool) -> ThemeColors {
    if dark_mode {
        ThemeColors {
            base_text: Color32::WHITE,
            remove_btn: Color32::from_rgb(70, 70, 70),
            remove_text: Color32::WHITE,
            add_btn: Color32::from_rgb(70, 130, 255),
            add_text: Color32::WHITE,
        }
    } else {
        ThemeColors {
            base_text: Color32::BLACK,
            remove_btn: Color32::from_rgb(230, 230, 230),
            remove_text: Color32::BLACK,
            add_btn: Color32::from_rgb(220, 220, 220),
            add_text: Color32::BLACK,
        }
    }
}
