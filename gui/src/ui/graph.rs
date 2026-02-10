use std::collections::HashMap;

use egui::{
    Id, Image, ImageButton, LayerId, Pos2, Rect, Scene, Ui, UiBuilder, Vec2, include_image, vec2,
};
use ritm_core::turing_graph::TuringStateWrapper;

use crate::{
    App,
    error::RitmError,
    turing::{State, TransitionId},
    ui::{
        constant::Constant,
        edit,
        graph::transition::{draw_arrow, draw_self_arrow},
        popup::RitmPopupEnum,
        utils,
    },
};

pub mod state;
pub mod transition;

pub struct Graph {
    selected_state: Option<usize>,
    selected_transitions: Option<TransitionId>,
    graph_rect: Rect,
    recenter: bool,
    is_stable: bool,
    is_dragging: bool,
    drag_transition: Option<usize>,
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            selected_state: Default::default(),
            selected_transitions: Default::default(),
            graph_rect: Rect::ZERO,
            recenter: false,
            is_stable: false,
            is_dragging: false,
            drag_transition: None,
        }
    }
}

impl Graph {
    /// Return None if no state are selected
    pub fn selected_state(&self) -> Option<usize> {
        self.selected_state
    }

    /// Return None if no transitions are selected
    pub fn selected_transitions(&self) -> Option<TransitionId> {
        self.selected_transitions
    }

    pub fn select_state(&mut self, state_id: usize) {
        self.selected_state = Some(state_id);
        self.selected_transitions = None;
    }

    pub fn select_transitions(&mut self, transition_id: TransitionId) {
        self.selected_transitions = Some(transition_id);
        self.selected_state = None;
    }

    /// Unselect state or transition selected
    pub fn unselect(&mut self) {
        self.selected_state = None;
        self.selected_transitions = None;
    }

    /// Request to recenter the graph
    pub fn recenter(&mut self) {
        self.recenter = true;
    }

    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }
}

/// Show the graph display of the turing machine
///
/// User can edit the graph and update the code and turing machine
pub fn show(app: &mut App, ui: &mut Ui) -> Result<(), RitmError> {
    // current rect of the element inside the scene
    let mut inner_rect = Rect::ZERO;

    let mut scene_rect = app.graph.graph_rect;

    // Compute the force applied on every node
    if !app.graph.is_dragging {
        apply_force(app);
    }

    let graph_rect = ui.available_rect_before_wrap();

    let scene_response = Scene::new()
        .zoom_range(0.0..=1.5)
        .show(ui, &mut scene_rect, |ui| {
            // Draw the transitions of the turing machine
            transition::show(app, ui)?;

            transition_dragging(ui, app, graph_rect)?;

            // Draw the states of the turing machine
            state::show(app, ui)?;

            // This Rect can be used to "Reset" the view of the graph
            inner_rect = ui.min_rect();

            Ok::<(), RitmError>(())
        })
        .response;

    // TODO maybe enable the button when small windows but change the behavior to save code as text file directly
    let layer = LayerId::new(egui::Order::Middle, Id::new("graph-button"));

    // Convert the graph to code
    to_code_button(ui, app, layer);

    // Save scene border and recenter if asked
    // TODO better way to recenter, avoid sticking to top
    app.graph.graph_rect = if app.graph.recenter {
        app.graph.recenter = false;
        inner_rect
    } else {
        scene_rect
    };

    // Reset the graph (after recenter because need to redraw the states)
    reset_button(ui, app, layer);

    // If the graph scene is clicked
    // TODO: need to rework state adding flow

    if scene_response.clicked() {
        if app.edit.is_adding_state {
            let click_pos = scene_response
                .interact_pointer_pos()
                .expect("no click position found");
            app.popup
                .switch_to(RitmPopupEnum::StateEdit(None, Some(click_pos)));
            app.turing.state_edit = None
        }

        // CLick on the scene reset selection and editing
        app.edit.is_adding_state &= !app.settings.reset_after_action;
        app.edit.is_adding_transition &= !app.settings.reset_after_action;
        app.graph.unselect();
    }

    edit::show(app, ui)?;

    // Repaint the canvas
    if !app.graph.is_stable {
        ui.ctx().request_repaint();
    }
    Ok(())
}

/// Apply natural force on the node
///
/// If 2 nodes are too close, they repulse each other to reach a distance L
/// If 2 nodes are linked by a transition, they attract each other to reach a distance L
fn apply_force(app: &mut App) {
    let mut forces: HashMap<usize, Vec2> = HashMap::new();

    // register the max force applied on a state to check if the system is stable
    let mut max_force_applied: f32 = 0.0;

    let states = app.turing.tm.graph_ref().get_states();

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

            let distance = utils::distance(
                states[i].inner_state.position,
                states[j].inner_state.position,
            );
            let direction = utils::direction(
                states[i].inner_state.position,
                states[j].inner_state.position,
            );

            // different equations are use based on the adjacency of the states
            if are_adjacent {
                force = utils::attract_force(
                    states[i].inner_state.position,
                    states[j].inner_state.position,
                );
            } else if distance < Constant::L {
                force = -utils::rep_force(
                    states[i].inner_state.position,
                    states[j].inner_state.position,
                );
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

    for state_mut in states_mut.iter_mut().filter(|s| !s.inner_state.is_pinned) {
        // translate the state by the amount of force
        state_mut.inner_state.position += *forces.get(&state_mut.get_id()).unwrap();
    }

    app.graph.is_stable = max_force_applied < Constant::STABILITY_TRESHOLD;
}

/// Button to convert the current displayed graph into code
fn to_code_button(ui: &mut Ui, app: &mut App, layer: LayerId) {
    if !app.transient.is_small_window {
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
                                .tint(app.theme.overlay),
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
}

/// Button to reset the graph to the initial and accepting state
fn reset_button(ui: &mut Ui, app: &mut App, layer: LayerId) {
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
                            .tint(app.theme.overlay),
                    )
                    .frame(false),
                )
                .clicked()
            {
                app.turing.reset();
            }
        },
    );
}

fn transition_dragging(ui: &mut Ui, app: &mut App, graph_rect: Rect) -> Result<(), RitmError> {
    if let Some(source_id) = app.graph.drag_transition
        && let Ok(source) = app.turing.get_state(source_id)
    {
        if !ui.input(|r| r.pointer.any_down()) {
            app.graph.drag_transition = None;
        }

        if let Some(absolute_position) = ui.input(|r| r.pointer.latest_pos()) {
            let target = if graph_rect.contains(absolute_position) {
                Pos2::new(
                    ui.clip_rect().left()
                        + (ui.clip_rect().width() * (absolute_position.x - graph_rect.left())
                            / graph_rect.width()),
                    ui.clip_rect().top()
                        + (ui.clip_rect().height() * (absolute_position.y - graph_rect.top())
                            / graph_rect.height()),
                )
            } else {
                absolute_position
            };

            if Rect::from_center_size(
                source.get_inner().position,
                Vec2::splat(Constant::STATE_RADIUS * 2.0),
            )
            .contains(target)
            {
                let transition_vec = app.turing.best_vector(source_id)?;
                let _ = draw_self_arrow(app, ui, source.get_inner().position, transition_vec);
            } else {
                let _ = draw_arrow(app, ui, source.get_inner().position, target, None);
            }
        }
    }
    Ok(())
}
