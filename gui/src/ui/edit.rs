use egui::{
    Align, Align2, DragValue, Frame, Id, Image, ImageButton, ImageSource, LayerId, Layout, Margin,
    Pos2, Rect, Response, Sense, Stroke, Ui, UiBuilder, Vec2, include_image, vec2,
};
use ritm_core::turing_graph::TuringGraph;

use crate::{
    App,
    error::RitmError,
    turing::Turing,
    ui::{
        constant::Constant,
        popup::RitmPopupEnum,
        tutorial::{TutorialBox, TutorialEnum},
    },
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
            if let Some(tutorial) = app.tutorial.current_tutorial()
                && tutorial == TutorialEnum::Edit
                && app.tutorial.current_step() == 5
            {
                app.graph.select_state(0);
            }

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
            let edit = ui
                .allocate_ui_with_layout(
                    vec2(icon_size, ui.available_height()),
                    Layout::bottom_up(Align::Center).with_cross_align(Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing = vec2(0.0, 5.0);

                        // State
                        // Only possible to add a state if nothing is selected
                        // IDEA : maybe permit it for state selected, and create a transition directly
                        if none_selected {
                            let state = button(
                                ui,
                                app,
                                include_image!("../../assets/icon/stateplus.svg"),
                                app.edit.is_adding_state,
                            );
                            if state.clicked() {
                                app.edit.is_adding_state ^= true;
                            }
                            app.tutorial.add_boxe(
                                "add_state",
                                TutorialBox::new(state.rect.expand(6.0))
                                    .with_align(Align2::LEFT_TOP),
                            );
                        }

                        // Transition
                        // Only possible to create transition if a state is selected
                        if state_selected {
                            let button = button(
                                ui,
                                app,
                                include_image!("../../assets/icon/transition.svg"),
                                app.edit.is_adding_transition,
                            );
                            if button.clicked() {
                                app.edit.is_adding_transition ^= true;
                            }

                            app.tutorial.add_boxe(
                                "add_transition",
                                TutorialBox::new(button.rect.expand(6.0))
                                    .with_align(Align2::LEFT_TOP),
                            );
                        }

                        // Delete
                        // If a state or transition is selected, then display the delete button
                        if either_selected && app.graph.selected_state().is_some_and(|s| s > 1) {
                            let delete = button(
                                ui,
                                app,
                                include_image!("../../assets/icon/delete.svg"),
                                false,
                            );
                            if delete.clicked() {
                                if let Some(state_selected) = app.graph.selected_state() {
                                    app.turing.remove_state(state_selected)?;
                                }

                                if let Some(transition_selected) = app.graph.selected_transitions()
                                {
                                    app.turing.remove_transitions(
                                        transition_selected.source_id,
                                        transition_selected.target_id,
                                    )?;
                                }
                            }
                            app.tutorial.add_boxe(
                                "delete",
                                TutorialBox::new(delete.rect.expand(6.0))
                                    .with_align(Align2::LEFT_TOP),
                            );
                        }

                        // Edit the selected transitions or state
                        if either_selected {
                            let edit = button(
                                ui,
                                app,
                                include_image!("../../assets/icon/edit.svg"),
                                false,
                            );
                            if edit.clicked() {
                                if let Some(state_selected) = app.graph.selected_state() {
                                    app.edit_state(state_selected)?
                                }

                                if let Some(transition_selected) = app.graph.selected_transitions()
                                {
                                    app.popup.switch_to(RitmPopupEnum::TransitionEdit((
                                        transition_selected.source_id,
                                        transition_selected.target_id,
                                    )));

                                    app.turing.prepare_transition_edit(
                                        transition_selected.source_id,
                                        transition_selected.target_id,
                                    )?;
                                }
                            }
                            app.tutorial.add_boxe(
                                "edit",
                                TutorialBox::new(edit.rect.expand(6.0))
                                    .with_align(Align2::LEFT_TOP),
                            );
                        }

                        // Recenter the graph
                        let recenter = button(
                            ui,
                            app,
                            include_image!("../../assets/icon/recenter.svg"),
                            false,
                        );
                        if recenter.clicked() {
                            app.graph.recenter();
                        }

                        app.tutorial.add_boxe(
                            "recenter",
                            TutorialBox::new(recenter.rect.expand(6.0))
                                .with_align(Align2::LEFT_TOP),
                        );

                        // Unpin every state of the graph
                        let unpin = button(
                            ui,
                            app,
                            include_image!("../../assets/icon/unpin.svg"),
                            false,
                        );
                        if unpin.clicked() {
                            app.turing.unpin_all();
                        }

                        app.tutorial.add_boxe(
                            "unpin",
                            TutorialBox::new(unpin.rect).with_align(Align2::LEFT_TOP),
                        );

                        let tape_counter = tape_dropdown(ui, app);

                        app.tutorial.add_boxe(
                            "tape_counter",
                            TutorialBox::new(tape_counter.rect).with_align(Align2::LEFT_TOP),
                        );

                        Ok::<(), RitmError>(())
                    },
                )
                .response;
            app.tutorial.add_boxe(
                "edit_section",
                TutorialBox::new(edit.rect.expand(3.0)).with_align(Align2::LEFT_TOP),
            );
            app.tutorial.add_boxe(
                "by_edit",
                TutorialBox::new(edit.rect.expand(3.0)).with_align(Align2::LEFT_CENTER),
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

fn tape_dropdown(ui: &mut Ui, app: &mut App) -> Response {
    let margin = 5;
    Frame::new()
        .stroke(Stroke::new(1.0, app.theme.border))
        .corner_radius(app.edit.icon_size / 2.0)
        .fill(app.theme.surface)
        .inner_margin(Margin::same(margin))
        .show(ui, |ui| {
            let mut k = app.turing.tm.graph_ref().get_k();
            ui.spacing_mut().interact_size.x = 0.0;
            ui.spacing_mut().button_padding = vec2(0.0, 0.0);
            ui.visuals_mut().selection.stroke = Stroke::new(1.0, app.theme.border);
            if ui
                .add_sized(
                    vec2(ui.available_width(), 0.0),
                    DragValue::new(&mut k)
                        .range(0..=9)
                        .clamp_existing_to_range(true)
                        .update_while_editing(false),
                )
                .changed()
            {
                app.graph.reset();
                app.turing = Turing::new_graph(TuringGraph::new(k, true).expect("should work"));
                app.turing.layer_graph();
            }
        })
        .response
}
