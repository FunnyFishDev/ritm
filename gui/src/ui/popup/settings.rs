use egui::{Button, Checkbox, ComboBox, Grid, RichText, Stroke, Ui, style::WidgetVisuals, vec2};
use ritm_core::turing_machine::Mode;

use crate::{
    App,
    error::RitmError,
    ui::{font::Font, theme::Theme},
};

pub struct Settings {
    pub reset_after_action: bool,
    pub convert_to_graph_on_load: bool,
    pub turing_machine_mode: Mode,
    pub enable_debug: bool,
    pub theme_changer: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            reset_after_action: true,
            turing_machine_mode: Mode::StopFirstReject,
            convert_to_graph_on_load: false,
            enable_debug: false,
            theme_changer: false,
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
            .spacing(vec2(40.0, 10.0))
            .show(ui, |ui| {
                turing_mode(ui, app);
                edit_mode(ui, app);
                load_setting(ui, app);
                theme_choose(ui, app);
                debug(ui, app);
                theme_changer(ui, app);
            });
    });
    Ok(())
}

fn turing_mode(ui: &mut Ui, app: &mut App) {
    ui.label(RichText::new("Turing machine mode").font(Font::default_medium()));
    ComboBox::from_id_salt("setting_turing_mode")
        .selected_text(
            RichText::new(match app.turing.get_mode() {
                Mode::SaveAll => "Nondeterministic",
                Mode::StopAfter(_) => "Maximum 1000 steps",
                Mode::StopFirstReject => "Deterministic",
            })
            .font(Font::default_medium()),
        )
        .width(20.0) // TODO change and think about this value, I hardcoded it
        .show_ui(ui, |ui| {
            if app.settings.turing_machine_mode != Mode::SaveAll {
                ui.selectable_value(
                    &mut app.settings.turing_machine_mode,
                    Mode::SaveAll,
                    "Nondeterministic",
                );
            }

            if let Mode::StopAfter(_) = app.settings.turing_machine_mode {
                // We do that because of the content of the enum
            } else {
                ui.selectable_value(
                    &mut app.settings.turing_machine_mode,
                    Mode::StopAfter(1000),
                    "Maximum 1000 steps",
                );
            };

            if app.settings.turing_machine_mode != Mode::StopFirstReject {
                ui.selectable_value(
                    &mut app.settings.turing_machine_mode,
                    Mode::StopFirstReject,
                    "Deterministic",
                );
            }
        });
    if *app.turing.get_mode() != app.settings.turing_machine_mode {
        app.turing.set_mode(&app.settings.turing_machine_mode);
    }
    ui.end_row();
}

fn edit_mode(ui: &mut Ui, app: &mut App) {
    ui.label(RichText::new("Reset after action").font(Font::default_medium()));
    ui.add(Checkbox::without_text(&mut app.settings.reset_after_action));
    ui.end_row();
}

fn load_setting(ui: &mut Ui, app: &mut App) {
    ui.label(RichText::new("Convert to graph on load").font(Font::default_medium()));
    ui.add(Checkbox::without_text(
        &mut app.settings.convert_to_graph_on_load,
    ));
    ui.end_row();
}

fn debug(ui: &mut Ui, app: &mut App) {
    ui.label(
        RichText::new("Debug")
            .font(Font::default_medium())
            .color(app.theme.surface),
    );
    if ui
        .add(
            Button::new("")
                .min_size(vec2(25.0, 15.0))
                .fill(app.theme.surface)
                .frame(false),
        )
        .clicked()
    {
        app.settings.enable_debug ^= true;
    }
    ui.end_row();
}

fn theme_changer(ui: &mut Ui, app: &mut App) {
    ui.label(
        RichText::new("Theme changer")
            .font(Font::default_medium())
            .color(app.theme.surface),
    );
    if ui
        .add(
            Button::new("")
                .min_size(vec2(25.0, 15.0))
                .fill(app.theme.surface)
                .frame(false),
        )
        .clicked()
    {
        app.settings.theme_changer ^= true;
    }
    ui.end_row();
}

fn theme_choose(ui: &mut Ui, app: &mut App) {
    ui.label(RichText::new("Theme").font(Font::default_medium()));
    ComboBox::from_id_salt("Themes")
        .selected_text(
            RichText::new(if app.theme == Theme::default() {
                "Default"
            } else if app.theme == Theme::retro() {
                "Retro"
            } else if app.theme == Theme::monochrome() {
                "Monochrome"
            } else {
                "ERROR"
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
