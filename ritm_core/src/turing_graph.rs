use thiserror::Error;

use crate::{
    turing_state::{TuringState, TuringStateError, TuringStateType},
    turing_tape::TuringTapeError,
    turing_transition::TuringTransition,
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
    ImmutableStateError { state: TuringState },
    #[error(
        "Tried to access a state with the index {accessed_index} but only {states_len} states are present"
    )]
    OutOfRangeStateError {
        accessed_index: usize,
        states_len: usize,
    },
    #[error("Ran into the following state error : {0}")]
    StateError(#[from] TuringStateError),
    #[error("Trying to access the state \"{state_name}\" but it is not in this graph")]
    UnknownStateError { state_name: String },
    #[error("Encountered a tape error : {0}")]
    TuringTapeError(#[from] TuringTapeError),
    #[error(
        "Expected a transition with {expected} elements but found a transition with {received} instead."
    )]
    IncompatibleTransitionError { expected: usize, received: usize },
}

#[derive(Debug, Clone)]
/// A struct representing a Turing Machine graph with `k` **writting** tapes (`k >= 1`).
pub struct TuringMachineGraph {
    /// The hashmap containing a mapping of all nodes names and their related index in the `states` field.
    name_index_hashmap: HashMap<String, usize>,
    /// The vector containing all the nodes of the turing machine graph
    states: Vec<TuringState>,
    /// The number of tapes this graph was made for
    k: usize,
}

impl TuringMachineGraph {
    /// Creates a new empty Turing Machine graph that has `k` writting tapes (`k >= 1`).
    ///
    /// Three default states will be created :
    /// * `q_i` : The initial state
    /// * `q_a` : The default accepting state
    /// * `q_r` : The default rejecting state
    pub fn new(k: usize) -> Result<Self, TuringGraphError> {
        if k == 0 {
            return Err(TuringGraphError::NotEnoughTapesError);
        }
        // Add the default states
        let init_state = TuringState::new(TuringStateType::Normal, "i");
        let accepting_state = TuringState::new(TuringStateType::Accepting, "a");
        let rejecting_state = TuringState::new(TuringStateType::Rejecting, "r");

        // Create the hash map with the already known states
        let mut name_index_hashmap: HashMap<String, usize> = HashMap::new();
        name_index_hashmap.insert("i".to_string(), 0); // init
        name_index_hashmap.insert("a".to_string(), 1); // accepting
        name_index_hashmap.insert("r".to_string(), 2); // rejecting

        Ok(Self {
            name_index_hashmap,
            states: vec![init_state, accepting_state, rejecting_state],
            k,
        })
    }

    /// Adds a new rule to a state of the machine of the form : `from {transition} to`.
    /// Meaning, a new edge is added to the graph.
    ///
    /// If one of the given state didn't already exists, a [TuringError::UnknownStateError] will be returned.
    pub fn append_rule_state_by_name(
        &mut self,
        from: impl ToString,
        transition: TuringTransition,
        to: impl ToString,
    ) -> Result<(), TuringGraphError> {
        let from = from.to_string();
        let to = to.to_string();

        // Checks if the given number of tapes is correct
        if transition.get_number_of_affected_tapes() != (self.k + 1) {
            return Err(TuringGraphError::IncompatibleTransitionError {
                expected: self.get_k(),
                received: transition.get_number_of_affected_tapes() - 1,
            });
        }
        let from_index = self.name_index_hashmap.get(&from);
        if from_index.is_none() {
            return Err(TuringGraphError::UnknownStateError { state_name: from });
        }
        let from_index = *from_index.unwrap();

        let to_index = self.name_index_hashmap.get(&to);
        if to_index.is_none() {
            return Err(TuringGraphError::UnknownStateError { state_name: to });
        }
        let to_index = *to_index.unwrap();

        self.add_rule_state_ind(from_index, transition, to_index)
    }

    /// Adds a new rule/transition to a state of the machine of the form : `from {transition} to`.
    /// Meaning, a new edge is added to the graph.
    ///
    /// ## Returns
    /// * If everything went correctly : `Ok(())`
    /// * Otherwise, it will return a [TuringError]
    pub fn append_rule_state(
        &mut self,
        from_index: usize,
        transition: TuringTransition,
        to_index: usize,
    ) -> Result<(), TuringGraphError> {
        // Checks if the given correct of number transitions was given
        if transition.chars_write.len() != self.k {
            return Err(TuringGraphError::IncompatibleTransitionError {
                expected: self.k,
                received: transition.chars_write.len(),
            });
        }
        match self.add_rule_state_ind(from_index, transition, to_index) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Adds a new rule to a state of the machine and returns the machine.
    ///
    /// If the given state didn't already exists, the state will be created.
    pub fn append_rule_state_self(
        mut self,
        from: &String,
        transition: TuringTransition,
        to: &String,
    ) -> Result<Self, TuringGraphError> {
        self.append_rule_state_by_name(from, transition, to)?;

        Ok(self)
    }

    /// Adds a new state to the turing machine graph and returns its index. Meaning a new node is added to the graph.
    ///
    /// If the state name already existed then the index of the already existing state is returned.
    pub fn add_state(&mut self, name: impl ToString) -> usize {
        let name = name.to_string();
        // Try to find the index of the state inside the hashmap
        match self.name_index_hashmap.get(&name) {
            // If the index was found, return it
            Some(e) => *e,
            // If not
            None => {
                // Pushes in the vector of states a new state with the given name
                self.states
                    .push(TuringState::new(TuringStateType::Normal, name.to_string()));
                // Adds the index of this state to the hashmap
                self.name_index_hashmap
                    .insert(name.to_string(), self.states.len() - 1);
                // Returns the index of the newly created state
                self.states.len() - 1
            }
        }
    }

    /// Adds a new state to the turing machine graph using variables indexes
    fn add_rule_state_ind(
        &mut self,
        from: usize,
        mut transition: TuringTransition,
        to: usize,
    ) -> Result<(), TuringGraphError> {
        if self.states.len() <= from {
            return Err(TuringGraphError::OutOfRangeStateError {
                accessed_index: from,
                states_len: self.states.len(),
            });
        }
        if self.states.len() <= to {
            return Err(TuringGraphError::OutOfRangeStateError {
                accessed_index: to,
                states_len: self.states.len(),
            });
        }
        // Change transition index
        transition.index_to_state = Some(to);

        let state = self.states.get_mut(from).unwrap();
        state.add_transition(transition)?;

        Ok(())
    }

    /// Returns the state (*node*) at the given index.
    pub fn get_state(&self, pointer: usize) -> Result<&TuringState, TuringGraphError> {
        if self.states.len() <= pointer {
            return Err(TuringGraphError::OutOfRangeStateError {
                accessed_index: pointer,
                states_len: self.states.len(),
            });
        }
        Ok(&self.states[pointer])
    }

    /// Returns the **mutable** state (*node*) at the given index.
    pub fn get_state_mut(&mut self, pointer: usize) -> Result<&mut TuringState, TuringGraphError> {
        if self.states.len() <= pointer {
            return Err(TuringGraphError::OutOfRangeStateError {
                accessed_index: pointer,
                states_len: self.states.len(),
            });
        }
        Ok(&mut self.states[pointer])
    }

    /// Returns the state (*node*) that has the given name.
    pub fn get_state_from_name(
        &self,
        name: impl ToString,
    ) -> Result<&TuringState, TuringGraphError> {
        let name = name.to_string();
        match self.name_index_hashmap.get(&name) {
            Some(index) => self.get_state(*index),
            None => Err(TuringGraphError::UnknownStateError { state_name: name }),
        }
    }

    /// Returns the **mutable** state (*node*) that has the given name.
    pub fn get_state_from_name_mut(
        &mut self,
        name: &String,
    ) -> Result<&mut TuringState, TuringGraphError> {
        match self.name_index_hashmap.get(name) {
            Some(index) => self.get_state_mut(*index),
            None => Err(TuringGraphError::UnknownStateError {
                state_name: name.to_string(),
            }),
        }
    }

    /// Get the transition index between two nodes if it exists.
    pub fn get_transition_indexes_by_name(
        &self,
        n1: impl ToString,
        n2: impl ToString,
    ) -> Result<Vec<usize>, TuringGraphError> {
        let n1 = n1.to_string();
        let n2 = n2.to_string();

        let mut res = vec![];
        // Get n1 and n2 indexes if they exists
        let n1_state = match self.name_index_hashmap.get(&n1) {
            Some(i) => &self.states[*i],
            None => {
                return Err(TuringGraphError::UnknownStateError {
                    state_name: n1.clone(),
                });
            }
        };
        let n2_index = match self.name_index_hashmap.get(&n2) {
            Some(i) => *i,
            None => {
                return Err(TuringGraphError::UnknownStateError {
                    state_name: n2.clone(),
                });
            }
        };

        for (i, t) in n1_state.transitions.iter().enumerate() {
            if t.index_to_state.unwrap() == n2_index {
                res.push(i);
            }
        }

        Ok(res)
    }

    /// Get all the transitions between two nodes.
    pub fn get_transitions_by_index(
        &self,
        n1: usize,
        n2: usize,
    ) -> Result<Vec<&TuringTransition>, TuringGraphError> {
        let mut vec = vec![];
        // Get n1 index
        let n1_state = self.get_state(n1)?;

        // Fetch all transition that go toward n2
        for t in n1_state.transitions.iter() {
            if t.index_to_state.unwrap() == n2 {
                vec.push(t);
            }
        }

        Ok(vec)
    }

    /// Removes **all** the transitions from this state to the given node
    pub fn remove_transitions(
        &mut self,
        from: impl ToString,
        to: impl ToString,
    ) -> Result<(), TuringGraphError> {
        let from = from.to_string();
        let to = to.to_string();

        let (n1_state, n2_index) = self.fetch_n1_state_n2_index(&from, &to)?;

        // Remove all transitions from n1 to n2
        n1_state.remove_transitions(n2_index);
        Ok(())
    }

    /// Removes **all** the transitions from a state to the given node, using the nodes `to` node index.
    pub fn remove_transitions_with_index(
        &mut self,
        from: usize,
        to: usize,
    ) -> Result<(), TuringGraphError> {
        // check that `from` state exists
        let n1_state = self.get_state_mut(from)?;

        // Remove all transitions from n1 to n2
        n1_state.remove_transitions(to);
        Ok(())
    }

    /// Removes all transitions of the form `from {transition} to` using the given parameters.
    ///  
    /// The `transition`'s `index_to_state` field, will not be be taken into account here (as it will be changed with the index of `to` anyways), the rest however is still important.
    ///
    /// Only `from`'s existance will be verified (and will return an error if it does not exists). But `to`'s index can be outside the bounds.
    pub fn remove_transition(
        &mut self,
        from: impl ToString,
        transition: &TuringTransition,
        to: impl ToString,
    ) -> Result<(), TuringGraphError> {
        let from = from.to_string();
        let to = to.to_string();

        let (n1_state, n2_index) = self.fetch_n1_state_n2_index(&from, &to)?;

        let mut trans = transition.clone();
        // In order to make sure it is removed, we change the index to the correct one
        trans.index_to_state = Some(n2_index);
        n1_state.remove_transition(&trans);

        Ok(())
    }

    fn fetch_n1_state_n2_index(
        &mut self,
        from: &String,
        to: &String,
    ) -> Result<(&mut TuringState, usize), TuringGraphError> {
        // Fetch n1 as a state
        let n1_state = self.name_index_hashmap.get(from);
        if n1_state.is_none() {
            return Err(TuringGraphError::UnknownStateError {
                state_name: from.to_string(),
            });
        }

        let n1_state = &mut self.states[*n1_state.unwrap()];

        // Fetch n2 as an index
        let n2_state = self.name_index_hashmap.get(to);
        if n2_state.is_none() {
            return Err(TuringGraphError::UnknownStateError {
                state_name: to.to_string(),
            });
        }
        let n2_index = n2_state.unwrap();

        Ok((n1_state, *n2_index))
    }

    /// Removes a state and **all** mentions of it in **all** transitions of **all** the other states of the TuringMachine using its name.
    pub fn remove_state_with_name(
        &mut self,
        state_name: impl ToString,
    ) -> Result<(), TuringGraphError> {
        let state_name = state_name.to_string();
        // First keep the index for later
        let index = self.name_index_hashmap.get(&state_name);
        if index.is_none() {
            return Err(TuringGraphError::UnknownStateError { state_name });
        }
        let index = *index.unwrap();
        // Use that index to remove that state
        self.remove_state_with_index(index)
    }

    /// Removes a state and **all** mentions of it in **all** transitions of **all** the other states of the TuringMachine using its index.
    fn remove_state_with_index(&mut self, state_index: usize) -> Result<(), TuringGraphError> {
        // if the node is one of the 3 initial nodes, throw an error
        if state_index <= 2 {
            return Err(TuringGraphError::ImmutableStateError {
                state: self
                    .get_state(state_index)
                    .expect("default state present")
                    .clone(),
            });
        }

        // get state name
        let state_name = self.get_state(state_index).cloned()?.name;

        /* Remove references to this state from *all* other nodes transitions */
        let mut states = vec![];
        // Fetch all names that aren't the state we are trying to remove
        for name in self.name_index_hashmap.keys() {
            if name.eq(&state_name) {
                continue;
            }
            states.push(name.clone());
        }

        let mut to_notify_neigh = vec![];

        // Remove the node
        self.states.remove(state_index); // this means that other indexes might have shifted too
        // Collect all values that are gonna change

        let mut prev_val;
        for name in &states {
            prev_val = *self.name_index_hashmap.get_mut(name).unwrap();
            // If this node had a bigger index, we lower it by one in the hashmap (it moved in the `states` vector)
            if prev_val >= state_index {
                to_notify_neigh.push(prev_val);
                let index = *self.name_index_hashmap.get_mut(name).unwrap();
                *self.name_index_hashmap.get_mut(name).unwrap() = index - 1;
                // Save this state for later use
                prev_val = index - 1;
            }

            // Remove all transitions to the removed node
            self.remove_transitions_with_index(prev_val, state_index)?
        }
        for state in &mut self.states {
            for changed_index in &to_notify_neigh {
                state.update_transitions(*changed_index, *changed_index - 1);
            }
        }

        self.name_index_hashmap.remove(&state_name);
        Ok(())
    }

    pub fn get_k(&self) -> usize {
        self.k
    }

    pub fn get_name_index_hashmap(&self) -> &HashMap<String, usize> {
        &self.name_index_hashmap
    }

    pub fn get_states(&self) -> &Vec<TuringState> {
        &self.states
    }
}

impl Display for TuringMachineGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::from("States:\n");

        // Print all states
        for state in &self.states {
            res.push_str(format!("{}: {}\n", state.name, state.state_type).as_str());
        }

        res.push_str("\nTransitions:\n");
        let mut res_tr = String::new();
        // Print all transitions btw states
        for (q1, i1) in &self.name_index_hashmap {
            for (q2, i2) in &self.name_index_hashmap {
                let transitions = self.get_transitions_by_index(*i1, *i2).unwrap();
                if transitions.is_empty() {
                    continue;
                }
                res_tr.push_str(format!("q_{} {} ", q1, '{').as_str());
                let spaces = 3 + q1.len();

                for i in 0..transitions.len() - 1 {
                    res_tr.push_str(
                        format!("{} \n{}| ", transitions.get(i).unwrap(), " ".repeat(spaces))
                            .as_str(),
                    );
                }
                // add last
                res_tr.push_str(format!("{} ", transitions.last().unwrap()).as_str());

                res_tr.push_str(format!("{} q_{};\n\n", "}", q2).as_str());
            }
        }
        if res_tr.is_empty() {
            res.push_str("None");
        } else {
            res.push_str(res_tr.as_str());
        }

        write!(f, "{}", res)
    }
}
