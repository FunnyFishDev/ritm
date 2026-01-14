use egui::{
    Align, Color32, Frame, Id, Image, ImageButton, LayerId, Layout, Margin, Rect, Sense, Shadow,
    Stroke, Ui, UiBuilder, Vec2, include_image, vec2,
};

use crate::{
    App,
    app::take_screenshot,
    error::RitmError,
    ui::{constant::Constant, popup::RitmPopupEnum, theme::Theme},
};

/// Control of the graph
pub fn show(app: &mut App, ui: &mut Ui, rect: Rect) -> Result<(), RitmError> {
    let icon_size = Constant::scale(ui, Constant::ICON_SIZE);

    // Floating control absolute position
    ui.scope_builder(
        UiBuilder::new()
            .max_rect(Rect::from_center_size(
                rect.center_bottom() + vec2(0.0, -icon_size * 3.0),
                (rect.width(), 1.0).into(),
            ))
            .sense(Sense::empty())
            .layout(Layout::top_down(Align::Center).with_cross_align(Align::Center)),
        |ui| {
            // The parent ui paint on the background layer, so we need to change it to a higher layer
            let layer = LayerId::new(egui::Order::Middle, Id::new("edit"));
            ui.scope_builder(UiBuilder::new().layer_id(layer), |ui| {
                // Used for background
                Frame::new()
                    .fill(app.theme.white)
                    .inner_margin(Margin::symmetric(
                        Constant::scale(ui, 20.0) as i8,
                        Constant::scale(ui, 10.0) as i8,
                    ))
                    .corner_radius(Constant::scale(ui, 15.0) as u8)
                    .stroke(Stroke::new(1.0, app.theme.gray))
                    .shadow(Shadow {
                        offset: [0, 4],
                        blur: 4,
                        spread: 0,
                        color: Color32::from_black_alpha(25),
                    })
                    .show(ui, |ui| {
                        let state_selected = app.selected_state;
                        let transition_selected = app.selected_transition;

                        // Need to compute the width of the menu to center it
                        // Recenter/Unpin/Pin
                        let mut count = 2;
                        if state_selected.is_none() && transition_selected.is_none() {
                            count += 1;
                        }
                        if state_selected.is_some() {
                            // Create transition from the selected state
                            count += 1;
                        }
                        if state_selected.is_some() || transition_selected.is_some() {
                            // Delete/Edit state/transition
                            count += 1;
                        }
                        if transition_selected.is_some() {
                            count += 1;
                        }
                        let width = icon_size * count as f32 + 20.0 * (count - 1) as f32;

                        ui.allocate_ui_with_layout(
                            vec2(width, icon_size),
                            Layout::left_to_right(Align::Min).with_cross_align(Align::Center),
                            |ui| {
                                ui.spacing_mut().item_spacing = vec2(20.0, 0.0);

                                // State
                                // Only possible to add a state if nothing is selected
                                // IDEA : maybe permit it for state selected, and create a transition directly
                                if app.selected_state.is_none()
                                    && app.selected_transition.is_none()
                                    && ui
                                        .add(
                                            ImageButton::new(
                                                Image::new(include_image!(
                                                    "../../assets/icon/stateplus.svg"
                                                ))
                                                .fit_to_exact_size(Vec2::splat(icon_size))
                                                .tint(if app.event.is_adding_state {
                                                    app.theme.selected
                                                } else {
                                                    Theme::constrast_color(app.theme.white)
                                                }),
                                            )
                                            .frame(false),
                                        )
                                        .clicked()
                                {
                                    app.event.is_adding_state ^= true;
                                }

                                // Transition
                                // Only possible to create transition if a state is selected
                                if app.selected_state.is_some()
                                    && ui
                                        .add(
                                            ImageButton::new(
                                                Image::new(include_image!(
                                                    "../../assets/icon/transition.svg"
                                                ))
                                                .fit_to_exact_size(Vec2::splat(icon_size))
                                                .tint(if app.event.is_adding_transition {
                                                    app.theme.selected
                                                } else {
                                                    Theme::constrast_color(app.theme.white)
                                                }),
                                            )
                                            .frame(false),
                                        )
                                        .clicked()
                                {
                                    app.event.is_adding_transition ^= true;
                                }

                                // Delete
                                // If a state or transition is selected, then display the delete button
                                if (app.selected_state.is_some()
                                    || app.selected_transition.is_some())
                                    && ui
                                        .add(
                                            ImageButton::new(
                                                Image::new(include_image!(
                                                    "../../assets/icon/delete.svg"
                                                ))
                                                .fit_to_exact_size(Vec2::splat(icon_size))
                                                .tint(Theme::constrast_color(app.theme.white)),
                                            )
                                            .frame(false),
                                        )
                                        .clicked()
                                {
                                    if let Some(state_selected) = state_selected {
                                        app.turing.remove_state(state_selected)?;
                                    }

                                    if let Some(transition_selected) = transition_selected {
                                        app.turing.remove_transitions(
                                            transition_selected.source_id,
                                            transition_selected.target_id,
                                        )?;
                                    }
                                }

                                if (app.selected_transition.is_some())
                                    && ui
                                        .add(
                                            ImageButton::new(
                                                Image::new(include_image!(
                                                    "../../assets/icon/edit.svg"
                                                ))
                                                .fit_to_exact_size(Vec2::splat(icon_size))
                                                .tint(Theme::constrast_color(app.theme.white)),
                                            )
                                            .frame(false),
                                        )
                                        .clicked()
                                {
                                    if let Some(state_selected) = state_selected {
                                        app.popup.switch_to(RitmPopupEnum::StateEdit(format!(
                                            "State {}",
                                            app.turing.get_state(state_selected)?.get_name()
                                        )));
                                    }
                                    if let Some(transition_selected) = transition_selected {
                                        app.popup.switch_to(RitmPopupEnum::TransitionEdit(format!(
                                            "Transition {} -> {}",
                                            app.turing.get_state(transition_selected.source_id)?.get_name(),
                                            app.turing.get_state(transition_selected.target_id)?.get_name()
                                        )));
                                        
                                        app.turing.prepare_transition_edit(
                                            transition_selected.source_id,
                                            transition_selected.target_id,
                                        );
                                    }
                                }

                                if ui
                                    .add(
                                        ImageButton::new(
                                            Image::new(include_image!(
                                                "../../assets/icon/recenter.svg"
                                            ))
                                            .fit_to_exact_size(Vec2::splat(icon_size))
                                            .tint(Theme::constrast_color(app.theme.white)),
                                        )
                                        .frame(false),
                                    )
                                    .clicked()
                                {
                                    app.event.need_recenter = true;
                                }

                                if ui
                                    .add(
                                        ImageButton::new(
                                            Image::new(include_image!(
                                                "../../assets/icon/unpin.svg"
                                            ))
                                            .fit_to_exact_size(Vec2::splat(icon_size))
                                            .tint(Theme::constrast_color(app.theme.white)),
                                        )
                                        .frame(false),
                                    )
                                    .clicked()
                                {
                                    app.turing.unpin();
                                }
                                Ok::<(), RitmError>(())
                            },
                        );
                    });
                take_screenshot(app, ui);
            });
        },
    );
    Ok(())
}
