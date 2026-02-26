use std::path::Path;

use egui::{
    AtomExt, CentralPanel, Checkbox, ComboBox, Context, DragValue, Grid, Id, Image, ImageSource,
    RichText, Stroke, TextBuffer, Ui, UserData, ViewportBuilder, ViewportCommand, ViewportId,
    include_image, style::WidgetVisuals, vec2,
};
use image::{ExtendedColorType, save_buffer};
use include_directory::{Dir, include_directory};
use ritm_core::turing_machine::Mode;

use crate::{
    App,
    error::RitmError,
    ui::{font::Font, theme::Theme},
};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Settings {
    pub reset_after_action: bool,
    pub convert_to_graph_on_load: bool,
    #[serde(with = "ModeRitm")]
    pub turing_machine_mode: Mode,
    pub enable_debug: bool,
    pub theme_changer: bool,
    pub edit_button_size: f32,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(remote = "Mode")]
enum ModeRitm {
    SaveAll,
    StopAfter(usize),
    StopFirstReject,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            reset_after_action: true,
            turing_machine_mode: Mode::StopFirstReject,
            convert_to_graph_on_load: false,
            enable_debug: false,
            theme_changer: false,
            edit_button_size: 25.0,
        }
    }
}

pub fn show(ui: &mut Ui, app: &mut App) -> Result<(), RitmError> {
    ui.scope(|ui| {
        Theme::set_widget(
            ui,
            WidgetVisuals {
                bg_stroke: Stroke::new(1.0, app.theme.border),
                corner_radius: 5.into(),
                expansion: 3.0,
                ..app.theme.default_widget()
            },
        );

        Grid::new("settings")
            .spacing(vec2(40.0, 20.0))
            .show(ui, |ui| {
                turing_mode(ui, app);
                edit_mode(ui, app);
                load_setting(ui, app);
                theme_choose(ui, app);
                localisation_setting(ui, app);
                edit_button_size(ui, app);
                tape_count(ui, app);
                // debug(ui, app);
                // theme_changer(ui, app);
            });
    });
    Ok(())
}

fn turing_mode(ui: &mut Ui, app: &mut App) {
    ui.label(RichText::new(t!("turing_machine_mode")).font(Font::default_medium()));
    ComboBox::from_id_salt("setting_turing_mode")
        .selected_text(
            RichText::new(match app.turing.get_mode() {
                Mode::SaveAll => t!("nondeterministic"),
                Mode::StopAfter(_) => t!("limited", "step" = 1000),
                Mode::StopFirstReject => t!("deterministic"),
            })
            .font(Font::default_medium()),
        )
        .width(20.0) // TODO change and think about this value, I hardcoded it
        .show_ui(ui, |ui| {
            if app.settings.turing_machine_mode != Mode::SaveAll {
                ui.selectable_value(
                    &mut app.settings.turing_machine_mode,
                    Mode::SaveAll,
                    t!("nondeterministic"),
                );
            }

            if let Mode::StopAfter(_) = app.settings.turing_machine_mode {
                // We do that because of the content of the enum
            } else {
                ui.selectable_value(
                    &mut app.settings.turing_machine_mode,
                    Mode::StopAfter(1000),
                    t!("limited", "step" = 1000),
                );
            };

            if app.settings.turing_machine_mode != Mode::StopFirstReject {
                ui.selectable_value(
                    &mut app.settings.turing_machine_mode,
                    Mode::StopFirstReject,
                    t!("deterministic"),
                );
            }
        });
    if *app.turing.get_mode() != app.settings.turing_machine_mode {
        app.turing.set_mode(&app.settings.turing_machine_mode);
    }
    ui.end_row();
}

fn edit_mode(ui: &mut Ui, app: &mut App) {
    ui.label(RichText::new(t!("reset_after_action")).font(Font::default_medium()));
    ui.add(Checkbox::without_text(&mut app.settings.reset_after_action));
    ui.end_row();
}

fn load_setting(ui: &mut Ui, app: &mut App) {
    ui.label(RichText::new(t!("convert_on_load")).font(Font::default_medium()));
    ui.add(Checkbox::without_text(
        &mut app.settings.convert_to_graph_on_load,
    ));
    ui.end_row();
}

static EXAMPLES: Dir = include_directory!("gui/locales");

fn get_file(name: &str) -> ImageSource<'_> {
    let bytes = EXAMPLES.get_file(name).unwrap().contents();
    ImageSource::Bytes {
        uri: format!("byte://../locales/{name}").into(),
        bytes: egui::load::Bytes::Static(bytes),
    }
}

fn localisation_setting(ui: &mut Ui, app: &mut App) {
    ui.label(RichText::new(t!("language")).font(Font::default_medium()));

    let locale = rust_i18n::locale().to_string();
    ui.menu_button(
        (
            Image::new(get_file(&format!("{locale}.svg"))).shrink_to_fit(),
            locale.clone(),
            Image::new(include_image!("../../../assets/icon/down.svg"))
                .tint(app.theme.overlay)
                .shrink_to_fit(),
        ),
        |ui| {
            for locale in rust_i18n::available_locales!() {
                if ui
                    .button((
                        Image::new(get_file(&format!("{locale}.svg")))
                            .atom_max_height_font_size(ui),
                        locale.clone(),
                    ))
                    .clicked()
                {
                    rust_i18n::set_locale(&locale);
                }
            }
        },
    );
    // <ComboBox::from_id_salt("language")
    //     .selected_text(
    //         RichText::new(match app.turing.get_mode() {
    //             Mode::SaveAll => t!("nondeterministic"),
    //             Mode::StopAfter(_) => t!("limited", "step" = 1000),
    //             Mode::StopFirstReject => t!("deterministic"),
    //         })
    //         .font(Font::default_medium()),
    //     )
    //     .width(20.0) // TODO change and think about this value, I hardcoded it
    //     .show_ui(ui, |ui| {
    //         if app.settings.turing_machine_mode != Mode::SaveAll {
    //             ui.selectable_value(
    //                 &mut app.settings.turing_machine_mode,
    //                 Mode::SaveAll,
    //                 t!("nondeterministic"),
    //             );
    //         }

    //         if let Mode::StopAfter(_) = app.settings.turing_machine_mode {
    //             // We do that because of the content of the enum
    //         } else {
    //             ui.selectable_value(
    //                 &mut app.settings.turing_machine_mode,
    //                 Mode::StopAfter(1000),
    //                 t!("limited", "step" = 1000),
    //             );
    //         };

    //         if app.settings.turing_machine_mode != Mode::StopFirstReject {
    //             ui.selectable_value(
    //                 &mut app.settings.turing_machine_mode,
    //                 Mode::StopFirstReject,
    //                 t!("deterministic"),
    //             );
    //         }
    //     });
    // if *app.turing.get_mode() != app.settings.turing_machine_mode {
    //     app.turing.set_mode(&app.settings.turing_machine_mode);
    // }>
    ui.end_row();
}

// fn debug(ui: &mut Ui, app: &mut App) {
//     ui.label(
//         RichText::new("Debug")
//             .font(Font::default_medium())
//             .color(app.theme.surface),
//     );
//     if ui
//         .add(
//             Button::new("")
//                 .min_size(vec2(25.0, 15.0))
//                 .fill(app.theme.surface)
//                 .frame(false),
//         )
//         .clicked()
//     {
//         app.settings.enable_debug ^= true;
//     }
//     ui.end_row();
// }

// fn theme_changer(ui: &mut Ui, app: &mut App) {
//     ui.label(
//         RichText::new("Theme changer")
//             .font(Font::default_medium())
//             .color(app.theme.surface),
//     );
//     if ui
//         .add(
//             Button::new("")
//                 .min_size(vec2(25.0, 15.0))
//                 .fill(app.theme.surface)
//                 .frame(false),
//         )
//         .clicked()
//     {
//         app.settings.theme_changer ^= true;
//     }
//     ui.end_row();
// }

fn theme_choose(ui: &mut Ui, app: &mut App) {
    ui.label(RichText::new(t!("theme")).font(Font::default_medium()));
    ComboBox::from_id_salt("Themes")
        .selected_text(
            RichText::new(if app.theme == Theme::default() {
                t!("default")
            } else if app.theme == Theme::retro() {
                t!("retro")
            } else if app.theme == Theme::monochrome() {
                t!("monochrome")
            } else {
                t!("default")
            })
            .font(Font::default_medium()),
        )
        .width(20.0) // TODO change and think about this value, I hardcoded it
        .show_ui(ui, |ui| {
            if app.theme != Theme::default() {
                ui.selectable_value(&mut app.theme, Theme::default(), "Default");
            }

            if app.theme != Theme::retro() {
                ui.selectable_value(&mut app.theme, Theme::retro(), "Retro");
            };

            if app.theme != Theme::monochrome() {
                ui.selectable_value(&mut app.theme, Theme::monochrome(), "Monochrome");
            }
        });
    ui.end_row();
}

fn edit_button_size(ui: &mut Ui, app: &mut App) {
    ui.label(RichText::new(t!("edit_icon_size")).font(Font::default_medium()));
    ComboBox::from_id_salt("edit_icon_size")
        .selected_text(
            RichText::new(match app.settings.edit_button_size {
                25.0 => t!("small").to_string(),
                35.0 => t!("medium").to_string(),
                45.0 => t!("big").to_string(),
                _ => "ERROR".to_string(),
            })
            .font(Font::default_medium()),
        )
        .width(20.0) // TODO change and think about this value, I hardcoded it
        .show_ui(ui, |ui| {
            if app.settings.edit_button_size != 25.0 {
                ui.selectable_value(&mut app.settings.edit_button_size, 25.0, t!("small"));
            }

            if app.settings.edit_button_size != 35.0 {
                ui.selectable_value(&mut app.settings.edit_button_size, 35.0, t!("medium"));
            }

            if app.settings.edit_button_size != 45.0 {
                ui.selectable_value(&mut app.settings.edit_button_size, 45.0, t!("big"));
            }
        });
    ui.end_row();
}

fn tape_count(ui: &mut Ui, app: &mut App) {
    ui.label(RichText::new(t!("tape_count")).font(Font::default_medium()));
    let mut k = app.turing.tm.graph_ref().get_k();
    if ui
        .add(
            DragValue::new(&mut k)
                .range(0..=4)
                .clamp_existing_to_range(true)
                .update_while_editing(false),
        )
        .changed()
    {
        // TODO: add error here
        app.update_k(k);
    }
    ui.end_row();
}

pub fn debug_show(ctx: &Context, app: &mut App) {
    let mut x = false;
    ctx.show_viewport_immediate(
        ViewportId::from_hash_of(Id::new("test")),
        ViewportBuilder::default()
            .with_always_on_top()
            .with_inner_size(vec2(150.0, 30.0)),
        |ctx, _vc| {
            CentralPanel::default().show(ctx, |ui| {
                if ui.button("Take screenshot").clicked() {
                    x = true;
                }
            })
        },
    );

    if x {
        ctx.send_viewport_cmd(ViewportCommand::Screenshot(UserData::default()));
    }
    take_screenshot(app, ctx);
}

fn take_screenshot(_app: &mut App, ctx: &Context) {
    let rect = ctx.screen_rect();

    let time = ctx.input(|r| r.time);
    ctx.input(|i| {
        i.events.iter().for_each(|e| {
            if let egui::Event::Screenshot { image, .. } = e {
                let image = image.region(&rect, Some(i.pixels_per_point));
                save_buffer(
                    Path::new(&format!(
                        "assets/help/screenshot-{}.png",
                        time.to_string().char_range(0..4)
                    )),
                    image.as_raw(),
                    image.source_size.x as u32,
                    image.source_size.y as u32,
                    ExtendedColorType::Rgba8,
                )
                .unwrap();
            }
        })
    });
}
