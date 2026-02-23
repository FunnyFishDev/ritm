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
                && let Some(RitmPopupEnum::StateEdit(selected)) = app.popup.current()
            {
                app.turing.state_edit = if let Some(state) = *selected
                    && let Some(state) = app.turing.get_state(state).ok()
                {
                    Some(StateEdit::from(state))
                } else {
                    let mut edit = StateEdit::empty();
                    edit.get_edit().name = format!(
                        "q_{}",
                        app.turing.tm.graph_ref().get_state_hashmap().iter().count() + 1
                    );
                    Some(edit)
                }
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

        ui.columns(2, |ui| {
            let text = RichText::new("Cancel")
                .color(Theme::constrast_color(app.theme.error))
                .font(Font::default_medium())
                .atom_grow(true);

            if ui[1]
                .add(
                    Button::new(text)
                        .stroke(Stroke::new(2.0, app.theme.border))
                        .fill(app.theme.error)
                        .corner_radius(10.0),
                )
                .clicked()
            {
                app.popup.close();
            }

            let Some(state) = &app.turing.state_edit else {
                app.popup.close();
                return Ok(());
            };

            let state_name = state.to().name.clone();

            if ui[0]
                .add(
                    Button::new(
                        RichText::new("Save")
                            .color(Theme::constrast_color(app.theme.success))
                            .font(Font::default_medium())
                            .atom_grow(true),
                    )
                    .stroke(Stroke::new(2.0, app.theme.border))
                    .fill(if state_name.is_empty() {
                        app.theme.disabled
                    } else {
                        app.theme.success
                    })
                    .corner_radius(10.0),
                )
                .clicked()
                && !state_name.is_empty()
            {
                let state_id = app.turing.apply_state_change()?;

                app.turing.state_edit = None;
                app.graph.select_state(state_id);
                app.popup.close();
            };
            Ok::<(), RitmError>(())
        })
    })
    .inner
}
