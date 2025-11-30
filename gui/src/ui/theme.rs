#[allow(dead_code)]
use egui::{
    Color32, Context, Shadow, Stroke, Visuals,
    style::{Selection, WidgetVisuals, Widgets},
};
use egui::{Ui, style::TextCursorStyle};
/// Theme of the application, holding different color for each part
pub struct Theme {
    pub background: Color32,
    pub graph: Color32,
    pub ribbon: Color32,
    pub code: Color32,
    pub highlight: Color32,
    pub selected: Color32,
    pub gray: Color32,
    pub white: Color32,
    pub valid: Color32,
    pub invalid: Color32,
}

impl Theme {
    /// Default theme of the application
    pub const DEFAULT: Theme = Theme {
        background: Color32::from_rgb(255, 163, 132),
        graph: Color32::from_rgb(254, 239, 218),
        ribbon: Color32::from_rgb(116, 189, 203),
        code: Color32::from_rgb(231, 242, 248),
        highlight: Color32::from_rgb(255, 105, 105),
        selected: Color32::from_rgb(149, 189, 252),
        gray: Color32::from_gray(102),
        white: Color32::WHITE,
        valid: Color32::GREEN,
        invalid: Color32::RED,
    };

    pub fn default_widget(&self) -> WidgetVisuals {
        WidgetVisuals {
            bg_fill: self.white,
            bg_stroke: Stroke::NONE,
            corner_radius: 2.into(),
            expansion: 0.0,
            fg_stroke: Stroke::new(1.0, self.gray),
            weak_bg_fill: self.white,
        }
    }

    /// Set the global theme used in egui widget
    pub fn set_global_theme(theme: &Self, ctx: &Context) {
        let default_widget = theme.default_widget();

        let default_shadow = Shadow {
            offset: [2, 4],
            blur: 4,
            spread: 0,
            color: Color32::from_black_alpha(25),
        };

        ctx.set_visuals(Visuals {
            text_cursor: TextCursorStyle {
                stroke: Stroke::new(1.0, theme.gray),
                ..Default::default()
            },
            window_fill: theme.white,
            window_corner_radius: 5.into(),
            window_stroke: Stroke::new(1.0, theme.gray),
            window_shadow: Shadow::NONE,
            popup_shadow: default_shadow,
            override_text_color: Some(theme.gray),
            text_edit_bg_color: Some(theme.white),
            widgets: Widgets {
                active: WidgetVisuals { ..default_widget },
                hovered: WidgetVisuals { ..default_widget },
                inactive: WidgetVisuals { ..default_widget },
                noninteractive: WidgetVisuals { ..default_widget },
                open: WidgetVisuals { ..default_widget }
            },
            selection: Selection {
                bg_fill: Color32::from_black_alpha(50),
                stroke: Stroke::new(1.0, theme.highlight)
            },
            ..Default::default()
        });
    }

    pub fn set_widget(ui: &mut Ui, widget: WidgetVisuals) {
        ui.visuals_mut().widgets.inactive = widget;
        ui.visuals_mut().widgets.active = widget;
        ui.visuals_mut().widgets.hovered = widget;
        ui.visuals_mut().widgets.noninteractive = widget;
        ui.visuals_mut().widgets.open = widget;
    }

    /// Compute the best contrast color between white and black for any RGB color
    pub fn constrast_color(color: Color32) -> Color32 {
        let luminance =
            (0.299 * color.r() as f32 + 0.587 * color.g() as f32 + 0.114 * color.b() as f32)
                / 255.0;

        if luminance > 0.5 {
            Color32::BLACK
        } else {
            Color32::WHITE
        }
    }
}
