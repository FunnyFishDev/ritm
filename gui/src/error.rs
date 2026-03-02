use std::fmt::Display;

use egui::{Context, Id, Label, Modal, RichText, vec2};

use crate::{App, turing::TransitionId, ui::font::Font};

#[derive(Debug, PartialEq)]
pub enum RitmError {
    GuiError(GuiError),
    CoreError(String),
}

#[derive(Debug, PartialEq)]
pub enum GuiError {
    InvalidApplicationState,
    GraphError { error: String },
    CodeError { error: String },
    NoTransitionSelected,
    NoStateSelected,
    InvalidTransition { reason: String },
    TransitionAlreadyExist { transition_id: TransitionId },
    StateAlreadyExist { state_id: usize, name: String },
    SyntaxError { error: String },
    NoTransitionEditing,
    NoStateEditing,
    FileError { error: String },
    InvalidInput { input: String },
    NoStep,
    Default { error: String },
}

impl Display for RitmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            Self::GuiError(s) => s.to_string(),
            Self::CoreError(s) => s.to_string(),
        };
        writeln!(f, "{}", error)
    }
}

impl Display for GuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            Self::InvalidApplicationState => "Application state invalid !".to_string(),
            Self::GraphError { error } => format!("Error in the graph : {}", error),
            Self::CodeError { error } => format!("Error in code : {}", error),
            Self::NoTransitionSelected => "No transition selected".to_string(),
            Self::NoStateSelected => "No state selected".to_string(),
            Self::InvalidTransition { reason } => format!("Invalid transition : {}", reason),
            Self::TransitionAlreadyExist { transition_id } => format!(
                "Transition {} between state {} and {} already exist",
                transition_id.id, transition_id.source_id, transition_id.target_id
            ),
            Self::StateAlreadyExist { state_id, name } => {
                format!("State {} with id {} already exist", name, state_id)
            }
            Self::SyntaxError { error } => format!("Syntax error : {}", error),
            Self::NoTransitionEditing => "No transition are being edited".to_string(),
            Self::NoStateEditing => "No state are being edited".to_string(),
            Self::FileError { error } => format!("Could not load file: {}", error),
            Self::InvalidInput { input } => format!("{} is an invalid input", input),
            Self::Default { error } => format!("Error : {}", error),
            Self::NoStep => "There is no next step !".to_string(),
        };
        writeln!(f, "{}", error)
    }
}

pub fn show(ctx: &Context, app: &mut App) {
    let Some(error) = app.error.front() else {
        return;
    };

    let error = error.to_string();
    let popup_size = vec2(300.0, 300.0);

    Modal::new(Id::new("error")).show(ctx, |ui| {
        ui.set_max_size(popup_size);
        ui.set_min_size(vec2(popup_size.x, 0.0));

        ui.vertical_centered_justified(|ui| {
            ui.add(
                Label::new(RichText::new(error).font(Font::default_medium()))
                    .halign(egui::Align::Center)
                    .selectable(true),
            );
            if ui
                .button(RichText::new("close").font(Font::default_medium()))
                .clicked()
            {
                app.error.pop_front();
            }
        });
    });
}
