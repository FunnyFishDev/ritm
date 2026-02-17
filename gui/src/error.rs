use std::fmt::Display;

use egui::{Context, Id, Label, LayerId, Popup, PopupAnchor, RichText, vec2};

use crate::{App, ui::font::Font};

#[derive(Debug, PartialEq)]
pub enum RitmError {
    GuiError(GuiError),
    CoreError(String),
}

#[derive(Debug, PartialEq)]
pub enum GuiError {
    InvalidState,
    GraphError(String),
    CodeError(String),
    NoTransitionSelected,
    NoStateSelected,
    InvalidTransition { reason: String },
    TransitionAlreadyExist,
    StateAlreadyExist,
    SyntaxError(String),
    NoTransitionEditing,
    NoStateEditing,
    FileError(String),
    InvalidInput(String),
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
            Self::InvalidState => "Application state invalid !".to_string(),
            Self::GraphError(err) => format!("Error in the graph : {}", err),
            Self::CodeError(err) => format!("Error in code : {}", err),
            Self::NoTransitionSelected => "No transition selected".to_string(),
            Self::NoStateSelected => "No state selected".to_string(),
            Self::InvalidTransition { reason } => format!("Invalid transition : {}", reason),
            Self::TransitionAlreadyExist => "Transition already exist".to_string(),
            Self::StateAlreadyExist => "State already exist".to_string(),
            Self::SyntaxError(err) => format!("Syntax error : {}", err),
            Self::NoTransitionEditing => "No transition are being edited".to_string(),
            Self::NoStateEditing => "No state are being edited".to_string(),
            Self::FileError(err) => format!("File error : {}", err),
            Self::InvalidInput(reason) => format!("Invalid input detected : {}", reason),
        };
        writeln!(f, "{}", error)
    }
}

pub fn error(ctx: Context, app: &mut App) {
    let Some(error) = &app.error else {
        return;
    };

    let error = error.to_string();
    let popup_size = vec2(300.0, 300.0);
    let rect = ctx.screen_rect().center() - popup_size / 2.0;
    Popup::new(
        Id::new("error"),
        ctx,
        PopupAnchor::Position(rect),
        LayerId::new(egui::Order::Debug, Id::new("error_layer")),
    )
    .show(|ui| {
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
                app.error = None;
            }
        });
    });
}
