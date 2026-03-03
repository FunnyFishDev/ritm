use egui::{
    CentralPanel, Frame, Id, Margin, ScrollArea, Ui, ViewportBuilder, ViewportId,
    color_picker::{Alpha, color_picker_color32},
    hex_color,
    style::TextCursorStyle,
    vec2,
};
#[allow(dead_code)]
use egui::{
    Color32, Context, Shadow, Stroke, Visuals,
    style::{Selection, WidgetVisuals, Widgets},
};
use egui_flex::{Flex, FlexInstance, item};

use crate::App;
/// Theme of the application, holding different color for each part
#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
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
            shadow: hex_color!("#00000030"),
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
            success: hex_color!("#00a824ff"),
            backtracked: hex_color!("#ecf026ff"),
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

    pub fn monochrome() -> Self {
        Self {
            primary: hex_color!("#bfbfbfff"),
            primary_variant: hex_color!("#000000ff"),
            secondary: hex_color!("#ffffffff"),
            secondary_variant: hex_color!("#000000ff"),
            background: hex_color!("#919191ff"),
            surface: hex_color!("#ffffffff"),
            border: hex_color!("#565656ff"),
            divider: hex_color!("#a7a7a7ff"),
            text_primary: hex_color!("#000000ff"),
            text_secondary: hex_color!("#000000ff"),
            text_disabled: hex_color!("#777777ff"),
            icon: hex_color!("#000000ff"),
            hover: hex_color!("#ff0000ff"),
            active: hex_color!("#c93700ff"),
            focus: hex_color!("#ff006bff"),
            success: hex_color!("#00be00ff"),
            warning: hex_color!("#ffff00ff"),
            error: hex_color!("#ff1932ff"),
            info: hex_color!("#7bbcffff"),
            selection: hex_color!("#44e7ffff"),
            overlay: hex_color!("#424242ff"),
            shadow: hex_color!("#000000ff"),
            code_background: hex_color!("#ffffffff"),
            code: hex_color!("#000000ff"),
            syntax_keyword: hex_color!("#000000ff"),
            syntax_string: hex_color!("#000000ff"),
            syntax_comment: hex_color!("#36bc00ff"),
            highlight: hex_color!("#ff00f4ff"),
            disabled: hex_color!("#949494ff"),
            backtracked: hex_color!("#e49f45"),
        }
    }

    pub fn save_new_theme(&mut self) {
        // TODO save here the custom theme in a file or with persistence
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
    pub fn as_global_theme(&self, ctx: &Context) {
        let default_widget = self.default_widget();

        let default_shadow = Shadow {
            offset: [2, 4],
            blur: 4,
            spread: 0,
            color: Color32::from_black_alpha(25),
        };

        ctx.set_visuals(Visuals {
            text_cursor: TextCursorStyle {
                stroke: Stroke::new(1.0, self.border),
                ..Default::default()
            },
            window_fill: self.surface,
            window_corner_radius: 5.into(),
            window_stroke: Stroke::new(1.0, self.border),
            window_shadow: Shadow::NONE,
            popup_shadow: default_shadow,
            override_text_color: Some(self.text_primary),
            text_edit_bg_color: Some(self.surface),
            widgets: Widgets {
                active: WidgetVisuals { ..default_widget },
                hovered: WidgetVisuals { ..default_widget },
                inactive: WidgetVisuals { ..default_widget },
                noninteractive: WidgetVisuals { ..default_widget },
                open: WidgetVisuals { ..default_widget },
            },
            selection: Selection {
                bg_fill: Color32::from_black_alpha(50),
                stroke: Stroke::new(1.0, self.highlight),
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

pub fn theme_changer(ctx: &Context, app: &mut App) {
    ctx.show_viewport_immediate(
        ViewportId::from_hash_of(Id::new("test")),
        ViewportBuilder::default().with_always_on_top(),
        |ctx, _vc| {
            CentralPanel::default()
                .frame(
                    Frame::new()
                        .fill(Color32::LIGHT_GRAY)
                        .inner_margin(Margin::same(10)),
                )
                .show(ctx, |ui| {
                    if ui.button("save theme").clicked() {
                        print_theme(app);
                    }
                    ScrollArea::vertical().show(ui, |ui| {
                        Flex::horizontal()
                            .wrap(true)
                            .gap(vec2(5.0, 15.0))
                            .show(ui, |ui| {
                                color_pick(ui, "primary", &mut app.theme.primary);
                                color_pick(ui, "primary_variant", &mut app.theme.primary_variant);
                                color_pick(ui, "secondary", &mut app.theme.secondary);
                                color_pick(
                                    ui,
                                    "secondary_variant",
                                    &mut app.theme.secondary_variant,
                                );
                                color_pick(ui, "background", &mut app.theme.background);
                                color_pick(ui, "surface", &mut app.theme.surface);
                                color_pick(ui, "border", &mut app.theme.border);
                                color_pick(ui, "divider", &mut app.theme.divider);
                                color_pick(ui, "text_primary", &mut app.theme.text_primary);
                                color_pick(ui, "text_secondary", &mut app.theme.text_secondary);
                                color_pick(ui, "text_disabled", &mut app.theme.text_disabled);
                                color_pick(ui, "icon", &mut app.theme.icon);
                                color_pick(ui, "hover", &mut app.theme.hover);
                                color_pick(ui, "active", &mut app.theme.active);
                                color_pick(ui, "focus", &mut app.theme.focus);
                                color_pick(ui, "success", &mut app.theme.success);
                                color_pick(ui, "warning", &mut app.theme.warning);
                                color_pick(ui, "error", &mut app.theme.error);
                                color_pick(ui, "info", &mut app.theme.info);
                                color_pick(ui, "selection", &mut app.theme.selection);
                                color_pick(ui, "overlay", &mut app.theme.overlay);
                                color_pick(ui, "shadow", &mut app.theme.shadow);
                                color_pick(ui, "code_background", &mut app.theme.code_background);
                                color_pick(ui, "code", &mut app.theme.code);
                                color_pick(ui, "syntax_keyword", &mut app.theme.syntax_keyword);
                                color_pick(ui, "syntax_string", &mut app.theme.syntax_string);
                                color_pick(ui, "syntax_comment", &mut app.theme.syntax_comment);
                                color_pick(ui, "highlight", &mut app.theme.highlight);
                                color_pick(ui, "disabled", &mut app.theme.disabled);
                            });
                    });
                })
        },
    );
}

fn color_pick(ui: &mut FlexInstance, name: &str, color: &mut Color32) {
    ui.add_ui(item(), |ui| {
        ui.vertical(|ui| {
            ui.label(name);
            color_picker_color32(ui, color, Alpha::Opaque);
        });
    });
}

fn print_theme(app: &App) {
    println!("primary: hex_color!(\"{}\"),", app.theme.primary.to_hex());
    println!(
        "primary_variant: hex_color!(\"{}\"),",
        app.theme.primary_variant.to_hex()
    );
    println!(
        "secondary: hex_color!(\"{}\"),",
        app.theme.secondary.to_hex()
    );
    println!(
        "secondary_variant: hex_color!(\"{}\"),",
        app.theme.secondary_variant.to_hex()
    );
    println!(
        "background: hex_color!(\"{}\"),",
        app.theme.background.to_hex()
    );
    println!("surface: hex_color!(\"{}\"),", app.theme.surface.to_hex());
    println!("border: hex_color!(\"{}\"),", app.theme.border.to_hex());
    println!("divider: hex_color!(\"{}\"),", app.theme.divider.to_hex());
    println!(
        "text_primary: hex_color!(\"{}\"),",
        app.theme.text_primary.to_hex()
    );
    println!(
        "text_secondary: hex_color!(\"{}\"),",
        app.theme.text_secondary.to_hex()
    );
    println!(
        "text_disabled: hex_color!(\"{}\"),",
        app.theme.text_disabled.to_hex()
    );
    println!("icon: hex_color!(\"{}\"),", app.theme.icon.to_hex());
    println!("hover: hex_color!(\"{}\"),", app.theme.hover.to_hex());
    println!("active: hex_color!(\"{}\"),", app.theme.active.to_hex());
    println!("focus: hex_color!(\"{}\"),", app.theme.focus.to_hex());
    println!("success: hex_color!(\"{}\"),", app.theme.success.to_hex());
    println!("warning: hex_color!(\"{}\"),", app.theme.warning.to_hex());
    println!("error: hex_color!(\"{}\"),", app.theme.error.to_hex());
    println!("info: hex_color!(\"{}\"),", app.theme.info.to_hex());
    println!(
        "selection: hex_color!(\"{}\"),",
        app.theme.selection.to_hex()
    );
    println!("overlay: hex_color!(\"{}\"),", app.theme.overlay.to_hex());
    println!("shadow: hex_color!(\"{}\"),", app.theme.shadow.to_hex());
    println!(
        "code_background: hex_color!(\"{}\"),",
        app.theme.code_background.to_hex()
    );
    println!("code: hex_color!(\"{}\"),", app.theme.code.to_hex());
    println!(
        "syntax_keyword: hex_color!(\"{}\"),",
        app.theme.syntax_keyword.to_hex()
    );
    println!(
        "syntax_string: hex_color!(\"{}\"),",
        app.theme.syntax_string.to_hex()
    );
    println!(
        "syntax_comment: hex_color!(\"{}\"),",
        app.theme.syntax_comment.to_hex()
    );
    println!(
        "highlight: hex_color!(\"{}\"),",
        app.theme.highlight.to_hex()
    );
    println!("disabled: hex_color!(\"{}\"),", app.theme.disabled.to_hex());
}
