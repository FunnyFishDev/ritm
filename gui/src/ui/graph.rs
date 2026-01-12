use std::collections::HashMap;

use egui::{
    Id, Image, ImageButton, LayerId, Rect, Scene, Ui, UiBuilder, Vec2, include_image, vec2,
};
use ritm_core::turing_graph::TuringStateWrapper;

use crate::{
    App, error::RitmError, turing::State, ui::{constant::Constant, edit, popup::RitmPopup, utils}
};

pub mod state;
pub mod transition;

/// Show the graph display of the turing machine
///
/// User can edit the graph and update the code and turing machine
pub fn show(app: &mut App, ui: &mut Ui) -> Result<(), RitmError> {
    // current rect of the element inside the scene
    let mut inner_rect = Rect::ZERO;
    let mut scene_rect = app.graph_rect;

    let ui_rect = ui.available_rect_before_wrap();

    // Compute the force applied on every node
    if !app.event.is_dragging {
        apply_force(app);
    }

    let scene_response = Scene::new()
        .zoom_range(0.0..=1.5)
        .show(ui, &mut scene_rect, |ui| {
            // Draw the transitions of the turing machine
            transition::show(app, ui)?;

            // Draw the states of the turing machine
            state::show(app, ui)?;

            // This Rect can be used to "Reset" the view of the graph
            inner_rect = ui.min_rect();

            Ok::<(), RitmError>(())
        })
        .response;

    // TODO maybe enable the button when small windows but change the behavior to save code as text file directly
    let layer = LayerId::new(egui::Order::Middle, Id::new("graph-button"));
    if !app.event.is_small_window {
        ui.scope_builder(
            UiBuilder::new()
                .layer_id(layer)
                .max_rect(Rect::from_min_size(ui.min_rect().min, vec2(35.0, 35.0))),
            |ui| {
                if ui
                    .put(
                        Rect::from_min_size(ui.min_rect().min, vec2(35.0, 35.0)),
                        ImageButton::new(
                            Image::new(include_image!("../../assets/icon/code.svg"))
                                .fit_to_exact_size(vec2(35.0, 35.0))
                                .tint(app.theme.gray),
                        )
                        .frame(false),
                    )
                    .clicked()
                {
                    app.graph_to_code();
                }
            },
        );
    }

    // Save scene border and recenter if asked
    app.graph_rect = if app.event.need_recenter {
        // TODO better way to recenter, avoid sticking to top
        app.event.need_recenter = false;
        inner_rect
    } else {
        scene_rect
    };

    // Reset the graph (after recenter because need to redraw the states)
    ui.scope_builder(
        UiBuilder::new()
            .layer_id(layer)
            .max_rect(Rect::from_min_size(
                ui.max_rect().right_top() - vec2(35.0, 0.0),
                vec2(35.0, 35.0),
            )),
        |ui| {
            if ui
                .put(
                    Rect::from_min_size(
                        ui.max_rect().right_top() - vec2(45.0, 0.0),
                        vec2(35.0, 35.0),
                    ),
                    ImageButton::new(
                        Image::new(include_image!("../../assets/icon/erase.svg"))
                            .fit_to_exact_size(vec2(35.0, 35.0))
                            .tint(app.theme.gray),
                    )
                    .frame(false),
                )
                .clicked()
            {
                app.turing.reset();
            }
        },
    );

    // If the graph scene is clicked
    if scene_response.clicked() {
        if app.event.is_adding_state {
            let click_pos = scene_response
                .interact_pointer_pos()
                .expect("no click position found");
            app.turing.add_state_with_pos("temp".to_string(), click_pos);
            app.popup = RitmPopup::StateEdit("temp".to_string());
        }

        // CLick on the scene reset selection and editing
        app.event.is_adding_state = !app.settings.toggle_after_action;
        app.event.is_adding_transition = false;
        app.selected_state = None;
        app.selected_transition = None;
    }

    edit::show(app, ui, ui_rect)?;

    // Repaint the canvas
    if !app.event.is_stable {
        ui.ctx().request_repaint();
    }
    Ok(())
}

/// Apply natural force on the node
///
/// If 2 nodes are too close, they repulse each other to reach a distance L
/// If 2 nodes are linked by a transition, they attract each other to reach a distance L
pub fn apply_force(app: &mut App) {
    let mut forces: HashMap<usize, Vec2> = HashMap::new();

    // register the max force applied on a state to check if the system is stable
    let mut max_force_applied: f32 = 0.0;

    let states = app
        .turing
        .tm
        .graph_ref()
        .get_states();

    for i in 0..states.len() {
        let mut force: f32 = 0.0;
        let mut final_force: Vec2 = Vec2::ZERO;

        for j in 0..states.len() {
            // continue if it's the same state
            if j == i {
                continue;
            }

            // true if there is a transition between the two states
            let transition_hashmap = app.turing.tm.graph_ref().get_transitions_hashmap();
            let are_adjacent = transition_hashmap.contains_key(&(i, j))
                || transition_hashmap.contains_key(&(j, i));

            let distance = utils::distance(states[i].inner_state.position, states[j].inner_state.position);
            let direction = utils::direction(states[i].inner_state.position, states[j].inner_state.position);

            // different equations are use based on the adjacency of the states
            if are_adjacent {
                force = utils::attract_force(states[i].inner_state.position, states[j].inner_state.position);
            } else if distance < Constant::L {
                force = -utils::rep_force(states[i].inner_state.position, states[j].inner_state.position);
            };

            // apply the force on the final force vector
            final_force += direction * force;
        }

        // save the highest force applied
        if force.abs() > max_force_applied {
            max_force_applied = force.abs();
        }

        // store the compute force to not alter the current physical state
        forces.insert(i, final_force);
    }

    let mut states_mut: Vec<&mut TuringStateWrapper<State>> =
        app.turing.tm.graph_mut().get_states_mut();

    for (i, state_mut) in states_mut.iter_mut().enumerate() {
        // translate the state by the amount of force
        state_mut.inner_state.position += *forces.get(&i).unwrap();
    }

    app.event.is_stable = max_force_applied < Constant::STABILITY_TRESHOLD;
}
