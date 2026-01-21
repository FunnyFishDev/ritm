use egui::{
    Align, AtomExt, Button, Color32, Image, Layout, RichText, Stroke, TextEdit, Ui, Vec2,
    include_image, vec2,
};

use crate::{
    App,
    error::RitmError,
    turing::StateEdit,
    ui::{font::Font, popup::RitmPopupEnum, theme::Theme},
};

pub fn show(ui: &mut Ui, app: &mut App) -> Result<(), RitmError> {
    ui.set_max_width(300.0);

    ui.with_layout(Layout::top_down(Align::Min), |ui| {
        ui.style_mut().spacing.item_spacing = vec2(10.0, 10.0);

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.add(
                Image::new(include_image!("../../../assets/icon/edit.svg"))
                    .fit_to_exact_size(Vec2::splat(
                        Font::get_heigth(ui, &Font::default_big()) + 4.0,
                    ))
                    .tint(app.theme.overlay),
            );

            if app.turing.state_edit.is_none()
                && let Some(RitmPopupEnum::StateEdit(_, _)) = app.popup.current()
            {
                let state = StateEdit::empty();
                app.turing.state_edit = Some(state);
            }

            let Some(state) = &mut app.turing.state_edit else {
                app.popup.close();
                return;
            };

            let edit = TextEdit::singleline(&mut state.get_edit().name)
                .font(Font::default_big())
                .background_color(Color32::from_black_alpha(20))
                .char_limit(5);

            ui.add(edit);
        });

        let text = RichText::new("Save")
            .color(Theme::constrast_color(app.theme.success))
            .font(Font::default_medium())
            .atom_grow(true);

        let Some(state) = &app.turing.state_edit else {
            app.popup.close();
            return;
        };

        let state_name = state.to().name.clone();

        ui.horizontal_centered(|ui| {
            if ui
                .add_sized(
                    vec2(ui.available_width(), 30.0),
                    Button::new(text)
                        .stroke(Stroke::new(2.0, app.theme.border))
                        .fill(if state_name.is_empty() {
                            app.theme.disabled
                        } else {
                            app.theme.success
                        })
                        .corner_radius(10.0),
                )
                .clicked() && !state_name.is_empty()
            {
                let state_id = if let RitmPopupEnum::StateEdit(_, Some(pos)) =
                    app.popup.current().expect("Should been selected")
                {
                    app.turing.add_state_with_pos(state_name, *pos)
                } else {
                    app.turing.add_state(state_name)
                };

                app.graph.select_state(state_id);
                app.popup.close();
            };
        });
    });
    Ok(())
}
