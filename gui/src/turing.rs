use egui::{Color32, Pos2, accesskit::Invalid};
use rand::random_range;
use ritm_core::turing_transition::{TuringDirection, TuringTransition};

use crate::App;

/// State visual representation
#[derive(PartialEq, Debug)]
pub struct State {
    pub id: usize,
    pub name: String,
    pub position: Pos2,
    pub is_pinned: bool,
    pub color: Color32,
    pub transitions: Vec<Transition>,
}

/// Transition are identified by the pair (source_state_id, own_id)
pub type TransitionId = (usize, usize);

/// Transition visual representation
#[derive(Default, PartialEq, Debug)]
pub struct Transition {
    pub text: String,
    pub id: usize,
    pub parent_id: usize,
    pub target_id: usize,
    pub unique_id: TransitionId,
}

/// Copy of the [`TuringTransition`] but with string to allow empty char
#[derive(Clone, PartialEq, Debug)]
pub struct TuringTransitionString {
    pub chars_read: Vec<String>,
    pub move_read: TuringDirection,
    pub chars_write: Vec<(String, TuringDirection)>,
    pub index_to_state: Option<usize>,
}

#[derive(Debug)]
pub struct TransitionEdit {
    edit: TuringTransitionString,
    base: TuringTransitionString,
    has_changed: bool,
}

impl TransitionEdit {
    pub fn from(ttmr: &TuringTransition) -> Self {
        let tts = TuringTransitionString {
            chars_read: ttmr
                .chars_read
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>(),
            move_read: ttmr.move_read.clone(),
            chars_write: ttmr
                .chars_write
                .iter()
                .map(|(char, dir)| (char.to_string(), dir.clone()))
                .collect::<Vec<(String, TuringDirection)>>(),
            index_to_state: ttmr.index_to_state,
        };

        Self {
            edit: tts.clone(),
            base: tts,
            has_changed: false,
        }
    }

    pub fn to(&self) -> Result<TuringTransition, Invalid> {
        if self.edit.chars_read.iter().any(|string| string.is_empty())
            || self
                .edit
                .chars_write
                .iter()
                .any(|(string, _)| string.is_empty())
        {
            return Err(Invalid::True);
        }

        Ok(TuringTransition {
            chars_read: self
                .edit
                .chars_read
                .iter()
                .map(|string| string.chars().next().unwrap())
                .collect::<Vec<char>>(),
            move_read: self.edit.move_read.clone(),
            chars_write: self
                .edit
                .chars_write
                .iter()
                .map(|(string, dir)| (string.chars().next().unwrap(), dir.clone()))
                .collect::<Vec<(char, TuringDirection)>>(),
            index_to_state: self.edit.index_to_state,
        })
    }

    /// The transition information editable
    pub fn get_edit(&mut self) -> &mut TuringTransitionString {
        &mut self.edit
    }

    /// Check if the transition information changed
    pub fn has_changed(&mut self) -> bool {
        if self.has_changed {
            true
        } else {
            self.has_changed = self.edit != self.base;
            self.has_changed
        }
    }

    /// Undo all changes made to the editable transition struct
    pub fn undo(&mut self) {
        self.edit = self.base.clone();
        self.has_changed = false;
    }
}

impl State {
    pub fn new_at_pos(id: usize, name: String, position: Pos2) -> Self {
        State {
            id,
            name,
            position,
            is_pinned: true,
            color: Color32::WHITE,
            transitions: vec![],
        }
    }

    pub fn new(id: usize, name: String) -> Self {
        State {
            id,
            name,
            position: Pos2::new(random_range(0.0..1.0), random_range(0.0..1.0)),
            is_pinned: true,
            color: Color32::WHITE,
            transitions: vec![],
        }
    }

    pub fn get(app: &App, id: usize) -> &Self {
        app.states.get(&id).unwrap()
    }

    pub fn get_mut(app: &mut App, id: usize) -> &mut Self {
        app.states.get_mut(&id).unwrap()
    }
}

impl Transition {
    pub fn new(text: String, id: usize, parent_id: usize, target_id: usize) -> Self {
        Transition {
            text,
            id,
            parent_id,
            target_id,
            unique_id: (parent_id, id),
        }
    }

    pub fn get(app: &App, identifier: (usize, usize)) -> &Self {
        State::get(app, identifier.0)
            .transitions
            .get(identifier.1)
            .unwrap()
    }

    pub fn get_mut(app: &mut App, identifier: (usize, usize)) -> &mut Self {
        State::get_mut(app, identifier.0)
            .transitions
            .get_mut(identifier.1)
            .unwrap()
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            id: 0,
            color: Color32::WHITE,
            name: "N/a".to_string(),
            transitions: vec![],
            position: Pos2::new(random_range(0.0..1.0), random_range(0.0..1.0)),
            is_pinned: true,
        }
    }
}
