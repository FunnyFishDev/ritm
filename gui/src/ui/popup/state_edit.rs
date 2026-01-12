use egui::{
    Align, AtomExt, Button, Color32, Context, Frame, Id, Image, Layout, Margin, Modal, RichText,
    Stroke, TextEdit, Ui, Vec2, include_image, vec2,
};

use crate::{
    App,
    ui::{font::Font, popup::RitmPopup, theme::Theme},
};

pub fn show(app: &mut App, ctx: &Context) {
    Modal::new(Id::new("state_edit"))
        .frame(Frame {
            fill: app.theme.white,
            stroke: Stroke::new(2.0, app.theme.gray),
            inner_margin: Margin::same(10),
            corner_radius: 10.into(),
            ..Default::default()
        })
        .show(ctx, |ui: &mut Ui| {
            ui.allocate_ui_with_layout(
                vec2(200.0, 0.0),
                Layout::top_down(Align::Center).with_cross_justify(true),
                |ui| {
                    ui.style_mut().spacing.item_spacing = vec2(0.0, 10.0);

                    ui.allocate_ui_with_layout(
                        vec2(200.0, 0.0),
                        Layout::right_to_left(Align::Center),
                        |ui| {
                            // ui.set_width(200.0);
                            ui.add(
                                Image::new(include_image!("../../../assets/icon/edit.svg"))
                                    .fit_to_exact_size(Vec2::splat(
                                        Font::get_heigth(ui, &Font::default_big()) + 4.0,
                                    ))
                                    .tint(app.theme.gray),
                            );

                            let Some((_state_id, state)) = &mut app.turing.state_edit else {
                                app.popup = RitmPopup::None;
                                return;
                            };

                            let edit = TextEdit::singleline(&mut state.get_edit().name)
                                .font(Font::default_big())
                                .background_color(Color32::from_black_alpha(20))
                                .char_limit(5);

                            ui.add(edit);
                        },
                    );

                    let text = RichText::new("Save")
                        .color(Theme::constrast_color(app.theme.valid))
                        .font(Font::default_medium())
                        .atom_grow(true);

                    let Some((_state_id, state)) = &app.turing.state_edit else {
                        app.popup = RitmPopup::None;
                        return;
                    };

                    if ui
                        .add(
                            Button::new(text)
                                .stroke(Stroke::new(2.0, app.theme.gray))
                                .fill(if state.to().name.is_empty() {
                                    app.theme.gray
                                } else {
                                    app.theme.valid
                                })
                                .corner_radius(10.0),
                        )
                        .clicked()
                    {
                        // no mut borrow
                        // let state = app.temp_state.as_ref().unwrap();
                        app.turing.add_state(state.to().name.to_string());
                        app.event.close_popup = true;
                    };
                },
            );

            if app.event.close_popup {
                app.event.close_popup = false;
                app.popup = RitmPopup::None;
                app.temp_state = None;
            }
        });
}
