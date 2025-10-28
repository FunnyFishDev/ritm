use std::{
    char,
    fmt::{Debug, Display},
};

use thiserror::Error;

use crate::turing_transition::TuringTransition;

#[derive(Debug, Error)]
pub enum TuringStateError {
    #[error(
        "A transition cannot be added due to the number of tapes it affects (expected {expected} but got {received})"
    )]
    IncompatibleTransitionError {
        /// Number of writting tapes expected
        expected: usize,
        /// Numbers of writting tapes got
        received: usize,
    },
    #[error(
        "Trying to access an out of range transition (tried to access {accessed_index} but has only {states_len} indexes)"
    )]
    OutOfRangeTransitionError {
        accessed_index: usize,
        states_len: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
/// Represents the different types of states that can be found inside a turing machine graph
pub enum TuringStateType {
    /// A normal state, has no special effect.
    Normal,
    /// Accepts the given input.
    Accepting,
    /// Immediatly rejects the given input.
    Rejecting,
}

impl Display for TuringStateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TuringStateType::Normal => "Normal",
                TuringStateType::Accepting => "Accepting",
                TuringStateType::Rejecting => "Rejecting",
            }
        )
    }
}

#[derive(Debug, Clone)]
/// Represents a state of a turing machine
pub struct TuringState {
    /// Represents if the state is a final state or not
    pub state_type: TuringStateType,
    /// The vector containing all the transitions to the neighboring states
    pub transitions: Vec<TuringTransition>,
    /// The name of this state
    pub name: String,
}

impl TuringState {
    /// Creates a new [TuringState]
    pub fn new(state_type: TuringStateType, name: impl Into<String>) -> Self {
        Self {
            state_type,
            transitions: vec![],
            name: name.into(),
        }
    }

    /// Changes the name of a [TuringState]
    pub fn rename(&mut self, name: &str) {
        self.name = name.to_string();
    }

    /// Adds a new transition to the state
    pub fn add_transition(&mut self, transition: TuringTransition) -> Result<(), TuringStateError> {
        // Check that the number of tapes from a transition is the same for all added transitions
        if !self.transitions.is_empty()
            && self
                .transitions
                .first()
                .unwrap()
                .get_number_of_affected_tapes()
                != transition.get_number_of_affected_tapes()
        {
            return Err(TuringStateError::IncompatibleTransitionError {
                expected: self
                    .transitions
                    .first()
                    .unwrap()
                    .get_number_of_affected_tapes(),
                received: transition.get_number_of_affected_tapes(),
            });
        }

        self.transitions.push(transition);
        Ok(())
    }

    /// Removes the transition ***at*** the given index and returns it if it was correctly returned
    pub fn remove_transition_with_index(
        &mut self,
        transition_index: usize,
    ) -> Result<TuringTransition, TuringStateError> {
        if self.transitions.len() <= transition_index {
            return Err(TuringStateError::OutOfRangeTransitionError {
                accessed_index: transition_index,
                states_len: self.transitions.len(),
            });
        }
        Ok(self.transitions.remove(transition_index))
    }

    /// Removes all the transitions matching the given parameter. Beware that the `index_to_state` field will also be part of the evaluation.
    ///
    /// If the transition wasn't part of this state, nothing will happen.
    pub fn remove_transition(&mut self, transition: &TuringTransition) {
        let mut res = vec![];

        for t in &self.transitions {
            if t != transition || t.index_to_state != transition.index_to_state {
                res.push(t.clone());
            }
        }

        self.transitions = res;
    }

    /// Removes all the transitions from this state ***that are pointing*** at the given index
    pub fn remove_transitions(&mut self, to_index: usize) {
        let mut transitions = vec![];
        for t in &self.transitions {
            if let Some(index_to_state) = t.index_to_state {
                // If it is pointing at the given index, we remove it
                if index_to_state == to_index {
                    continue;
                }
            }
            transitions.push(t.clone());
        }
        self.transitions = transitions;
    }

    /// Updates the transition index to a new one
    pub fn update_transitions(&mut self, to_index_curr: usize, to_index_new: usize) {
        for t in &mut self.transitions {
            if let Some(index_to_state) = t.index_to_state {
                // If it was pointing to the old index, update it
                if index_to_state == to_index_curr {
                    t.index_to_state = Some(to_index_new);
                    // println!("changing it : from {} to {}", index_to_state, to_index_new);
                }
            }
        }
    }

    /// Checks for all transitions that can be taken when reading a char in this state
    pub fn get_valid_transitions(&self, chars_read: &Vec<char>) -> Vec<&TuringTransition> {
        let mut res = vec![];
        for t in &self.transitions {
            if chars_read.eq(&t.chars_read) {
                res.push(t);
            }
        }
        res
    }

    /// Checks for all the indexes of the transitions that can be taken when reading a char in this state
    pub fn get_valid_transitions_indexes(&self, chars_read: &Vec<char>) -> Vec<usize> {
        let mut res = vec![];
        for i in 0..self.transitions.len() {
            let t = &self.transitions[i];
            if chars_read.eq(&t.chars_read) {
                res.push(i);
            }
        }
        res
    }

    /// Gets all the transitions that can be taken to reach the given index.
    pub fn get_transitions_to(&self, to_index: usize) -> Vec<&TuringTransition> {
        let mut res = vec![];

        for t in &self.transitions {
            if let Some(i) = t.index_to_state
                && i == to_index
            {
                res.push(t);
            }
        }

        res
    }
}

impl Display for TuringState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}: {})", self.name, self.state_type)
    }
}

impl PartialEq for TuringState {
    fn eq(&self, other: &Self) -> bool {
        self.state_type == other.state_type
            && self.transitions == other.transitions
            && self.name == other.name
    }
}
