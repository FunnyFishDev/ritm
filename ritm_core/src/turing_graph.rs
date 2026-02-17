use indexmap::IndexMap;
use thiserror::Error;

use crate::{
    turing_index::{TransitionId, TuringStateIndex, TuringTransitionIndex},
    turing_tape::TuringTapeError,
    turing_transition::{
        TransitionMultRibbonInfo, TransitionOneRibbonInfo, TransitionsInfo, TuringTransition,
        TuringTransitionWrapper,
    },
};
use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
};

#[derive(Debug, Error)]
pub enum TuringGraphError {
    #[error("Tried to modify (or remove) a state that cannot be changed : {state}")]
    ImmutableStateError { state: TuringStateInfo },
    #[error(
        "Tried to add a state (or rename an existant one) with the name \"{name}\" but it is already owned by \"{state}\" "
    )]
    AlreadyPresentNameError {
        name: String,
        state: TuringStateInfo,
    },
    #[error(
        "Tried to add the transition \"{transition}\" between \"{from}\" and \"{to}\" but it was already present"
    )]
    AlreadyPresentTransitionError {
        from: TuringStateIndex,
        to: TuringStateIndex,
        transition: TransitionsInfo,
    },
    #[error(
        "Tried to access a state using index \"{accessed_index}\" but it is not present in the graph"
    )]
    UnknownStateIndex { accessed_index: TuringStateIndex },
    #[error(
        "Tried to access a transition using index \"{accessed_index}\" but it is not present in the graph"
    )]
    UnknownTransitionIndex {
        accessed_index: TuringTransitionIndex,
    },
    #[error("Encountered a tape error : {0}")]
    TuringTapeError(#[from] TuringTapeError),
    #[error(
        "Expected a transition with {expected} elements but found a transition with {received} instead."
    )]
    IncompatibleTransitionError { expected: usize, received: usize },
}

pub const DEFAULT_INIT_STATE: &str = "i";
pub const DEFAULT_ACCEPTING_STATE: &str = "a";

#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Represents additional data being carried in the turing graph.
pub trait TuringState: Clone + Default + Debug {
    /// Creates a new init state
    fn new_init() -> Self;
    /// Creates a new accepting state
    fn new_accepting() -> Self;
}

impl Display for TuringStateInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, id: {}, type: {})",
            self.name, self.id, self.state_type
        )
    }
}

impl<S: TuringState> From<TuringStateWrapper<S>> for TuringStateInfo {
    fn from(value: TuringStateWrapper<S>) -> Self {
        value.info
    }
}
impl<S: TuringState> From<&TuringStateWrapper<S>> for TuringStateInfo {
    fn from(value: &TuringStateWrapper<S>) -> Self {
        value.info.clone()
    }
}

/// Represents a node in this graph.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TuringStateWrapper<S: TuringState> {
    pub inner_state: S,
    info: TuringStateInfo,
}

impl<S: TuringState> TuringStateWrapper<S> {
    /// Returns the important information of this state (name, type, id).
    pub fn get_info(&self) -> &TuringStateInfo {
        &self.info
    }
    /// Returns the additional information carried in this state.
    pub fn get_inner(&self) -> &S {
        &self.inner_state
    }
    /// Returns the name of this state.
    pub fn get_name(&self) -> &String {
        &self.info.name
    }
    /// Returns the type of this state.
    pub fn get_type(&self) -> TuringStateType {
        self.info.state_type.clone()
    }
    /// Returns the id of this state.
    pub fn get_id(&self) -> usize {
        self.info.id
    }
}

/// Contains all information in relation to a state of a graph.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TuringStateInfo {
    /// The name of the state (unique in the graph).
    name: String,
    /// The type of this state.
    state_type: TuringStateType,
    /// The identifier of this state (unique in this graph).
    id: usize,
}

impl TuringStateInfo {
    /// Gets the type of this state.
    pub fn get_type(&self) -> TuringStateType {
        self.state_type.clone()
    }
    /// Gets the identifier of this state.
    pub fn get_id(&self) -> usize {
        self.id
    }
    /// Gets the name of this state.
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

/// A tuple containing three values in relation to an existing transitions.
/// * The first one, is the state the transition starts from.
/// * The second one, is the transition itself.
/// * The third one, is the destination of the transition.
type TransitionRefs<'a, S, T> = (
    &'a TuringStateWrapper<S>,
    &'a TuringTransitionWrapper<T>,
    &'a TuringStateWrapper<S>,
);

#[derive(Debug, Clone, PartialEq)]
/// A struct representing a Turing Machine graph either with a reading and `k` **writting** tapes (`k >= 1`).
/// Or a graph for a machine with only one ribbon.
pub struct TuringGraph<S, T>
where
    S: TuringState,
    T: TuringTransition,
{
    /// The hashmap containing all the states present in this graph. `identifier` -> `state`
    state_hashmap: IndexMap<usize, TuringStateWrapper<S>>,
    /// The hashmap containing all the transitions present in this graph. `(from, to)` -> `transitions`
    transition_hasmap: IndexMap<(usize, usize), Vec<TuringTransitionWrapper<T>>>,
    /// The next id to give to a state.
    next_state_id: usize,
    /// A list of id's that are available due to a removed state.
    available_state_id: VecDeque<usize>,
    /// The number of tapes this graph was made for.
    k: usize,
}

impl<S: TuringState, T: TuringTransition> Default for TuringGraph<S, T> {
    fn default() -> Self {
        Self::new(1, false).expect("correct for one work ribbon")
    }
}

impl<S: TuringState> TuringStateWrapper<S> {
    fn new_normal(name: impl Into<String>, index: usize) -> Self {
        Self::new_type(S::default(), name, index, TuringStateType::Normal)
    }
    fn new_accepting(name: impl Into<String>, index: usize) -> Self {
        Self::new_type(S::new_accepting(), name, index, TuringStateType::Accepting)
    }
    fn new_type(
        inner_state: S,
        name: impl Into<String>,
        index: usize,
        state_type: TuringStateType,
    ) -> Self {
        Self {
            inner_state,
            info: TuringStateInfo {
                name: name.into(),
                state_type,
                id: index,
            },
        }
    }
}

impl<S, T> TuringGraph<S, T>
where
    S: TuringState,
    T: TuringTransition,
{
    /// Creates a new empty Turing Machine graph. The graph accepts transitions involving `k` writting tapes (`k > 0`), as well as transitions only involving one tape.
    /// * Always creates :
    ///     * `q_i` : The initial state
    /// * If `default_state` parameter is set to [`true`] :
    ///     * `q_a` : The default accepting state.
    pub fn new(k: usize, default_state: bool) -> Result<Self, TuringGraphError> {
        // Add the default states
        let mut state_hashmap = IndexMap::new();

        // Always adds init
        state_hashmap.insert(
            0,
            TuringStateWrapper::new_type(
                TuringState::new_init(),
                DEFAULT_INIT_STATE,
                0,
                TuringStateType::Normal,
            ),
        );

        let mut next_state_index = 1;
        // Only adds default accepting if needed
        if default_state {
            state_hashmap.insert(
                1,
                TuringStateWrapper::new_accepting(DEFAULT_ACCEPTING_STATE, 1),
            );
            next_state_index = 2;
        }

        Ok(Self {
            state_hashmap,
            transition_hasmap: IndexMap::new(),
            available_state_id: VecDeque::new(),
            next_state_id: next_state_index,
            k,
        })
    }

    /// Adds a new rule to a state of the machine of the form : `from {transition} to`.
    /// Meaning, a new edge is added to the graph.
    ///
    /// # Errors :
    /// * [`TuringGraphError::UnknownStateIndex`] if one of the given state does not exists.
    /// * [`TuringGraphError::IncompatibleTransitionError`] if the transition number of ribbons is not compatible with the others already present.
    /// * [`TuringGraphError::AlreadyPresentTransitionError`] if the transition between the same nodes already exists.
    pub fn append_transition(
        &mut self,
        from: impl Into<TuringStateIndex>,
        transition: impl Into<TuringTransitionWrapper<T>>,
        to: impl Into<TuringStateIndex>,
    ) -> Result<(), TuringGraphError> {
        let transition = transition.into();

        let from = from.into();
        let to = to.into();

        // Checks if the given number of tapes is correct
        if transition.info.get_nb_ribbons() != (self.k + 1) {
            return Err(TuringGraphError::IncompatibleTransitionError {
                expected: self.k + 1,
                received: transition.info.get_nb_ribbons(),
            });
        }

        let state_from = self.try_get_state(from.clone())?.info.id;
        let state_to = self.try_get_state(to.clone())?.info.id;

        let transition_vector = self
            .transition_hasmap
            .entry((state_from, state_to))
            .or_default();
        if transition_vector.contains(&transition) {
            return Err(TuringGraphError::AlreadyPresentTransitionError {
                from,
                to,
                transition: transition.info,
            });
        }
        transition_vector.push(transition);
        Ok(())
    }

    /// Creates a valid default transitions between two states.
    /// See [`TuringTransitionInfo::create_default`] for more informations.
    pub fn append_default_transition(
        &mut self,
        from: impl Into<TuringStateIndex>,
        additional_info: Option<T>,
        to: impl Into<TuringStateIndex>,
    ) -> Result<(), TuringGraphError> {
        let default_transition = if self.k == 0 {
            TransitionOneRibbonInfo::default().into()
        } else {
            TransitionMultRibbonInfo::create_default(self.k).into()
        };

        if let Some(inner_transition) = additional_info {
            self.append_transition(
                from,
                TuringTransitionWrapper {
                    info: default_transition,
                    inner_transition,
                },
                to,
            )
        } else {
            self.append_transition(from, default_transition, to)
        }
    }

    /// Adds a new state using a name and a type to the turing machine graph and returns its index if not already present.
    /// # Errors :
    /// * [`TuringGraphError::AlreadyPresentNameError`] if the given name is already present in the graph.
    pub fn try_add_state(
        &mut self,
        name: impl ToString,
        state_type: TuringStateType,
    ) -> Result<usize, TuringGraphError> {
        let name = name.to_string();
        match self.get_state_index(&name) {
            Some(index) => Err(TuringGraphError::AlreadyPresentNameError {
                name,
                state: self.get_state(index).expect("is present").info.clone(),
            }),
            None => Ok(self.add_state(name, state_type)),
        }
    }

    /// Adds a new state to the turing machine graph and returns its index. And if the state was already present then its index nor is type will be affected.
    pub fn add_state(&mut self, name: impl ToString, state_type: TuringStateType) -> usize {
        let name = name.to_string();
        match self.get_state_index(&name) {
            Some(index) => index,
            None => {
                let state_id = match self.available_state_id.pop_front() {
                    Some(id) => id,
                    None => {
                        self.next_state_id += 1;
                        self.next_state_id - 1
                    }
                };
                self.state_hashmap.insert(state_id, {
                    if state_type == TuringStateType::Accepting {
                        TuringStateWrapper::new_accepting(name, state_id)
                    } else {
                        TuringStateWrapper::new_normal(name, state_id)
                    }
                });

                state_id
            }
        }
    }

    /// Returns the state (*node*) at the given index.
    /// # Errors
    /// * [TuringGraphError::UnknownStateIndex] if the given index is not present in the graph.
    pub fn try_get_state(
        &self,
        index: impl Into<TuringStateIndex>,
    ) -> Result<&TuringStateWrapper<S>, TuringGraphError> {
        let index = index.into();

        let index_res = match &index {
            TuringStateIndex::ID(id) => Some(*id),
            TuringStateIndex::Value(val) => self.get_state_index(val),
        };
        if let Some(id) = index_res
            && let Some(state) = self.state_hashmap.get(&id)
        {
            return Ok(state);
        };

        Err(TuringGraphError::UnknownStateIndex {
            accessed_index: index,
        })
    }

    /// Returns the state at the given index if it is present, [`None`] if not.
    pub fn get_state(&self, index: impl Into<TuringStateIndex>) -> Option<&TuringStateWrapper<S>> {
        self.try_get_state(index).ok()
    }

    /// Returns the **mutable** state (*node*) at the given index.
    /// # Errors
    /// * [TuringGraphError::UnknownStateIndex] if the given index is not present in the graph.
    pub fn try_get_state_mut(
        &mut self,
        index: impl Into<TuringStateIndex>,
    ) -> Result<&mut TuringStateWrapper<S>, TuringGraphError> {
        let index = index.into();

        let index_res = match &index {
            TuringStateIndex::ID(id) => Some(*id),
            TuringStateIndex::Value(val) => self.get_state_index(val),
        };
        if let Some(id) = index_res
            && let Some(state) = self.state_hashmap.get_mut(&id)
        {
            return Ok(state);
        };

        Err(TuringGraphError::UnknownStateIndex {
            accessed_index: index,
        })
    }

    /// Returns the state index using the given value if it exists.
    fn get_state_index(&self, state_name: impl ToString) -> Option<usize> {
        let state_name = state_name.to_string();
        for (index, state) in &self.state_hashmap {
            if state.info.name == state_name {
                return Some(*index);
            }
        }
        None
    }

    /// Returns the list of transitions that are actually possible to take on a state when reading a certain set of characters.
    /// The returned vector contains tuples :
    /// * The first element is a transition that we could take.
    /// * The second element is the identifier of the state targetted.
    pub fn get_valid_transitions(
        &self,
        index: impl Into<TuringStateIndex>,
        chars_read: Vec<char>,
    ) -> Result<Vec<(&TuringTransitionWrapper<T>, usize)>, TuringGraphError> {
        let mut res = Vec::new();

        let state_id = self.try_get_state(index)?.info.id;

        self.transition_hasmap
            .iter()
            .for_each(|((from, to), transitions)| {
                // If the state is the one we are looking for
                if *from == state_id {
                    // If the character to read are equivalent
                    transitions.iter().for_each(|transition| {
                        if transition.info.is_valid(&chars_read) {
                            res.push((transition, *to));
                        }
                    });
                }
            });

        Ok(res)
    }

    /// Returns the list of transitions that are actually possible to take on a state when reading a certain set of characters.
    /// The returned vector contains tuples :
    /// * The first value is the index of the transition in the full list of transitions between the initial state and the destination (second tuple value).
    /// * The second element is the identifier of the state targetted.
    pub fn get_valid_transitions_indexes(
        &self,
        index: impl Into<TuringStateIndex>,
        chars_read: Vec<char>,
    ) -> Result<Vec<(usize, usize)>, TuringGraphError> {
        let mut res = Vec::new();

        let state_id = self.try_get_state(index)?.info.id;

        self.transition_hasmap
            .iter()
            .for_each(|((from, to), transitions)| {
                // If the state is the one we are looking for
                if *from == state_id {
                    // If the character to read are equivalent
                    transitions.iter().enumerate().for_each(|(i, transition)| {
                        if transition.info.is_valid(&chars_read) {
                            res.push((*to, i));
                        }
                    });
                }
            });

        Ok(res)
    }

    /// Get the transitions between two nodes if any.
    /// # Errors
    /// * [TuringGraphError::UnknownStateIndex] if one of the given index is not present in the graph.
    pub fn get_transitions(
        &self,
        from: impl Into<TuringStateIndex>,
        to: impl Into<TuringStateIndex>,
    ) -> Result<Option<&Vec<TuringTransitionWrapper<T>>>, TuringGraphError> {
        let from = from.into();
        let to = to.into();

        let state_from = self.try_get_state(from)?.info.id;
        let state_to = self.try_get_state(to)?.info.id;

        Ok(self.transition_hasmap.get(&(state_from, state_to)))
    }

    /// Get the transitions between two nodes if any.
    /// # Errors
    /// * [TuringGraphError::UnknownStateIndex] if one of the given index is not present in the graph.
    pub fn get_transitions_mut(
        &mut self,
        from: impl Into<TuringStateIndex>,
        to: impl Into<TuringStateIndex>,
    ) -> Result<Option<&mut Vec<TuringTransitionWrapper<T>>>, TuringGraphError> {
        let from = from.into();
        let to = to.into();

        let state_from = self.try_get_state(from)?.info.id;
        let state_to = self.try_get_state(to)?.info.id;

        Ok(self.transition_hasmap.get_mut(&(state_from, state_to)))
    }

    /// Removes **all** the transitions from this state to the given node.
    /// # Errors
    /// * [TuringGraphError::UnknownStateIndex] if one of the given index is not present in the graph.
    pub fn remove_transitions(
        &mut self,
        from: impl Into<TuringStateIndex>,
        to: impl Into<TuringStateIndex>,
    ) -> Result<(), TuringGraphError> {
        let state_from = self.try_get_state(from)?.info.id;
        let to_from = self.try_get_state(to)?.info.id;

        // Remove all transitions from n1 to n2
        self.transition_hasmap.swap_remove(&(state_from, to_from));
        Ok(())
    }

    /// Removes a transition of the form `from {transition} to` using the given parameters.
    /// If the given transition is not present, no error will be returned (and no panic will be called).
    /// # Errors
    /// * [TuringGraphError::UnknownStateIndex] if one of the given index is not present in the graph.
    pub fn remove_transition(
        &mut self,
        transition_id: impl Into<TuringTransitionIndex>,
    ) -> Result<(), TuringGraphError> {
        let transition_id = transition_id.into();

        let to_remove = transition_id.transition_id;
        let state_from = self.try_get_state(transition_id.source_id)?.info.id;
        let state_to = self.try_get_state(transition_id.target_id)?.info.id;
        if let Some(transitions) = self.transition_hasmap.get_mut(&(state_from, state_to)) {
            match to_remove {
                TransitionId::ID(id) => {
                    if id <= transitions.len() {
                        transitions.remove(id);
                    }
                }
                TransitionId::Value(turing_transition_info) => {
                    // Only keep transition not fitting the given information
                    transitions.retain(|trans| trans.info != turing_transition_info);
                }
            };
        }

        // If there aren't any transitions between the two nodes, remove the vector.
        if self
            .transition_hasmap
            .get(&(state_from, state_to))
            .is_some_and(|v| v.is_empty())
        {
            self.transition_hasmap.swap_remove(&(state_from, state_to));
        }

        Ok(())
    }

    /// Removes a state and **all** mentions of it in **all** transitions of **all** the other states of the graph.
    /// # Errors
    /// * [TuringGraphError::UnknownStateIndex] if the given index is not present in the graph.
    /// * [TuringGraphError::ImmutableStateError] if the given index is one of the states that cannot be removed (like the initial one).
    pub fn remove_state(
        &mut self,
        index: impl Into<TuringStateIndex>,
    ) -> Result<(), TuringGraphError> {
        // First keep the index for later
        let state_id = self.try_get_state(index)?.info.id;
        if state_id == 0 {
            return Err(TuringGraphError::ImmutableStateError {
                state: self.try_get_state(0)?.info.clone(),
            });
        }

        // Remove all transitions that start with the index
        let keys_to_remove: Vec<(usize, usize)> = self
            .transition_hasmap
            .keys()
            .filter_map(|(from, to)| {
                if *from == state_id || *to == state_id {
                    Some((*from, *to))
                } else {
                    None
                }
            })
            .collect();

        for key in keys_to_remove {
            self.transition_hasmap.swap_remove(&key);
        }
        // This id is now availalbe
        self.available_state_id.push_back(state_id);
        // Finally remove the state itself
        self.state_hashmap.swap_remove(&state_id);
        Ok(())
    }

    /// Gets references all indexed values from the given [`TuringTransitionIndex`].
    pub fn try_get_states_transition(
        &self,
        transition_id: impl Into<TuringTransitionIndex>,
    ) -> Result<TransitionRefs<'_, S, T>, TuringGraphError> {
        // Retourne ref to source, transition and target
        let transition_id = transition_id.into();
        let transition_clone = transition_id.clone();

        let source = self.try_get_state(transition_id.source_id)?;
        let target = self.try_get_state(transition_id.target_id)?;

        let transition = match self
            .transition_hasmap
            .get(&(source.get_id(), target.get_id()))
        {
            Some(transitions) => match transition_id.transition_id {
                TransitionId::ID(id) => transitions.get(id),
                TransitionId::Value(turing_transition_info) => transitions
                    .iter()
                    .find(|t| t.info == turing_transition_info),
            },
            None => None,
        };
        if let Some(transition) = transition {
            Ok((source, transition, target))
        } else {
            Err(TuringGraphError::UnknownTransitionIndex {
                accessed_index: transition_clone,
            })
        }
    }

    /// Gets the transition referenced by the given index if it exists.
    pub fn try_get_transition(
        &self,
        transition_id: impl Into<TuringTransitionIndex>,
    ) -> Result<&TuringTransitionWrapper<T>, TuringGraphError> {
        Ok(self.try_get_states_transition(transition_id)?.1)
    }

    /// Returns the number of writting ribbons.
    pub fn get_k(&self) -> usize {
        self.k
    }

    /// Returns the hashmap containing all the states present in this graph. `identifier` -> `state`
    pub fn get_state_hashmap(&self) -> &IndexMap<usize, TuringStateWrapper<S>> {
        &self.state_hashmap
    }

    /// Returns a vector with a reference of all state contained in the graph.
    pub fn get_states(&self) -> Vec<&TuringStateWrapper<S>> {
        self.state_hashmap.values().collect()
    }

    /// Returns a vector with a mutable reference of all state contained in the graph.
    pub fn get_states_mut(&mut self) -> Vec<&mut TuringStateWrapper<S>> {
        self.state_hashmap.values_mut().collect()
    }

    /// Changes the name of an already present state in the graph.
    /// # Errors
    /// * [`TuringGraphError::AlreadyPresentNameError`] if the new name is already present in the graph.
    /// * [`TuringGraphError::UnknownStateIndex`] if the given index is not present in the graph.
    pub fn rename_state(
        &mut self,
        index: impl Into<TuringStateIndex>,
        new_name: impl ToString,
    ) -> Result<(), TuringGraphError> {
        let index = index.into();

        let new_name = new_name.to_string();
        // Check if the new name is not already present somewhere else (does not crash when trying to rename a state using the same index)
        if let Ok(val) = self.try_get_state(&new_name) {
            return Err(TuringGraphError::AlreadyPresentNameError {
                name: new_name,
                state: (val).into(),
            });
        }

        // Try to get state :
        let state = self.try_get_state_mut(index)?;
        // Updates the name
        state.info.name = new_name;

        Ok(())
    }

    /// Returns the hashmap containing all the transitions present in this graph. `(from, to)` -> `transition`
    pub fn get_transitions_hashmap(
        &self,
    ) -> &IndexMap<(usize, usize), Vec<TuringTransitionWrapper<T>>> {
        &self.transition_hasmap
    }

    /// Checks that the given state has no outgoing transitions.
    /// # Errors
    /// * [`TuringGraphError::UnknownStateIndex`] if the given index is not present in the graph.
    pub fn is_state_dead_end(
        &self,
        index: impl Into<TuringStateIndex>,
    ) -> Result<bool, TuringGraphError> {
        let state = self.try_get_state(index)?.get_id();

        let mut is_end = true;
        for transition in self.transition_hasmap.keys() {
            if transition.0 == state && transition.1 != state {
                is_end = false;
            }
        }
        Ok(is_end)
    }

    /// Checks that the given state has some ingoing transitions.
    /// # Errors
    /// * [`TuringGraphError::UnknownStateIndex`] if the given index is not present in the graph.
    pub fn is_state_accessible(
        &self,
        index: impl Into<TuringStateIndex>,
    ) -> Result<bool, TuringGraphError> {
        let state = self.try_get_state(index)?.get_id();

        let mut is_accessible = false;
        for transition in self.transition_hasmap.keys() {
            if transition.1 == state {
                is_accessible = true;
            }
        }
        Ok(is_accessible)
    }
}

impl<S: TuringState, T: TuringTransition> Display for TuringGraph<S, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::from("States:\n");

        // Print all states
        for state in self.state_hashmap.values() {
            res.push_str(format!("{}: {}\n", state.info.name, state.info.state_type).as_str());
        }

        res.push_str("\nTransitions:\n");
        let mut res_tr = String::new();
        // Print all transitions btw states
        for ((q1, q2), transitions) in &self.transition_hasmap {
            if transitions.is_empty() {
                continue;
            }
            let q1 = &self.state_hashmap[q1];
            let q2 = &self.state_hashmap[q2];
            res_tr.push_str(format!("q_{} {} ", q1.get_name(), '{').as_str());
            let spaces = 3 + q1.get_name().len();
            for transition in transitions.iter().take(transitions.len() - 1) {
                res_tr.push_str(format!("{} \n{}| ", transition.info, " ".repeat(spaces)).as_str());
            }
            // add last
            res_tr.push_str(format!("{} ", transitions.last().unwrap().info).as_str());

            res_tr.push_str(format!("{} q_{};\n\n", "}", q2.get_name()).as_str());
        }
        if res_tr.is_empty() {
            res.push_str("None");
        } else {
            res.push_str(res_tr.as_str());
        }

        write!(f, "{}", res)
    }
}
