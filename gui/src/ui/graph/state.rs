use egui::{Align, Label, Rect, RichText, Sense, Stroke, Ui, vec2};

use crate::{
    App,
    error::RitmError,
    ui::{constant::Constant, font::Font, theme::Theme},
};

/// Display every state of the turing machine
pub fn show(app: &mut App, ui: &mut Ui) -> Result<(), RitmError> {
    // This line copy every keys of the hasmap to avoid borrowing the struct App that we need in each call.
    let keys: Vec<usize> = app
        .turing
        .tm
        .graph_ref()
        .get_state_hashmap()
        .keys()
        // .filter(|f| app.graph.drag_transition.is_none_or(|(s, _)| s != **f))
        .copied()
        .collect();
    for i in keys {
        draw_node(app, ui, i)?;
    }
    Ok(())
}

/// Draw a single state
pub fn draw_node(app: &mut App, ui: &mut Ui, state_id: usize) -> Result<(), RitmError> {
    // Get the state information
    let state = app
        .turing
        .tm
        .graph_mut()
        .get_state(state_id)
        .expect("state exist");

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
            Stroke::new(4.0, app.theme.selection)
        } else if app.turing.current_step.get_state_pointer() == state_id {
            Stroke::new(4.0, app.theme.highlight)
        } else {
            Stroke::new(2.0, app.theme.border)
        },
    );

    let name = RichText::new(state.get_name())
        .font(Font::default_big())
        .color(Theme::constrast_color(state.inner_state.color));

    let label = Label::new(name).wrap().halign(Align::Center);

    // Draw the label inside the node, without overflow
    ui.put(rect, label);

    // Listen for click and drag event on the node
    let response = ui.allocate_rect(
        rect,
        if app
            .graph
            .drag_transition
            .is_some_and(|(f, _)| f == state_id)
        {
            Sense::CLICK
        } else {
            Sense::click_and_drag()
        },
    );

    if response.clicked() {
        if app.edit.is_adding_transition {
            app.turing.add_transition(
                app.graph.selected_state().expect("state selected"),
                state_id,
            )?;
            app.edit.is_adding_transition &= !app.settings.reset_after_action;
            app.edit.is_adding_state = false;
        } else {
            app.graph.select_state(state_id);
            app.edit.is_adding_state = false;
        }
    }

    // Reborrow the state as a mut this time
    let state = app
        .turing
        .tm
        .graph_mut()
        .try_get_state_mut(state_id)
        .expect("state exist");

    if let Some((s, _)) = app.graph.drag_transition
        && response.contains_pointer()
    {
        app.graph.drag_transition = Some((s, Some(state_id)));
    }

    if app.graph.drag_transition.is_none()
        && response.is_pointer_button_down_on()
        && !response.dragged()
    {
        let time = ui.input(|r| r.time);
        let time_down = time - ui.input(|r| r.pointer.press_start_time()).unwrap_or(time);
        if time_down
            > ui.ctx()
                .options(|r| r.input_options.max_click_duration - 0.3)
        {
            app.graph.drag_transition = Some((state_id, Some(state_id)));
        }
        ui.ctx().request_repaint();
    }

    // If dragged, make the node follow the pointer
    if response.dragged() {
        state.inner_state.position = response
            .interact_pointer_pos()
            .expect("Pointer should exist");
        state.inner_state.is_pinned = true;
    }

    if response.drag_started() {
        app.graph.drag_transition = None;
        app.graph.is_dragging = true
    }
    if response.drag_stopped() {
        app.graph.is_dragging = false
    }
    Ok(())
}
