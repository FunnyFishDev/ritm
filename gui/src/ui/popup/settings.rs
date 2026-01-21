use egui::{Checkbox, Grid, Label, RichText, Stroke, Ui, style::WidgetVisuals, vec2};
use ritm_core::turing_machine::Mode;

use crate::{
    App,
    error::RitmError,
    ui::{component::combobox::ComboBox, font::Font, theme::Theme},
};

pub struct Settings {
    pub toggle_after_action: bool,
    pub turing_machine_mode: Mode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            toggle_after_action: Default::default(),
            turing_machine_mode: Mode::SaveAll,
        }
    }
}

pub fn show(ui: &mut Ui, app: &mut App) -> Result<(), RitmError> {
    ui.set_max_size(ui.ctx().screen_rect().size() * 0.8);
    ui.set_min_size(ui.ctx().screen_rect().size() * 0.8);

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
                ui.add(Label::new(
                    RichText::new("Turing machine mode").font(Font::default_medium()),
                ));

                ComboBox::from_id_salt("setting_turing_mode")
                    .selected_text(
                        RichText::new(match app.settings.turing_machine_mode {
                            Mode::SaveAll => "Unsafe",
                            Mode::StopAfter(_) => "Safe",
                            Mode::StopFirstReject => "Error",
                        })
                        .font(Font::default_medium()),
                    )
                    .width(20.0) // TODO change and think about this value, I hardcoded it
                    .show_ui(ui, |ui| {
                        if app.settings.turing_machine_mode != Mode::SaveAll {
                            ui.selectable_value(
                                &mut app.settings.turing_machine_mode,
                                Mode::SaveAll,
                                "Unsafe",
                            );
                        }
                        if let Mode::StopAfter(_) = app.settings.turing_machine_mode {
                        } else {
                            ui.selectable_value(
                                &mut app.settings.turing_machine_mode,
                                Mode::StopAfter(500),
                                "Safe",
                            );
                        }
                    });
                ui.end_row();

                ui.add(Label::new(
                    RichText::new("Toggle after action").font(Font::default_medium()),
                ));
                ui.add(Checkbox::without_text(
                    &mut app.settings.toggle_after_action,
                ));
                ui.end_row();
            });
    });
    Ok(())
}
