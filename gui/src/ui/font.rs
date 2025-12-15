use egui::{Color32, FontFamily, FontId, Ui, Vec2, text::LayoutJob, vec2};

/// Access to the different font used in the application
pub struct Font;

/// Font are accessed by function
///
/// To add one, first load it in the app.rs
impl Font {
    pub const BIG_SIZE: f32 = 20.0;
    pub const MEDIUM_SIZE: f32 = 16.0;
    pub const SMALL_SIZE: f32 = 12.0;

    /// default font used in the application
    pub fn default(size: f32) -> FontId {
        FontId {
            family: FontFamily::Name("RobotoMono-regular".into()),
            size,
        }
    }

    /// default font used in the application
    pub fn default_medium() -> FontId {
        FontId {
            family: FontFamily::Name("RobotoMono-regular".into()),
            size: Self::MEDIUM_SIZE,
        }
    }

    /// default font used in the application
    pub fn default_big() -> FontId {
        FontId {
            family: FontFamily::Name("RobotoMono-regular".into()),
            size: Self::BIG_SIZE,
        }
    }

    /// default font used in the application
    pub fn default_small() -> FontId {
        FontId {
            family: FontFamily::Name("RobotoMono-regular".into()),
            size: Self::SMALL_SIZE,
        }
    }

    /// bold version of the default font used in the application
    pub fn bold() -> FontId {
        FontId {
            family: FontFamily::Name("RobotoMono-Bold".into()),
            size: Self::MEDIUM_SIZE,
        }
    }

    /// Compute the size of a text with a certain font
    pub fn _text_size(ui: &Ui, fond_id: FontId, text: &str) -> Vec2 {
        let rect = ui.fonts(|f| {
            f.layout_job(LayoutJob::simple_singleline(
                text.to_string(),
                fond_id,
                Color32::PLACEHOLDER,
            ))
            .rect
        });

        vec2(rect.width(), rect.height())
    }

    pub fn get_heigth(ui: &Ui, fond_id: &FontId) -> f32 {
        ui.fonts(|f| f.row_height(fond_id))
    }

    pub fn get_width(ui: &Ui, fond_id: &FontId) -> f32 {
        ui.fonts(|f| f.glyph_width(fond_id, 'M'))
    }
}
