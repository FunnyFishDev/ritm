use std::collections::HashMap;

use egui::{
    Id, Image, ImageButton, LayerId, Rect, Scene, Ui, UiBuilder, Vec2, include_image, vec2,
};

use crate::{
    App,
    turing::State,
    ui::{constant::Constant, edit, popup::RitmPopup, utils},
};

pub mod state;
pub mod transition;

/// Show the graph display of the turing machine
///
/// User can edit the graph and update the code and turing machine
pub fn show(app: &mut App, ui: &mut Ui) {
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
            transition::show(app, ui);

            // Draw the states of the turing machine
            state::show(app, ui);

            // This Rect can be used to "Reset" the view of the graph
            inner_rect = ui.min_rect();
        })
        .response;

    // TODO maybe enable the button when samll windows but change the behavior to save code as text file directly
    if !app.event.is_small_window {
        let layer = LayerId::new(egui::Order::Middle, Id::new("to-code"));
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

    // If the graph scene is clicked
    if scene_response.clicked() {
        if app.event.is_adding_state {
            app.temp_state = Some(State::new_at_pos(
                0,
                "temp".to_string(),
                scene_response.interact_pointer_pos().unwrap(),
            ));
            app.popup = RitmPopup::StateEdit;
        }

        // CLick on the scene reset selection and editing
        app.event.is_adding_state = !app.settings.toggle_after_action;
        app.event.is_adding_transition = false;
        app.selected_state = None;
        app.selected_transition = None;
    }

    edit::show(app, ui, ui_rect);

    // Repaint the canvas
    if !app.event.is_stable {
        ui.ctx().request_repaint();
    }
}

/// Apply natural force on the node
///
/// If 2 nodes are too close, they repulse each other to reach a distance L
/// If 2 nodes are linked by a transition, they attract each other to reach a distance L
pub fn apply_force(app: &mut App) {
    let mut forces: HashMap<usize, Vec2> = HashMap::new();

    // register the max force applied on a state to check if the system is stable
    let mut max_force_applied: f32 = 0.0;

    for (i, state_1) in app.states.iter() {
        let mut force: f32 = 0.0;
        let mut final_force: Vec2 = Vec2::ZERO;

        for (j, state_2) in app.states.iter() {
            // continue if it's the same state
            if j == i {
                continue;
            }

            // true if there is a transition between the two states
            let are_adjacent = app
                .turing
                .graph_ref()
                .get_transitions_by_index(*i, *j)
                .is_ok_and(|v| !v.is_empty())
                || app
                    .turing
                    .graph_ref()
                    .get_transitions_by_index(*j, *i)
                    .is_ok_and(|v| !v.is_empty());

            let distance = utils::distance(state_1.position, state_2.position);
            let direction = utils::direction(state_1.position, state_2.position);

            // different equations are use based on the adjacency of the states
            if are_adjacent {
                force = utils::attract_force(state_1.position, state_2.position);
            } else if distance < Constant::L {
                force = -utils::rep_force(state_1.position, state_2.position);
            };

            // apply the force on the final force vector
            final_force += direction * force;
        }

        // save the highest force applied
        if force.abs() > max_force_applied {
            max_force_applied = force.abs();
        }

        // store the compute force to not alter the current physical state
        forces.insert(*i, final_force);
    }

    for (i, state) in app.states.iter_mut().filter(|f| !f.1.is_pinned) {
        // translate the state by the amount of force
        state.position += *forces.get(i).unwrap();
    }

    app.event.is_stable = max_force_applied < Constant::STABILITY_TRESHOLD;
}
