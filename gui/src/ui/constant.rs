use egui::{Context, Vec2, vec2};

/// Constant used in the application
pub struct Constant {}

impl Constant {
    pub const DEFAULT_SIZE: Vec2 = vec2(800.0, 600.0);

    // Graph
    pub const CREP: f32 = 30000.0;
    pub const CSPRING: f32 = 50.0;
    pub const L: f32 = 250.0;
    pub const MAX_FORCE: f32 = 5000000.0;
    pub const STATE_RADIUS: f32 = 40.0;
    pub const TRANSITION_THICKNESS: f32 = 1.0;
    pub const STABILITY_TRESHOLD: f32 = 0.001;
    pub const ARROW_SIZE: f32 = 10.0;
    pub const TRANSITION_CURVATURE: f32 = 20.0;
    pub const SELF_TRANSITION_SIZE: f32 = 100.0;

    // Ribbon
    pub const SQUARE_SIZE: f32 = 50.0;
    pub const VERTICAL_SPACING: f32 = 5.0;
    pub const HORIZONTAL_SPACING: f32 = 5.0;
    pub const SQUARE_CORNER: f32 = 8.0;

    // Control
    pub const CONTROL_ICON_SIZE: f32 = 40.0;

    pub const ICON_SIZE: f32 = 35.0;

    // /// Scale down the value passed when the application is smaller than default
    // pub fn scale<T: From<f32> + Into<f32>>(ui: &Ui, value: T) -> T {
    //     let size = ui.ctx().screen_rect().size();
    //     if Self::DEFAULT_SIZE.x <= size.x && Self::DEFAULT_SIZE.y <= size.y {
    //         value
    //     } else {
    //         T::from(
    //             (size.x / Self::DEFAULT_SIZE.x).min(size.y / Self::DEFAULT_SIZE.y) * value.into(),
    //         )
    //     }
    // }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn update_scale(ctx: &Context) {
        let size = ctx.screen_rect().size() * ctx.pixels_per_point();
        if Self::DEFAULT_SIZE.x <= size.x && Self::DEFAULT_SIZE.y <= size.y {
            ctx.set_pixels_per_point(1.0);
        } else {
            ctx.set_pixels_per_point(
                (size.x / Self::DEFAULT_SIZE.x).min(size.y / Self::DEFAULT_SIZE.y),
            );
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn update_scale(ctx: &Context) {
        let size = ctx.screen_rect().size() * ctx.pixels_per_point();
        if Self::DEFAULT_SIZE.x <= size.x && Self::DEFAULT_SIZE.y <= size.y {
            ctx.set_pixels_per_point(1.0);
        } else {
            ctx.set_pixels_per_point(
                (size.x / Self::DEFAULT_SIZE.x).min(size.y / Self::DEFAULT_SIZE.y),
            );
        }
    }
}
