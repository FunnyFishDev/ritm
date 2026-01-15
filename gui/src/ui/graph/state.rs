use egui::{Align, Label, Rect, RichText, Sense, Stroke, Ui, vec2};

use crate::{
    App, error::RitmError, ui::{constant::Constant, font::Font, theme::Theme}
};

/// Display every state of the turing machine
pub fn show(app: &mut App, ui: &mut Ui) -> Result<(), RitmError> {
    // This line copy every keys of the hasmap to avoid borrowing the struct App that we need in each call.
    let keys: Vec<usize> = app.turing.tm.graph_ref().get_state_hashmap().keys().copied().collect();
    for i in keys {
        draw_node(app, ui, i)?;
    }
    Ok(())
}

/// Draw a single state
fn draw_node(app: &mut App, ui: &mut Ui, state_id: usize) -> Result<(), RitmError> {
    // Get the state information
    let state = app.turing.tm.graph_mut().get_state(state_id).expect("state exist");

    // Define the boundaries of the node
    let rect = Rect::from_center_size(
        state.inner_state.position,
        vec2(Constant::STATE_RADIUS, Constant::STATE_RADIUS) * 2.0,
    );

    // Draw the node circle
    ui.painter().circle(
        state.inner_state.position,
        Constant::STATE_RADIUS,
        state.inner_state.color,
        if app.graph.selected_state.is_some_and(|id| id == state_id) {
            Stroke::new(4.0, app.theme.selected)
        } else if app.turing.current_step.get_state_pointer() == state_id {
            Stroke::new(4.0, app.theme.highlight)
        } else {
            Stroke::new(2.0, app.theme.gray)
        },
    );

    let name = RichText::new(state.get_name())
        .font(Font::default_big())
        .color(Theme::constrast_color(state.inner_state.color));

    let label = Label::new(name).wrap().halign(Align::Center);

    // Draw the label inside the node, without overflow
    ui.put(rect, label);

    // Listen for click and drag event on the node
    let response = ui.allocate_rect(rect, Sense::click_and_drag());

    if response.clicked() {
        if app.event.is_adding_transition {
            app.turing.add_transition(app.graph.selected_state().expect("state selected"), state_id)?;
        } else {
            app.graph.select_state(state_id);
            app.event.is_adding_state = false;
        }
    }

    // Reborrow the state as a mut this time
    let state = app
        .turing
        .tm
        .graph_mut()
        .try_get_state_mut(state_id)
        .expect("state exist");

    // If dragged, make the node follow the pointer
    if response.dragged() {
        state.inner_state.position = response.interact_pointer_pos().unwrap();
        state.inner_state.is_pinned = true;
    }

    if response.drag_started() {
        app.event.is_dragging = true
    }
    if response.drag_stopped() {
        app.event.is_dragging = false
    }
    Ok(())
}
