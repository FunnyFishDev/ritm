use thiserror::Error;

use crate::{
    turing_tape::TuringTapeError,
    turing_transition::{TuringTransition, TuringTransitionInfo, TuringTransitionWrapper},
};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

#[derive(Debug, Error)]
pub enum TuringGraphError {
    #[error("Tried to create a turing machine with no writting tapes")]
    NotEnoughTapesError,
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
        from: TuringGraphIndex,
        to: TuringGraphIndex,
        transition: TuringTransitionInfo,
    },
    #[error(
        "Tried to access a state using index \"{accessed_index}\" but it is present in the graph"
    )]
    UnknownStateIndex { accessed_index: TuringGraphIndex },
    #[error("Encountered a tape error : {0}")]
    TuringTapeError(#[from] TuringTapeError),
    #[error(
        "Expected a transition with {expected} elements but found a transition with {received} instead."
    )]
    IncompatibleTransitionError { expected: usize, received: usize },
}

/// Can be used to try to find a state in the graph
#[derive(Debug, Clone, PartialEq)]
pub enum TuringGraphIndex {
    /// Represents the index value of the state to get.
    ID(usize),
    /// Represents the name of the state to get.
    Name(String),
}

impl TuringGraphIndex {
    pub fn from(index: impl Into<TuringGraphIndex>) -> Self {
        index.into()
    }
}

impl Display for TuringGraphIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TuringGraphIndex::ID(val) => val.to_string(),
                TuringGraphIndex::Name(val) => val.to_string(),
            }
        )
    }
}

impl From<String> for TuringGraphIndex {
    fn from(value: String) -> Self {
        TuringGraphIndex::Name(value)
    }
}
impl From<&String> for TuringGraphIndex {
    fn from(value: &String) -> Self {
        value.to_string().into()
    }
}

impl From<&str> for TuringGraphIndex {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<usize> for TuringGraphIndex {
    fn from(value: usize) -> Self {
        TuringGraphIndex::ID(value)
    }
}

impl From<&usize> for TuringGraphIndex {
    fn from(value: &usize) -> Self {
        (*value).into()
    }
}

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

pub trait TuringState: Clone + Default + Debug {
    fn new_init() -> Self;
    fn new_accepting() -> Self;
    fn visited(&mut self);
}

impl Display for TuringStateInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}: {}, {})", self.id, self.name, self.state_type)
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TuringStateWrapper<S: TuringState> {
    inner_state: S,
    info: TuringStateInfo,
}

impl<S: TuringState> TuringStateWrapper<S> {
    pub fn get_info(&self) -> &TuringStateInfo {
        &self.info
    }
    pub fn get_name(&self) -> &String {
        &self.info.name
    }
    pub fn get_type(&self) -> &TuringStateType {
        &self.info.state_type
    }
    pub fn get_id(&self) -> usize {
        self.info.id
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TuringStateInfo {
    name: String,
    state_type: TuringStateType,
    id: usize,
}

impl TuringStateInfo {
    pub fn get_type(&self) -> &TuringStateType {
        &self.state_type
    }
    pub fn get_id(&self) -> usize {
        self.id
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, Clone)]
/// A struct representing a Turing Machine graph with `k` **writting** tapes (`k >= 1`).
pub struct TuringMachineGraph<S, T>
where
    S: TuringState,
    T: TuringTransition,
{
    state_hashmap: HashMap<usize, TuringStateWrapper<S>>,
    transition_hasmap: HashMap<(usize, usize), Vec<TuringTransitionWrapper<T>>>,

    next_state_index: usize,

    /// The number of tapes this graph was made for
    k: usize,
}

impl<S: TuringState> TuringStateWrapper<S> {
    fn new_normal(inner_state: S, name: impl Into<String>, index: usize) -> Self {
        Self::new_type(inner_state, name, index, TuringStateType::Normal)
    }
    fn new_accepting(inner_state: S, name: impl Into<String>, index: usize) -> Self {
        Self::new_type(inner_state, name, index, TuringStateType::Accepting)
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

impl<S, T> TuringMachineGraph<S, T>
where
    S: TuringState,
    T: TuringTransition,
{
    /// Creates a new empty Turing Machine graph that has `k` writting tapes (`k >= 1`).
    ///
    /// default states will be created :
    /// * `q_i` : The initial state
    /// * `q_a` : The default accepting state, but only if `default_acc_state` is set to True.
    pub fn new(k: usize, default_acc_state: bool) -> Result<Self, TuringGraphError> {
        if k == 0 {
            return Err(TuringGraphError::NotEnoughTapesError);
        }
        // Add the default states
        let mut state_hashmap = HashMap::new();

        // Always adds init
        state_hashmap.insert(
            0,
            TuringStateWrapper::new_normal(TuringState::new_init(), "i", 0),
        );

        let mut next_state_index = 1;
        // Only adds default accepting if needed
        if default_acc_state {
            state_hashmap.insert(
                1,
                TuringStateWrapper::new_accepting(TuringState::new_accepting(), "a", 1),
            );
            next_state_index = 2;
        }

        Ok(Self {
            state_hashmap,
            transition_hasmap: HashMap::new(),
            next_state_index,
            k,
        })
    }

    /// Adds a new rule to a state of the machine of the form : `from {transition} to`.
    /// Meaning, a new edge is added to the graph.
    ///
    /// If one of the given state didn't already exists, a [TuringError::UnknownStateError] will be returned.
    pub fn append_transition(
        &mut self,
        from: impl Into<TuringGraphIndex>,
        transition: impl Into<TuringTransitionWrapper<T>>,
        to: impl Into<TuringGraphIndex>,
    ) -> Result<(), TuringGraphError> {
        let transition = transition.into();

        let from = from.into();
        let to = to.into();

        // Checks if the given number of tapes is correct
        if transition.get_number_of_affected_tapes() != (self.k + 1) {
            return Err(TuringGraphError::IncompatibleTransitionError {
                expected: self.get_k(),
                received: transition.get_number_of_affected_tapes() - 1,
            });
        }

        let state_from = self.get_state(from.clone())?.info.id;
        let state_to = self.get_state(to.clone())?.info.id;

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

    /// Adds a new state to the turing machine graph and returns its index. Meaning a new node is added to the graph.
    ///
    /// If the state name already existed then the index of the already existing state is returned and its state type does not change.
    pub fn add_state(&mut self, name: impl ToString, state_type: TuringStateType) -> usize {
        let name = name.to_string();
        match self.get_state_index(&name) {
            Some(index) => index,
            None => {
                self.state_hashmap.insert(
                    self.next_state_index,
                    TuringStateWrapper::new_type(
                        S::default(),
                        name,
                        self.next_state_index,
                        state_type,
                    ),
                );
                self.next_state_index += 1;
                self.next_state_index - 1
            }
        }
    }

    /// Returns the state (*node*) at the given index.
    pub fn get_state(
        &self,
        index: impl Into<TuringGraphIndex>,
    ) -> Result<&TuringStateWrapper<S>, TuringGraphError> {
        let index = index.into();

        let index_res = match &index {
            TuringGraphIndex::ID(id) => Some(*id),
            TuringGraphIndex::Name(val) => self.get_state_index(val),
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

    /// Returns the **mutable** state (*node*) at the given index.
    pub fn get_state_mut(
        &mut self,
        index: impl Into<TuringGraphIndex>,
    ) -> Result<&mut TuringStateWrapper<S>, TuringGraphError> {
        let index = index.into();

        let index_res = match &index {
            TuringGraphIndex::ID(id) => Some(*id),
            TuringGraphIndex::Name(val) => self.get_state_index(val),
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

    pub fn get_valid_transitions(
        &self,
        index: impl Into<TuringGraphIndex>,
        chars_read: Vec<char>,
    ) -> Result<Vec<(&TuringTransitionWrapper<T>, usize)>, TuringGraphError> {
        let mut res = Vec::new();

        let state_id = self.get_state(index)?.info.id;

        self.transition_hasmap
            .iter()
            .for_each(|((from, to), transitions)| {
                // If the state is the one we are looking for
                if *from == state_id {
                    // If the character to read are equivalent
                    transitions.iter().for_each(|transition| {
                        if transition.info.chars_read == chars_read {
                            res.push((transition, *to));
                        }
                    });
                }
            });

        Ok(res)
    }

    /// Get the transitions between two nodes if any.
    pub fn get_transitions(
        &self,
        from: impl Into<TuringGraphIndex>,
        to: impl Into<TuringGraphIndex>,
    ) -> Result<Option<&Vec<TuringTransitionWrapper<T>>>, TuringGraphError> {
        let from = from.into();
        let to = to.into();

        let state_from = self.get_state(from)?.info.id;
        let state_to = self.get_state(to)?.info.id;

        Ok(self.transition_hasmap.get(&(state_from, state_to)))
    }

    /// Removes **all** the transitions from this state to the given node
    pub fn remove_transitions(
        &mut self,
        from: impl Into<TuringGraphIndex>,
        to: impl Into<TuringGraphIndex>,
    ) -> Result<(), TuringGraphError> {
        let state_from = self.get_state(from)?.info.id;
        let to_from = self.get_state(to)?.info.id;

        // Remove all transitions from n1 to n2
        self.transition_hasmap.remove(&(state_from, to_from));
        Ok(())
    }

    /// Removes all transitions of the form `from {transition} to` using the given parameters.
    pub fn remove_transition(
        &mut self,
        from: impl Into<TuringGraphIndex>,
        transition: &TuringTransitionWrapper<T>,
        to: impl Into<TuringGraphIndex>,
    ) -> Result<(), TuringGraphError> {
        let state_from = self.get_state(from)?.info.id;
        let state_to = self.get_state(to)?.info.id;

        if let Some(transitions) = self.transition_hasmap.get_mut(&(state_from, state_to)) {
            for trans in transitions {}
        }

        // TODO: Fix this !

        Ok(())
    }

    /// Removes a state and **all** mentions of it in **all** transitions of **all** the other states of the graph.
    pub fn remove_state(
        &mut self,
        index: impl Into<TuringGraphIndex>,
    ) -> Result<(), TuringGraphError> {
        // First keep the index for later
        let state_id = self.get_state(index)?.info.id;
        if state_id == 0 {
            return Err(TuringGraphError::ImmutableStateError {
                state: self.get_state(0)?.info.clone(),
            });
        }

        // Remove all transitions that start with the index
        let keys_to_remove: Vec<(usize, usize)> = self
            .transition_hasmap
            .keys()
            .filter_map(|(from, to)| {
                if *from == state_id {
                    Some((*from, *to))
                } else {
                    None
                }
            })
            .collect();

        for key in keys_to_remove {
            self.transition_hasmap.remove(&key);
        }
        // Finally remove the state itself
        self.state_hashmap.remove(&state_id);
        Ok(())
    }

    pub fn get_k(&self) -> usize {
        self.k
    }

    pub fn get_state_hashmap(&self) -> &HashMap<usize, TuringStateWrapper<S>> {
        &self.state_hashmap
    }

    pub fn get_states(&self) -> Vec<&TuringStateWrapper<S>> {
        self.state_hashmap.values().collect()
    }

    pub fn rename_state(
        &mut self,
        index: impl Into<TuringGraphIndex>,
        new_name: impl ToString,
    ) -> Result<(), TuringGraphError> {
        let index = index.into();

        let new_name = new_name.to_string();
        // Check if the new name is not already present somewhere else (does not crash when trying to rename a state using the same index)
        if let Ok(val) = self.get_state(&new_name) {
            return Err(TuringGraphError::AlreadyPresentNameError {
                name: new_name,
                state: (val).into(),
            });
        }

        // Try to get state :
        let state = self.get_state_mut(index)?;
        // Updates the name
        state.info.name = new_name;

        Ok(())
    }

    // fn to_id(&self, index: impl Into<TuringGraphIndex>) -> usize {
    //     match index.into() {
    //         TuringGraphIndex::ID(id) => id,
    //         TuringGraphIndex::Name(name) => self.get_state_index(name).expect("present"),
    //     }
    // }
}

// impl<S: TuringState, T: TuringTransition> Display for TuringMachineGraph<S, T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut res = String::from("States:\n");

//         // Print all states
//         for state in self.state_hashmap.values() {
//             res.push_str(format!("{}: {}\n", state.name, state.state_type).as_str());
//         }

//         res.push_str("\nTransitions:\n");
//         let mut res_tr = String::new();
//         // Print all transitions btw states
//         for (q1, i1) in &self.name_index_hashmap {
//             for (q2, i2) in &self.name_index_hashmap {
//                 let transitions = self.get_transitions_by_index(*i1, *i2).unwrap();
//                 if transitions.is_empty() {
//                     continue;
//                 }
//                 res_tr.push_str(format!("q_{} {} ", q1, '{').as_str());
//                 let spaces = 3 + q1.len();

//                 for i in 0..transitions.len() - 1 {
//                     res_tr.push_str(
//                         format!("{} \n{}| ", transitions.get(i).unwrap(), " ".repeat(spaces))
//                             .as_str(),
//                     );
//                 }
//                 // add last
//                 res_tr.push_str(format!("{} ", transitions.last().unwrap()).as_str());

//                 res_tr.push_str(format!("{} q_{};\n\n", "}", q2).as_str());
//             }
//         }
//         if res_tr.is_empty() {
//             res.push_str("None");
//         } else {
//             res.push_str(res_tr.as_str());
//         }

//         write!(f, "{}", res)
//     }
// }
