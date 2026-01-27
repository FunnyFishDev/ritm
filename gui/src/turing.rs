use std::{collections::BTreeSet, process::exit};

use egui::{Color32, Pos2, vec2};
use rand::{random, random_range};
use ritm_core::{
    turing_graph::{TuringGraph, TuringState, TuringStateType, TuringStateWrapper},
    turing_machine::{Mode, TuringExecutionSteps, TuringMachine},
    turing_transition::{
        TuringDirection, TuringTransition, TuringTransitionInfo, TuringTransitionWrapper,
    },
};

use crate::error::RitmError;

pub type TransitionWrapper = TuringTransitionWrapper<Transition>;
pub type StateWrapper = TuringStateWrapper<State>;

pub struct Turing {
    pub tm: TuringMachine<State, Transition>,
    pub current_step: TuringExecutionSteps,
    pub accepted: Option<bool>,
    pub transition_edit: Option<((usize, usize), Vec<TransitionEdit>)>,
    pub state_edit: Option<StateEdit>,
}

impl Default for Turing {
    fn default() -> Self {
        let graph: TuringGraph<State, Transition> =
            TuringGraph::new(1, true).expect("Turing graph creation fail");
        let mode = Mode::StopFirstReject;
        let mut tm =
            TuringMachine::new(graph, "".to_string(), mode).expect("Turing machine creation fail");
        let step = tm.into_iter().next().expect("Initial step creation fail");
        Self {
            tm,
            accepted: None,
            current_step: step,
            state_edit: None,
            transition_edit: None,
        }
    }
}

impl Turing {
    /// Return a turing machine using the graph passed
    pub fn new_graph(graph: TuringGraph<State, Transition>) -> Self {
        let mode = Mode::StopFirstReject;
        let mut tm =
            TuringMachine::new(graph, "".to_string(), mode).expect("Turing machine creation fail");
        let step = tm.into_iter().next().expect("Initial step creation fail");
        Self {
            tm,
            accepted: None,
            current_step: step,
            state_edit: None,
            transition_edit: None,
        }
    }

    /// Fetch the next step of the turing machine.
    ///
    /// If there is no next step, then we check
    /// the last step state type to accept or reject.
    pub fn next_step(&mut self) {
        // Ignore if the machine already reached the last step
        if self.accepted.is_some() {
            return;
        }

        match self.tm.into_iter().next() {
            Some(step) => self.current_step = step,
            None => {
                // If there is no next step, then we check the last step state type to accept or reject.
                self.accepted = Some(
                    self.current_step.get_current_state().get_type() == TuringStateType::Accepting,
                );
            }
        }
    }

    /// Reset the machine to its initial state
    pub fn reset(&mut self) {
        self.accepted = None;
        self.transition_edit = None;
        self.state_edit = None;
        self.tm.reset();
        self.next_step();
    }

    /// The state is saved with the core assigned id
    pub fn add_state(&mut self, name: String) -> usize {
        self.tm.graph_mut().add_state(name, TuringStateType::Normal)
    }

    /// The state is saved with the core assigned id
    pub fn add_state_with_pos(&mut self, name: String, position: Pos2) -> usize {
        let state_id = self.tm.graph_mut().add_state(name, TuringStateType::Normal);
        self.tm
            .graph_mut()
            .try_get_state_mut(state_id)
            .expect("state has been created")
            .inner_state
            .position = position;
        state_id
    }

    pub fn add_transition(&mut self, source_id: usize, target_id: usize) -> Result<(), RitmError> {
        self.tm
            .graph_mut()
            .append_default_transition(source_id, None, target_id)
            .map_err(|e| RitmError::CoreError(e.to_string()))
    }

    pub fn get_state(&self, id: usize) -> Result<&StateWrapper, RitmError> {
        self.tm
            .graph_ref()
            .get_state(id)
            .ok_or(RitmError::CoreError("State not found".to_string()))
    }

    pub fn get_state_mut(&mut self, id: usize) -> &mut StateWrapper {
        self.tm
            .graph_mut()
            .try_get_state_mut(id)
            .map_err(|e| RitmError::CoreError(e.to_string()))
            .expect("state exist")
    }

    /// Remove the state if it exist. If not an error [`RitmError::CoreError`]
    /// is returned
    pub fn remove_state(&mut self, state_id: usize) -> Result<(), RitmError> {
        self.tm
            .graph_mut()
            .remove_state(state_id)
            .map_err(|e| RitmError::CoreError(e.to_string()))
    }

    /// Remove the transition if it exist. If not an error [`RitmError::CoreError`]
    /// is returned
    pub fn remove_transition(&mut self, transition_id: TransitionId) -> Result<(), RitmError> {
        let TransitionId {
            source_id,
            id,
            target_id,
        } = transition_id;

        self.tm
            .graph_mut()
            .remove_transition((source_id, id, target_id))
            .map_err(|e| RitmError::CoreError(e.to_string()))
    }

    /// Remove the transition if it exist. If not an error [`RitmError::CoreError`]
    /// is returned
    pub fn remove_transitions(
        &mut self,
        source_id: usize,
        target_id: usize,
    ) -> Result<(), RitmError> {
        self.tm
            .graph_mut()
            .remove_transitions(source_id, target_id)
            .map_err(|e| RitmError::CoreError(e.to_string()))
    }

    /// Fetch the transition wrapper if it exist. If not an error [`RitmError::CoreError`]
    /// is returned
    pub fn get_transition(
        &self,
        transition_id: impl Into<TransitionId>,
    ) -> Result<&TuringTransitionWrapper<Transition>, RitmError> {
        let TransitionId {
            source_id,
            id,
            target_id,
        } = transition_id.into();

        let res = self
            .tm
            .graph_ref()
            .get_transitions(source_id, target_id)
            .map_err(|e| RitmError::CoreError(e.to_string()));

        match res {
            Ok(res) => match res {
                Some(transitions) => {
                    if transitions.len() <= id {
                        Err(RitmError::CoreError(format!("Transition {} not found", id)))
                    } else {
                        Ok(&transitions[id])
                    }
                }
                None => Err(RitmError::CoreError(format!(
                    "No transition found between states {} and {}",
                    source_id, target_id
                ))),
            },
            Err(err) => Err(err),
        }
    }

    pub fn get_transitions(
        &self,
        source_id: usize,
        target_id: usize,
    ) -> Result<&Vec<TuringTransitionWrapper<Transition>>, RitmError> {
        self.tm
            .graph_ref()
            .get_transitions(source_id, target_id)
            .map(|v| {
                if let Some(transitions) = v {
                    Ok(transitions)
                } else {
                    Err(RitmError::CoreError("no".to_string()))
                }
            }) // TODO: replace the expect with a new RitmError
            .expect("no")
    }

    /// The mode currently used
    pub fn get_mode(&self) -> &Mode {
        self.tm.get_mode()
    }

    /// TODO: changing the mode should reset the machine ?
    pub fn set_mode(&mut self, mode: &Mode) {
        self.tm.set_mode(mode);
    }

    /// Apply transition changes if correct
    pub fn apply_transition_change(&mut self) -> Result<(), RitmError> {
        let Some(((source, target), transitions_edit)) = self.transition_edit.as_ref() else {
            return Err(RitmError::GuiError(
                "No transition is being edited".to_string(),
            ));
        };

        self.tm
            .graph_mut()
            .remove_transitions(source, target)
            .map_err(|e| RitmError::CoreError(e.to_string()))?;

        let transitions_edit = transitions_edit
            .iter()
            .map(|f| f.to())
            .collect::<Vec<Result<TransitionWrapper, RitmError>>>();

        for transition in transitions_edit {
            self.tm
                .graph_mut()
                .append_transition(source, transition?.clone(), target)
                .map_err(|e| RitmError::CoreError(e.to_string()))?;
        }
        Ok(())
    }

    /// Update the position of each state so the graph
    /// is displayed as a layered graph
    pub fn layer_graph(&mut self) {
        let mut state_list: BTreeSet<usize> = BTreeSet::new();
        let mut layer_state: Vec<usize> = vec![];

        for (i, state) in self.tm.graph_ref().get_states().iter().enumerate() {
            let index = i;

            //before it was || state.transitions.is_empty() but check for transition empty instead as a fix
            if state.get_type() == TuringStateType::Accepting
                || self.tm.graph_ref().get_transitions_hashmap().is_empty()
            {
                layer_state.push(index);
            } else {
                state_list.insert(index);
            }
        }

        let mut j = 0.0;
        while !(state_list.is_empty() && layer_state.is_empty()) {
            let layer_count = layer_state.len() as f32 - 1.0;
            for (i, state_id) in layer_state.iter().enumerate() {
                self.tm
                    .graph_mut()
                    .try_get_state_mut(state_id)
                    .expect("state shoud exist")
                    .inner_state
                    .position = Pos2::new(
                    (j - (layer_count / 2.0 - i as f32)) * -200.0,
                    (j + (layer_count / 2.0 - i as f32)) * -200.0,
                ) + vec2(random(), random());
            }

            let next_layer_state: Vec<usize> = state_list
                .iter()
                .filter(|state_id| {
                    layer_state.iter().any(|layer_state_id| {
                        self.tm
                            .graph_ref()
                            .get_transitions_hashmap()
                            .contains_key(&(**state_id, *layer_state_id))
                    })
                })
                .copied()
                .collect();

            state_list.retain(|k| !next_layer_state.contains(k));

            layer_state = next_layer_state;

            j += 1.0;

            if j > 5. {
                exit(0)
            }
        }
    }

    /// Unpin all states
    pub fn unpin(&mut self) {
        self.tm
            .graph_mut()
            .get_states_mut()
            .iter_mut()
            .for_each(|s| s.inner_state.is_pinned = false);
    }

    /// Pin all states
    pub fn pin(&mut self) {
        self.tm
            .graph_mut()
            .get_states_mut()
            .iter_mut()
            .for_each(|s| s.inner_state.is_pinned = true);
    }

    /// Setup transition edit
    pub fn prepare_transition_edit(&mut self, source: usize, target: usize) {
        let transitions_edit: Vec<TransitionEdit> = self
            .tm
            .graph_mut()
            .get_transitions(source, target)
            .expect("Should have transitions")
            .expect("Should REALLY have transitions")
            .iter()
            .map(TransitionEdit::from)
            .collect();

        self.transition_edit = Some(((source, target), transitions_edit))
    }

    /// TODO: add error management
    pub fn set_word(&mut self, word: &String) -> Result<(), RitmError> {
        self.tm
            .reset_word(word)
            .map_err(|e| RitmError::CoreError(e.to_string()))?;
        self.reset();
        Ok(())
    }

    pub fn cancel_transition_change(&mut self) {
        self.transition_edit = None;
    }

    pub fn get_transition_edit(&self) -> Result<&((usize, usize), Vec<TransitionEdit>), RitmError> {
        self.transition_edit
            .as_ref()
            .ok_or(RitmError::GuiError("No transition found".to_string()))
    }

    pub fn get_transition_edit_mut(
        &mut self,
    ) -> Result<&mut ((usize, usize), Vec<TransitionEdit>), RitmError> {
        self.transition_edit
            .as_mut()
            .ok_or(RitmError::GuiError("No transition found".to_string()))
    }
}

/// State visual representation
#[derive(PartialEq, Debug, Clone)]
pub struct State {
    pub position: Pos2,
    pub is_pinned: bool,
    pub color: Color32,
}

impl TuringState for State {
    fn new_init() -> Self {
        State::default()
    }

    fn new_accepting() -> Self {
        State::default()
    }
}

/// Transition are identified by the trio (source_id, id, target_id)
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct TransitionId {
    pub source_id: usize,
    pub id: usize,
    pub target_id: usize,
}

impl From<(usize, usize, usize)> for TransitionId {
    fn from(value: (usize, usize, usize)) -> Self {
        Self {
            source_id: value.0,
            id: value.1,
            target_id: value.2,
        }
    }
}

/// Transition visual representation
#[derive(Default, PartialEq, Debug, Clone)]
pub struct Transition {}

impl TuringTransition for Transition {}

#[derive(Clone, PartialEq, Debug)]
pub struct TransitionWrapperCopy {
    pub inner_transition: Transition,
    /// The chars that have to be read in order apply the rest of the transition : `a_0,..., a_{n-1}`
    pub chars_read: Vec<String>,
    /// The move to take after writing/reading the character : `D_0`
    pub move_read: TuringDirection,
    /// The character to replace the character just read : `(b_1, D_1),..., (b_{n-1}, D_{n-1})`
    pub chars_write: Vec<(String, TuringDirection)>,
}

impl From<&TransitionWrapper> for TransitionWrapperCopy {
    fn from(value: &TransitionWrapper) -> Self {
        Self {
            inner_transition: value.inner_transition.clone(),
            chars_read: value
                .info
                .chars_read
                .iter()
                .map(|e| e.to_string())
                .collect(),
            chars_write: value
                .info
                .chars_write
                .iter()
                .map(|e| (e.0.to_string(), e.1.clone()))
                .collect(),
            move_read: value.info.move_read.clone(),
        }
    }
}

impl TransitionWrapperCopy {
    pub fn try_to(&self) -> Result<TransitionWrapper, RitmError> {
        if self.chars_read.iter().any(|string| string.is_empty())
            || self.chars_write.iter().any(|(string, _)| string.is_empty())
        {
            return Err(RitmError::GuiError(
                "Empty char present in the transition".to_string(),
            ));
        }
        let chars_read = self
            .chars_read
            .iter()
            .map(|e| e.chars().nth(0).expect("Shouldn't have empty char"))
            .collect();
        let chars_write = self
            .chars_write
            .iter()
            .map(|(string, td)| {
                (
                    string.chars().nth(0).expect("Shouldn't have empty char"),
                    td.clone(),
                )
            })
            .collect();
        Ok(TransitionWrapper {
            // TODO: use new new() method
            info: TuringTransitionInfo::new(chars_read, self.move_read.clone(), chars_write)
                .map_err(|e| RitmError::CoreError(e.to_string()))?,

            inner_transition: self.inner_transition.clone(),
        })
    }
}

#[derive(Debug)]
pub struct TransitionEdit {
    base: TransitionWrapperCopy,
    edit: TransitionWrapperCopy,
    has_changed: bool,
}

impl TransitionEdit {
    pub fn from(ttmr: &TransitionWrapper) -> Self {
        Self {
            base: ttmr.into(),
            edit: ttmr.into(),
            has_changed: false,
        }
    }

    pub fn to(&self) -> Result<TransitionWrapper, RitmError> {
        if self.edit.chars_read.iter().any(|string| string.is_empty())
            || self
                .edit
                .chars_write
                .iter()
                .any(|(string, _)| string.is_empty())
        {
            return Err(RitmError::GuiError(
                "Empty char present in the transition".to_string(),
            ));
        }

        self.edit.try_to()
    }

    /// The transition information editable
    pub fn get_edit(&mut self) -> &mut TransitionWrapperCopy {
        &mut self.edit
    }

    /// Check if the transition information changed
    pub fn has_changed(&self) -> bool {
        self.edit != self.base
    }

    /// Undo all changes made to the editable transition struct
    pub fn undo(&mut self) {
        self.edit = self.base.clone();
        self.has_changed = false;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateWrapperCopy {
    pub name: String,
    pub state_type: TuringStateType,
    pub state: State,
}

#[derive(Debug)]
pub struct StateEdit {
    base: StateWrapperCopy,
    edit: StateWrapperCopy,
    has_changed: bool,
}

impl StateEdit {
    pub fn from(ttmr: &StateWrapper) -> Self {
        let state_wrapper = StateWrapperCopy {
            name: ttmr.get_name().to_string(),
            state_type: ttmr.get_type(),
            state: ttmr.inner_state.clone(),
        };
        Self {
            base: state_wrapper.clone(),
            edit: state_wrapper,
            has_changed: false,
        }
    }

    pub fn empty() -> Self {
        let state_wrapper = StateWrapperCopy {
            name: "".to_string(),
            state_type: TuringStateType::Normal,
            state: State {
                position: Pos2::ZERO,
                is_pinned: false,
                color: Color32::WHITE,
            },
        };
        Self {
            base: state_wrapper.clone(),
            edit: state_wrapper,
            has_changed: false,
        }
    }

    pub fn to(&self) -> &StateWrapperCopy {
        &self.edit
    }

    /// The transition information editable
    pub fn get_edit(&mut self) -> &mut StateWrapperCopy {
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
    pub fn at_pos(position: Pos2) -> Self {
        State {
            position,
            is_pinned: true,
            color: Color32::WHITE,
        }
    }
}

impl Transition {
    pub fn new() -> Self {
        Transition {}
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            color: Color32::WHITE,
            position: Pos2::new(random_range(0.0..1.0), random_range(0.0..1.0)),
            is_pinned: true,
        }
    }
}
