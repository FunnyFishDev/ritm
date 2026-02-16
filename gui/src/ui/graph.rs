use std::collections::HashMap;

use egui::{
    Align2, Id, Image, ImageButton, LayerId, Pos2, Rect, Scene, Sense, Ui, UiBuilder, Vec2,
    include_image, vec2,
};
use ritm_core::turing_graph::TuringStateWrapper;

use crate::{
    App,
    error::RitmError,
    turing::{State, TransitionId, Turing},
    ui::{
        constant::Constant,
        edit,
        graph::transition::{draw_arrow, draw_self_arrow},
        popup::RitmPopupEnum,
        tutorial::TutorialBox,
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
    drag_transition: Option<(usize, Option<usize>)>,
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

    let graph_rect = ui.available_rect_before_wrap();

    app.tutorial.add_boxe(
        "graph_section",
        TutorialBox::new(graph_rect)
            .with_align(Align2::LEFT_CENTER)
            .with_text_size(vec2(400.0, 500.0)),
    );

    app.tutorial.add_boxe(
        "by_touch",
        TutorialBox::new(graph_rect)
            .with_align(Align2::LEFT_CENTER)
            .with_text_size(vec2(400.0, 500.0)),
    );

    app.tutorial.add_boxe(
        "new_element_creation",
        TutorialBox::new(Rect::from_center_size(graph_rect.center(), Vec2::ZERO))
            .with_text_size(vec2(400.0, 500.0)),
    );

    // Compute the force applied on every node
    if !app.graph.is_dragging {
        apply_force(app);
    }

    let scene_response = Scene::new()
        .sense(if app.tutorial.in_tutorial() {
            Sense::empty()
        } else {
            Sense::click_and_drag()
        })
        .zoom_range(0.0..=1.5)
        .show(ui, &mut scene_rect, |ui| {
            // Draw the transitions of the turing machine
            transition::show(app, ui)?;

            // Draw the states of the turing machine
            state::show(app, ui)?;

            if let Err(x) = transition_dragging(ui, app, graph_rect) {
                println!("{x}");
            }

            // This Rect can be used to "Reset" the view of the graph
            inner_rect = ui.min_rect();

            Ok::<(), RitmError>(())
        })
        .response;

    app.tutorial.add_boxe(
        "initial_state",
        TutorialBox::new(Rect::from_center_size(
            relative_to_absolute(
                graph_rect,
                scene_response.rect,
                app.turing.get_state(0)?.get_inner().position,
            ),
            Vec2::splat(
                (Constant::STATE_RADIUS + 4.0) * 2.0 * graph_rect.width()
                    / scene_response.rect.width(),
            ),
        ))
        .with_align(Align2::LEFT_CENTER),
    );

    app.tutorial.add_boxe(
        "accept_state",
        TutorialBox::new(Rect::from_center_size(
            relative_to_absolute(
                graph_rect,
                scene_response.rect,
                app.turing.get_state(1)?.get_inner().position,
            ),
            Vec2::splat(
                (Constant::STATE_RADIUS + 4.0) * 2.0 * graph_rect.width()
                    / scene_response.rect.width(),
            ),
        ))
        .with_align(Align2::RIGHT_CENTER),
    );

    if scene_response.is_pointer_button_down_on() && !scene_response.dragged() {
        let time = ui.input(|r| r.time);
        let time_down = time - ui.input(|r| r.pointer.press_start_time()).unwrap_or(time);
        if time_down
            > ui.ctx()
                .options(|r| r.input_options.max_click_duration - 0.3)
        {
            app.popup.switch_to(RitmPopupEnum::StateEdit(
                None,
                scene_response.interact_pointer_pos(),
            ));
        }
        ui.ctx().request_repaint();
    }

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
        if app.graph.selected_state.is_some() {
            app.graph.unselect();
        } else if app.edit.is_adding_state {
            let click_pos = scene_response
                .interact_pointer_pos()
                .expect("no click position found");
            app.popup
                .switch_to(RitmPopupEnum::StateEdit(None, Some(click_pos)));
            app.turing.state_edit = None
        }

        // CLick on the scene reset selection and editing
        app.edit.is_adding_state &= !app.settings.reset_after_action;
        app.edit.is_adding_transition = false;
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
        state_mut.inner_state.position += *forces.get(&state_mut.get_id()).expect("Should exist")
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
                let button = ui.put(
                    Rect::from_min_size(ui.min_rect().min, vec2(35.0, 35.0)),
                    ImageButton::new(
                        Image::new(include_image!("../../assets/icon/code.svg"))
                            .fit_to_exact_size(vec2(35.0, 35.0))
                            .tint(app.theme.overlay),
                    )
                    .frame(false),
                );
                if button.clicked() {
                    app.graph_to_code();
                }

                app.tutorial.add_boxe(
                    "to_code",
                    TutorialBox::new(button.rect).with_align(Align2::RIGHT_CENTER),
                );
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
            let button = ui.put(
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
            );
            if button.clicked() {
                app.turing = Turing::default()
            }
            app.tutorial.add_boxe(
                "erase",
                TutorialBox::new(button.rect).with_align(Align2::LEFT_CENTER),
            );
        },
    );
}

fn transition_dragging(ui: &mut Ui, app: &mut App, graph_rect: Rect) -> Result<(), RitmError> {
    if let Some((source_id, target_id)) = app.graph.drag_transition {
        // If the mouse/pen is released then we check if a transition can be added
        if !ui.input(|r| r.pointer.any_down()) {
            if let Some(target_id) = target_id
                && app.turing.add_transition(source_id, target_id).is_ok()
            {
                app.turing.prepare_transition_edit(source_id, target_id)?;
                app.popup
                    .switch_to(RitmPopupEnum::TransitionEdit((source_id, target_id)));
            }

            app.graph.drag_transition = None;
        }
        // We draw the arrow if still down
        else if let Ok(source) = app.turing.get_state(source_id)
            && let Some(absolute_position) = ui.input(|r| r.pointer.latest_pos())
        {
            let target = if graph_rect.contains(absolute_position) {
                absolute_to_relative(ui.clip_rect(), graph_rect, absolute_position)
            } else {
                absolute_to_relative(
                    ui.clip_rect(),
                    graph_rect,
                    absolute_position.clamp(graph_rect.min, graph_rect.max),
                )
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
        state::draw_node(app, ui, source_id)?;
    }

    if let Some((s, _)) = app.graph.drag_transition {
        app.graph.drag_transition = Some((s, None));
    }
    Ok(())
}

fn absolute_to_relative(relative_rect: Rect, absolute_rect: Rect, absolute_position: Pos2) -> Pos2 {
    Pos2::new(
        relative_rect.left()
            + (relative_rect.width() * (absolute_position.x - absolute_rect.left())
                / absolute_rect.width()),
        relative_rect.top()
            + (relative_rect.height() * (absolute_position.y - absolute_rect.top())
                / absolute_rect.height()),
    )
}

fn relative_to_absolute(absolute_rect: Rect, relative_rect: Rect, relative_pos: Pos2) -> Pos2 {
    Pos2::new(
        absolute_rect.left()
            + (absolute_rect.width() * (relative_pos.x - relative_rect.left())
                / relative_rect.width()),
        absolute_rect.top()
            + (absolute_rect.height() * (relative_pos.y - relative_rect.top())
                / relative_rect.height()),
    )
}
