#[allow(dead_code)]
use egui::{
    Color32, Context, Shadow, Stroke, Visuals,
    style::{Selection, WidgetVisuals, Widgets},
};
use egui::{Ui, hex_color, style::TextCursorStyle};
/// Theme of the application, holding different color for each part
pub struct Theme {
    pub primary: Color32,
    pub primary_variant: Color32,
    pub secondary: Color32,
    pub secondary_variant: Color32,
    pub background: Color32,
    pub surface: Color32,
    pub border: Color32,
    pub divider: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_disabled: Color32,
    pub icon: Color32,
    pub hover: Color32,
    pub active: Color32,
    pub focus: Color32,
    pub success: Color32,
    pub backtracked: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub info: Color32,
    pub selection: Color32,
    pub overlay: Color32,
    pub shadow: Color32,
    pub code_background: Color32,
    pub code: Color32,
    pub syntax_keyword: Color32,
    pub syntax_string: Color32,
    pub syntax_comment: Color32,
    pub highlight: Color32,
    pub disabled: Color32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: hex_color!("#8d6346ff"),
            primary_variant: hex_color!("#000000ff"),
            secondary: hex_color!("#fff0e0ff"),
            secondary_variant: hex_color!("#000000ff"),
            background: hex_color!("#68c251ff"),
            surface: hex_color!("#ffffffff"),
            border: hex_color!("#454545ff"),
            divider: hex_color!("#bbbbbbff"),
            text_primary: hex_color!("#111111ff"),
            text_secondary: hex_color!("#a4a4a4ff"),
            text_disabled: hex_color!("#777777ff"),
            icon: hex_color!("#323232ff"),
            hover: hex_color!("#ff0000ff"),
            active: hex_color!("#acff00ff"),
            focus: hex_color!("#ff0000ff"),
            success: hex_color!("#19c832ff"),
            backtracked: hex_color!("#e49f45"),
            warning: hex_color!("#ffff00ff"),
            error: hex_color!("#ff1964ff"),
            info: hex_color!("#ff0000ff"),
            selection: hex_color!("#5adbffff"),
            overlay: hex_color!("#000000ff"),
            shadow: hex_color!("#000000ff"),
            code_background: hex_color!("#313e45ff"),
            code: hex_color!("#fcf3edff"),
            syntax_keyword: hex_color!("#000000ff"),
            syntax_string: hex_color!("#000000ff"),
            syntax_comment: hex_color!("#55a04bff"),
            highlight: hex_color!("#ff5e5eff"),
            disabled: hex_color!("#b3b3b3ff"),
        }
    }
}

impl Theme {
    /// Default theme of the application
    // pub const DEFAULT: Theme = Theme {
    //     background: Color32::from_rgb(255, 163, 132),
    //     graph: Color32::from_rgb(254, 239, 218),
    //     ribbon: Color32::from_rgb(116, 189, 203),
    //     code: Color32::from_rgb(231, 242, 248),
    //     highlight: Color32::from_rgb(255, 105, 105),
    //     selected: Color32::from_rgb(149, 189, 252),
    //     gray: Color32::from_gray(102),
    //     white: Color32::WHITE,
    //     valid: Color32::GREEN,
    //     invalid: Color32::RED,
    // };
    pub fn retro() -> Self {
        Self {
            primary: hex_color!("#74bdcbff"),
            primary_variant: hex_color!("#000000ff"),
            secondary: hex_color!("#fff0e0ff"),
            secondary_variant: hex_color!("#000000ff"),
            background: hex_color!("#ffa384ff"),
            surface: hex_color!("#fffafaff"),
            border: hex_color!("#7d7d7dff"),
            divider: hex_color!("#bbbbbbff"),
            text_primary: hex_color!("#323232ff"),
            text_secondary: hex_color!("#a4a4a4ff"),
            text_disabled: hex_color!("#777777ff"),
            icon: hex_color!("#ffffffff"),
            hover: hex_color!("#ff0000ff"),
            active: hex_color!("#C73E1Dff"),
            focus: hex_color!("#ff5080ff"),
            success: hex_color!("#19c832ff"),
            backtracked: hex_color!("#e49f45"),
            warning: hex_color!("#ffff00ff"),
            error: hex_color!("#ff1932ff"),
            info: hex_color!("#7bbcffff"),
            selection: hex_color!("#68C3D4ff"),
            overlay: hex_color!("#1E3232ff"),
            shadow: hex_color!("#000000ff"),
            code_background: hex_color!("#313e45ff"),
            code: hex_color!("#fcf3edff"),
            syntax_keyword: hex_color!("#000000ff"),
            syntax_string: hex_color!("#000000ff"),
            syntax_comment: hex_color!("#55a04bff"),
            highlight: hex_color!("#ff5e5eff"),
            disabled: hex_color!("#d2d2d2ff"),
        }
    }

    pub fn default_widget(&self) -> WidgetVisuals {
        WidgetVisuals {
            bg_fill: self.surface,
            bg_stroke: Stroke::new(1.0, self.border),
            corner_radius: 2.into(),
            expansion: 0.0,
            fg_stroke: Stroke::new(1.0, self.border),
            weak_bg_fill: self.surface,
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
                stroke: Stroke::new(1.0, theme.border),
                ..Default::default()
            },
            window_fill: theme.surface,
            window_corner_radius: 5.into(),
            window_stroke: Stroke::new(1.0, theme.border),
            window_shadow: Shadow::NONE,
            popup_shadow: default_shadow,
            override_text_color: Some(theme.text_primary),
            text_edit_bg_color: Some(theme.surface),
            widgets: Widgets {
                active: WidgetVisuals { ..default_widget },
                hovered: WidgetVisuals { ..default_widget },
                inactive: WidgetVisuals { ..default_widget },
                noninteractive: WidgetVisuals { ..default_widget },
                open: WidgetVisuals { ..default_widget },
            },
            selection: Selection {
                bg_fill: Color32::from_black_alpha(50),
                stroke: Stroke::new(1.0, theme.highlight),
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
