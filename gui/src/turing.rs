use std::collections::BTreeSet;

use egui::{Color32, Pos2, Vec2, vec2};
use rand::{random, random_range};
use ritm_core::{
    turing_graph::{
        TuringGraph, TuringGraphError, TuringState, TuringStateType, TuringStateWrapper,
    },
    turing_machine::{Mode, TuringExecutionSteps, TuringMachine},
    turing_transition::{TuringTransition, TuringTransitionWrapper},
};

use crate::error::{GuiError, RitmError};

pub type TransitionWrapper = TuringTransitionWrapper<Transition>;
pub type StateWrapper = TuringStateWrapper<State>;
pub type TransitionsEdit = ((usize, usize), Vec<(TransitionEdit, Option<String>)>);

pub struct Turing {
    pub tm: TuringMachine<State, Transition>,
    pub current_step: TuringExecutionSteps,
    pub accepted: Option<bool>,
    pub transition_edit: Option<TransitionsEdit>,
    pub state_edit: Option<StateEdit>,
}

impl Default for Turing {
    fn default() -> Self {
        let graph: TuringGraph<State, Transition> =
            TuringGraph::new(0, true).expect("Turing graph creation fail");
        let mode = Mode::SaveAll;
        let mut tm =
            TuringMachine::new(graph, "".to_string(), mode).expect("Turing machine creation fail");
        let step = tm.into_iter().next().expect("Initial step creation fail");
        let mut turing = Self {
            tm,
            accepted: None,
            current_step: step.clone(),
            state_edit: None,
            transition_edit: None,
        };
        turing.layer_graph();
        turing
    }
}

/// Dedicated method
impl Turing {
    /// Return a turing machine using the graph passed
    pub fn new_graph(graph: TuringGraph<State, Transition>) -> Result<Self, RitmError> {
        let mode = Mode::SaveAll;
        let mut tm = TuringMachine::new(graph, "".to_string(), mode)
            .map_err(|e| RitmError::CoreError(e.to_string()))?;
        let step = tm
            .into_iter()
            .next()
            .ok_or(RitmError::GuiError(GuiError::NoStep))?;
        Ok(Self {
            tm,
            accepted: None,
            current_step: step,
            state_edit: None,
            transition_edit: None,
        })
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

    /// Insert the word as the new input and reset the turing machine
    pub fn set_word(&mut self, word: &String) -> Result<(), RitmError> {
        self.tm
            .reset_word(word)
            .map_err(|e| RitmError::CoreError(e.to_string()))?;
        self.reset();
        Ok(())
    }

    /// The mode currently used
    pub fn get_mode(&self) -> &Mode {
        self.tm.get_mode()
    }

    /// TODO: changing the mode should reset the machine ?
    pub fn set_mode(&mut self, mode: &Mode) {
        self.tm.set_mode(mode);
        self.reset();
    }
}

/// Access method
impl Turing {
    /// If the state was not already added, adds it to the graph with the core assigned id
    pub fn add_state(&mut self, name: String) -> usize {
        match self
            .tm
            .graph_mut()
            .try_add_state(name, TuringStateType::Normal)
        {
            Ok(id) => id,
            Err(e) => match e {
                TuringGraphError::AlreadyPresentNameError { name: _, state } => state.get_id(),
                TuringGraphError::EmptyNameError => {
                    self.tm.graph_mut().add_next_state(TuringStateType::Normal)
                }
                _ => unreachable!("The other graph erros can never be reached"),
            },
        }
    }

    /// The state is saved with the core assigned id
    pub fn add_state_with_pos(&mut self, name: String, position: Pos2) -> Result<usize, RitmError> {
        let state_id = self
            .tm
            .graph_mut()
            .try_add_state(name, TuringStateType::Normal)
            .map_err(|e| RitmError::CoreError(e.to_string()))?;

        self.tm
            .graph_mut()
            .try_get_state_mut(state_id)
            .expect("state has been created")
            .inner_state
            .position = position;
        Ok(state_id)
    }

    /// Fetch a state by its id
    pub fn get_state(&self, id: usize) -> Result<&StateWrapper, RitmError> {
        self.tm
            .graph_ref()
            .get_state(id)
            .ok_or(RitmError::CoreError("State not found".to_string()))
    }

    /// Fetch a state by id, but mutable
    pub fn get_state_mut(&mut self, id: usize) -> Result<&mut StateWrapper, RitmError> {
        self.tm
            .graph_mut()
            .try_get_state_mut(id)
            .map_err(|e| RitmError::CoreError(e.to_string()))
    }

    /// Remove the state if it exist. If not return [`RitmError::CoreError`]
    pub fn remove_state(&mut self, state_id: usize) -> Result<(), RitmError> {
        self.tm
            .graph_mut()
            .remove_state(state_id)
            .map_err(|e| RitmError::CoreError(e.to_string()))
    }

    pub fn rename_state(&mut self, selected: usize, state_name: String) -> Result<(), RitmError> {
        self.tm
            .graph_mut()
            .rename_state(selected, state_name)
            .map_err(|e| RitmError::CoreError(e.to_string()))
    }

    /// Add a new transition between the source and the target
    pub fn add_default_transition(
        &mut self,
        source_id: usize,
        target_id: usize,
    ) -> Result<(), RitmError> {
        self.tm
            .graph_mut()
            .append_default_transition(source_id, None, target_id)
            .map_err(|e| RitmError::CoreError(e.to_string()))
    }

    /// Fetch the transition wrapper if it exist. If not return [`RitmError::CoreError`]
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

    /// Remove the transition if it exist. If not return [`RitmError::CoreError`]
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

    /// Remove all transitions between the source and target if they exist. If not return [`RitmError::CoreError`]
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

    /// Return an error if no transition exist
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
                    Err(RitmError::CoreError("No transition found".to_string()))
                }
            })
            .map_err(|e| RitmError::CoreError(e.to_string()))?
    }
}

/// Transition editing
impl Turing {
    /// Setup transition edit
    pub fn prepare_transition_edit(
        &mut self,
        source: usize,
        target: usize,
    ) -> Result<(), RitmError> {
        let transitions_edit: Vec<(TransitionEdit, Option<String>)> = self
            .tm
            .graph_mut()
            .get_transitions(source, target)
            .map_err(|e| RitmError::CoreError(e.to_string()))?
            .ok_or(RitmError::CoreError("No transitions found".to_string()))?
            .iter()
            .map(|e| (TransitionEdit::from(e), None))
            .collect();

        self.transition_edit = Some(((source, target), transitions_edit));
        Ok(())
    }

    /// Apply transition changes if correct
    pub fn apply_transition_change(&mut self) -> Result<(), RitmError> {
        let Some(((source, target), transitions_edit)) = self.transition_edit.as_mut() else {
            return Err(RitmError::GuiError(GuiError::InvalidApplicationState));
        };

        let mut new_transitions: Vec<Result<TuringTransitionWrapper<Transition>, RitmError>> =
            Vec::new();

        for (transition_edit, _) in transitions_edit {
            new_transitions.push(match transition_edit.to() {
                Ok(transition) => Ok(transition),
                Err(err) => Err(err),
            })
        }

        // Check if all transition can be written, aborting if not
        if new_transitions.iter().any(|f| f.is_err()) {
            let mut reason = String::new();
            for (i, trans) in new_transitions.into_iter().enumerate() {
                if let Err(err) = trans {
                    reason.push_str(format!("{i}:{err}").as_str());
                }
            }
            return Err(RitmError::GuiError(GuiError::InvalidTransition { reason }));
        }

        self.tm
            .graph_mut()
            .remove_transitions(*source, *target)
            .map_err(|e| RitmError::CoreError(e.to_string()))?;

        // Create new transition
        for transition in new_transitions {
            self.tm
                .graph_mut()
                .append_transition(*source, transition?.clone(), *target)
                .map_err(|e| RitmError::CoreError(e.to_string()))?;
        }
        Ok(())
    }

    pub fn cancel_transition_change(&mut self) {
        self.transition_edit = None;
    }

    pub fn get_transitions_edit(&self) -> Result<&TransitionsEdit, RitmError> {
        self.transition_edit
            .as_ref()
            .ok_or(RitmError::GuiError(GuiError::NoTransitionEditing))
    }

    /// Return an error if the transition edit has not been set
    pub fn get_transitions_edit_mut(&mut self) -> Result<&mut TransitionsEdit, RitmError> {
        self.transition_edit
            .as_mut()
            .ok_or(RitmError::GuiError(GuiError::NoTransitionEditing))
    }

    /// Return an error if the transition edit has not been set
    pub fn get_transition_edit_mut(
        &mut self,
        id: usize,
    ) -> Result<&mut TransitionWrapper, RitmError> {
        self.transition_edit
            .as_mut()
            .map(|(_, s)| s[id].0.get_edit())
            .ok_or(RitmError::GuiError(GuiError::NoTransitionEditing))
    }

    /// Prepare a state to be edited
    pub fn prepare_state_edit(&mut self, state_id: usize) -> Result<(), RitmError> {
        let state = self.get_state(state_id)?;
        self.state_edit = Some(StateEdit::from(state));
        Ok(())
    }

    /// Apply the current change to the state being edited
    pub fn apply_state_change(&mut self) -> Result<usize, RitmError> {
        let state_edit = self
            .state_edit
            .as_ref()
            .ok_or(RitmError::GuiError(GuiError::NoStateEditing))?;

        match state_edit.id {
            Some(state_id) => {
                let x = state_edit.to();
                self.rename_state(state_id, x.name.clone())?;
                Ok(state_id)
            }
            None => {
                let name = state_edit.to().name.clone();
                let state_copy = state_edit.edit.state.clone();
                let state_id = self.add_state(name);
                let state = self.get_state_mut(state_id).expect("should exist");
                state.inner_state = state_copy;
                Ok(state_id)
            }
        }
    }

    pub fn cancel_state_change(&mut self) {
        self.state_edit = None;
    }
    /// Update the position of each state so the graph
    /// is displayed as a layered graph
    pub fn layer_graph(&mut self) {
        let mut state_list: BTreeSet<usize> = BTreeSet::new();
        let mut layer_state: Vec<usize> = vec![];

        for (index, state) in self.tm.graph_ref().get_states().iter().enumerate() {
            if state.get_type() == TuringStateType::Accepting
                || self
                    .tm
                    .graph_ref()
                    .is_state_dead_end(index)
                    .expect("SHOULD HAVE STATE")
            {
                layer_state.push(index);
            } else {
                state_list.insert(index);
            }
        }

        let mut j = 0.0;
        let mut state_list_size = 0;
        while !(state_list.is_empty() && layer_state.is_empty()) {
            if !state_list.is_empty() && state_list.len() == state_list_size {
                layer_state.push(
                    state_list
                        .pop_first()
                        .expect("should have at least one element"),
                );
            }

            let layer_count = layer_state.len() as f32 - 1.0;
            for (i, state_id) in layer_state.iter().enumerate() {
                self.tm
                    .graph_mut()
                    .try_get_state_mut(state_id)
                    .expect("state shoud exist")
                    .inner_state
                    .position = Pos2::new(
                    (j - (layer_count / 2.0 - i as f32)) * -400.0,
                    (j + (layer_count / 2.0 - i as f32)) * -400.0,
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

            state_list_size = state_list.len();

            j += 1.0;
        }
    }

    /// Unpin a state
    pub fn unpin(&mut self, state_id: usize) -> Result<(), RitmError> {
        self.get_state_mut(state_id)?.inner_state.is_pinned = false;
        Ok(())
    }

    /// Unpin all states
    pub fn unpin_all(&mut self) {
        self.tm
            .graph_mut()
            .get_states_mut()
            .iter_mut()
            .for_each(|s| s.inner_state.is_pinned = false);
    }

    /// Pin a state
    pub fn pin(&mut self, state_id: usize) -> Result<(), RitmError> {
        self.get_state_mut(state_id)?.inner_state.is_pinned = true;
        Ok(())
    }

    /// Pin all states
    pub fn pin_all(&mut self) {
        self.tm
            .graph_mut()
            .get_states_mut()
            .iter_mut()
            .for_each(|s| s.inner_state.is_pinned = true);
    }

    pub fn graph_center(&self) -> Pos2 {
        self.tm
            .graph_ref()
            .get_states()
            .iter()
            .fold(Pos2::ZERO, |acc, e| acc + e.inner_state.position.to_vec2())
            / self.tm.graph_ref().get_state_hashmap().len() as f32
    }

    pub fn neighbors(&self, state_id: usize) -> Vec<usize> {
        let mut res = self
            .tm
            .graph_ref()
            .get_transitions_hashmap()
            .iter()
            .filter_map(|(k, _v)| {
                if k.0 == k.1 {
                    None
                } else if k.0 == state_id {
                    Some(k.1)
                } else if k.1 == state_id {
                    Some(k.0)
                } else {
                    None
                }
            })
            .collect::<Vec<usize>>();
        res.sort();
        res.dedup();
        res
    }

    pub fn best_vector(&self, state_id: usize) -> Result<Vec2, RitmError> {
        let mut transition_vector =
            self.neighbors(state_id)
                .iter()
                .try_fold(Vec2::ZERO, |acc, e| {
                    let target_position = self.get_state(*e)?.inner_state.position;
                    let source_position = self.get_state(state_id)?.inner_state.position;
                    Ok(acc + (target_position - source_position).normalized())
                })?;

        if transition_vector == Vec2::ZERO {
            transition_vector =
                self.get_state(state_id)?.inner_state.position - self.graph_center();
        }
        Ok(transition_vector)
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
        Self {
            color: Color32::LIGHT_BLUE,
            ..Default::default()
        }
    }

    fn new_accepting() -> Self {
        Self {
            color: Color32::LIGHT_GREEN,
            ..Default::default()
        }
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

#[derive(Debug, Clone)]
pub struct TransitionEdit {
    base: TransitionWrapper,
    edit: TransitionWrapper,
    has_changed: bool,
}

impl TransitionEdit {
    pub fn from(ttmr: &TransitionWrapper) -> Self {
        Self {
            base: ttmr.clone(),
            edit: ttmr.clone(),
            has_changed: false,
        }
    }

    pub fn to(&self) -> Result<TransitionWrapper, RitmError> {
        if self.edit.info.get_chars_read().contains(&'\0') {
            return Err(RitmError::GuiError(GuiError::InvalidTransition {
                reason: "Empty char present in the transition".to_string(),
            }));
        }

        Ok(self.edit.clone())
    }

    /// The transition information editable
    pub fn get_edit(&mut self) -> &mut TransitionWrapper {
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
    id: Option<usize>,
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
            id: Some(ttmr.get_id()),
            base: state_wrapper.clone(),
            edit: state_wrapper,
            has_changed: false,
        }
    }

    pub fn empty(next_id: usize) -> Self {
        let state_wrapper = StateWrapperCopy {
            name: next_id.to_string(),
            state_type: TuringStateType::Normal,
            state: State {
                position: Pos2::ZERO,
                is_pinned: false,
                color: Color32::WHITE,
            },
        };
        Self {
            id: None,
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
