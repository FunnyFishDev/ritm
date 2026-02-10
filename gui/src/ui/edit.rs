use egui::{
    Align, Frame, Id, Image, ImageButton, ImageSource, LayerId, Layout, Margin, Pos2, Rect,
    Response, Sense, Stroke, Ui, UiBuilder, Vec2, include_image, vec2,
};

use crate::{
    App,
    error::RitmError,
    turing::StateEdit,
    ui::{constant::Constant, popup::RitmPopupEnum},
};

pub struct Edit {
    icon_size: f32,
    pub is_adding_state: bool,
    pub is_adding_transition: bool,
}

impl Default for Edit {
    fn default() -> Self {
        Self {
            icon_size: 25.0,
            is_adding_state: false,
            is_adding_transition: false,
        }
    }
}

/// Control of the graph
pub fn show(app: &mut App, ui: &mut Ui) -> Result<(), RitmError> {
    let icon_size = Constant::scale(ui, Constant::ICON_SIZE);

    // The parent ui paint on the background layer, so we need to change it to a higher layer
    let layer = LayerId::new(egui::Order::Middle, Id::new("edit"));
    let ui_rect = ui.available_rect_before_wrap();
    // Floating control absolute position
    ui.scope_builder(
        UiBuilder::new()
            .max_rect(Rect::from_min_max(
                Pos2::new(ui_rect.right() - icon_size - 10.0, ui_rect.top()),
                Pos2::new(ui_rect.right() - 10.0, ui_rect.bottom() - 10.0),
            ))
            .sense(Sense::empty())
            .layout(Layout::bottom_up(Align::Center).with_cross_align(Align::Center))
            .layer_id(layer),
        |ui| {
            // TODO: replace with flags from bitflags crate
            let state_selected =
                app.graph.selected_state().is_some() && app.graph.selected_transitions().is_none();
            let _transition_selected =
                app.graph.selected_transitions().is_some() && app.graph.selected_state().is_none();
            let _both_selected =
                app.graph.selected_state().is_some() && app.graph.selected_transitions().is_some();
            let either_selected =
                app.graph.selected_state().is_some() || app.graph.selected_transitions().is_some();
            let none_selected =
                app.graph.selected_state().is_none() && app.graph.selected_transitions().is_none();

            // Vertical alignment, bottom to up
            ui.allocate_ui_with_layout(
                vec2(icon_size, ui.available_height()),
                Layout::bottom_up(Align::Center).with_cross_align(Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 5.0);

                    // State
                    // Only possible to add a state if nothing is selected
                    // IDEA : maybe permit it for state selected, and create a transition directly
                    if none_selected
                        && button(
                            ui,
                            app,
                            include_image!("../../assets/icon/stateplus.svg"),
                            app.edit.is_adding_state,
                        )
                        .clicked()
                    {
                        app.edit.is_adding_state ^= true;
                    }

                    // Transition
                    // Only possible to create transition if a state is selected
                    if state_selected
                        && button(
                            ui,
                            app,
                            include_image!("../../assets/icon/transition.svg"),
                            app.edit.is_adding_transition,
                        )
                        .clicked()
                    {
                        app.edit.is_adding_transition ^= true;
                        app.edit.is_adding_transition ^=
                            app.settings.reset_after_action || !app.edit.is_adding_transition;
                    }

                    // Delete
                    // If a state or transition is selected, then display the delete button
                    if either_selected
                        && button(
                            ui,
                            app,
                            include_image!("../../assets/icon/delete.svg"),
                            false,
                        )
                        .clicked()
                    {
                        if let Some(state_selected) = app.graph.selected_state() {
                            app.turing.remove_state(state_selected)?;
                        }

                        if let Some(transition_selected) = app.graph.selected_transitions() {
                            app.turing.remove_transitions(
                                transition_selected.source_id,
                                transition_selected.target_id,
                            )?;
                        }
                    }

                    // Edit the selected transitions or state
                    if either_selected
                        && button(ui, app, include_image!("../../assets/icon/edit.svg"), false)
                            .clicked()
                    {
                        if let Some(state_selected) = app.graph.selected_state() {
                            app.popup
                                .switch_to(RitmPopupEnum::StateEdit(Some(state_selected), None));

                            app.turing.state_edit =
                                Some(StateEdit::from(app.turing.get_state(state_selected)?));
                        }

                        if let Some(transition_selected) = app.graph.selected_transitions() {
                            app.popup
                                .switch_to(RitmPopupEnum::TransitionEdit(transition_selected));

                            app.turing.prepare_transition_edit(
                                transition_selected.source_id,
                                transition_selected.target_id,
                            )?;
                        }
                    }

                    // Recenter the graph
                    if button(
                        ui,
                        app,
                        include_image!("../../assets/icon/recenter.svg"),
                        false,
                    )
                    .clicked()
                    {
                        app.graph.recenter();
                    }

                    // Unpin every state of the graph
                    if button(
                        ui,
                        app,
                        include_image!("../../assets/icon/unpin.svg"),
                        false,
                    )
                    .clicked()
                    {
                        app.turing.unpin();
                    }

                    Ok::<(), RitmError>(())
                },
            );
        },
    );
    Ok(())
}

fn button(ui: &mut Ui, app: &mut App, icon: ImageSource, selected: bool) -> Response {
    let margin = 5;
    Frame::new()
        .stroke(Stroke::new(1.0, app.theme.border))
        .corner_radius(app.edit.icon_size / 2.0)
        .fill(app.theme.surface)
        .inner_margin(Margin::same(margin))
        .show(ui, |ui| {
            ui.add(
                ImageButton::new(
                    Image::new(icon)
                        .fit_to_exact_size(Vec2::splat(app.edit.icon_size))
                        .tint(if selected {
                            app.theme.active
                        } else {
                            app.theme.overlay
                        }),
                )
                .frame(false)
                .corner_radius(app.edit.icon_size / 2.0),
            )
        })
        .inner
}
