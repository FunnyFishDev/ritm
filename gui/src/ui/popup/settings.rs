use egui::{Checkbox, ComboBox, Grid, RichText, Stroke, Ui, style::WidgetVisuals, vec2};
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
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            reset_after_action: true,
            turing_machine_mode: Mode::SaveAll,
            convert_to_graph_on_load: false,
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
            });
    });
    Ok(())
}

fn turing_mode(ui: &mut Ui, app: &mut App) {
    ui.label(RichText::new("Turing machine mode").font(Font::default_medium()));
    if ComboBox::from_id_salt("setting_turing_mode")
        .selected_text(
            RichText::new(match app.settings.turing_machine_mode {
                Mode::SaveAll => "Nondeterministic",
                Mode::StopAfter(_) => "Limited Nondeterministic",
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
                    Mode::StopAfter(500),
                    "Limited Nondeterministic",
                );
            };

            if app.settings.turing_machine_mode != Mode::StopFirstReject {
                ui.selectable_value(
                    &mut app.settings.turing_machine_mode,
                    Mode::StopFirstReject,
                    "Deterministic",
                );
            }
        })
        .response
        .changed()
    {
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
