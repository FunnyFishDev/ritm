use egui::{Align, Label, Rect, RichText, Sense, Stroke, Ui, vec2};

use crate::{
    App,
    turing::State,
    ui::{constant::Constant, font::Font, theme::Theme},
};

/// Display every state of the turing machine
pub fn show(app: &mut App, ui: &mut Ui) {
    // This line copy every keys of the hasmap to avoid borrowing the struct App that we need in each call.
    let keys: Vec<usize> = app.states.keys().copied().collect::<Vec<usize>>();
    for i in keys {
        draw_node(app, ui, i);
    }
}

/// Draw a single state
fn draw_node(app: &mut App, ui: &mut Ui, state_id: usize) {
    // Get the state information
    let state = State::get(app, state_id);

    // Define the boundaries of the node
    let rect = Rect::from_center_size(
        state.position,
        vec2(Constant::STATE_RADIUS, Constant::STATE_RADIUS) * 2.0,
    );

    // Draw the node circle
    ui.painter().circle(
        state.position,
        Constant::STATE_RADIUS,
        state.color,
        if app.selected_state.is_some_and(|id| id == state_id) {
            Stroke::new(4.0, app.theme.selected)
        } else if let Some(current_state_id) = app
            .turing
            .graph_ref()
            .get_name_index_hashmap()
            .get(&app.step.get_current_state().name)
            && *current_state_id == state_id
        {
            Stroke::new(4.0, app.theme.highlight)
        } else {
            Stroke::new(2.0, app.theme.gray)
        },
    );

    let name = RichText::new(&state.name)
        .font(Font::default_big())
        .color(Theme::constrast_color(state.color));

    let label = Label::new(name).wrap().halign(Align::Center);

    // Draw the label inside the node, without overflow
    ui.put(rect, label);

    // Listen for click and drag event on the node
    let response = ui.allocate_rect(rect, Sense::click_and_drag());

    if response.clicked() {
        if app.event.is_adding_transition {
            app.add_transition(state_id);
        } else {
            app.selected_transition = None;
            app.selected_state = Some(state_id)
        }
    }

    // If dragged, make the node follow the pointer
    if response.dragged() {
        app.states.get_mut(&state_id).unwrap().position = response.interact_pointer_pos().unwrap();
        State::get_mut(app, state_id).is_pinned = true;
    }

    if response.drag_started() {
        app.event.is_dragging = true
    }
    if response.drag_stopped() {
        app.event.is_dragging = false
    }
}
