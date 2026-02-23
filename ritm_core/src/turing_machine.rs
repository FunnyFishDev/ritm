use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
};

use thiserror::Error;

use crate::{
    turing_graph::{TuringGraph, TuringGraphError, TuringState, TuringStateInfo, TuringStateType},
    turing_tape::{TuringTape, TuringTapeError},
    turing_transition::{TransitionsInfo, TuringTransition, TuringTransitionError},
};

#[derive(Debug, Error)]
pub enum TuringMachineError {
    #[error("Encountered a graph error : {0}")]
    GraphError(#[from] TuringGraphError),
    #[error("Encountered a transition error : {0}")]
    TransitionError(#[from] TuringTransitionError),
    #[error("Encountered a tape error : {0}")]
    TapeError(#[from] TuringTapeError),
}

#[derive(Clone, Debug, PartialEq)]
/// Represents the different mode a turing machine can have during it's execution
pub enum Mode {
    /// Explores all possible paths (and possibilities using backtracking) until an accepting state is found or no path is left is to take.
    SaveAll, // May god bless your ram
    /// Stops after the specified amount of iteration is reached even if the execution is not over.
    StopAfter(usize),
    /// Stops after meeting the first rejecting state or when the execution is blocked, even if backtracking is possible
    StopFirstReject,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Mode::SaveAll => "Saves All and does a full exploration".to_string(),
                Mode::StopAfter(val) => format!("Stops After {} iterations", val),
                Mode::StopFirstReject => "Stops after the First Reject".to_string(),
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct SavedState {
    /// The index of the saved state
    pub saved_state_index: usize,
    /// A stack containing all the indexes of the transitions left to take
    pub next_transitions: VecDeque<(usize, usize)>,
    /// The value of the [`TuringTape`] when they were saved
    pub saved_tapes: Vec<TuringTape>,
    /// The value of the iteration that was saved.
    pub iteration: usize,
}

#[derive(Debug)]
pub struct TuringMachine<S, T>
where
    S: TuringState,
    T: TuringTransition,
{
    /// The turing machine graph that will execute a word
    graph: TuringGraph<S, T>,
    data: IterationData,
    /// The current number of iterations already done
    iteration: usize,
    /// Copy of the iteration step returned (if any).
    last_iteration: Option<TuringExecutionSteps>,
    /// Checks wether or not the iteration is over or not
    is_over: bool,
}

#[derive(Debug)]
pub struct IterationData {
    /// A vector containing all writting rubbons. If k=0, contains all least one writing tape replacing the reading tape.
    tapes: Vec<TuringTape>,
    /// The current word to read
    word: String,
    /// The ID of the current state of the turing machine
    state_pointer: usize,
    /// Represents if the iteration has started or got reset.
    is_first_state: bool,
    /// A stack representing the memory of the exploration of this turing machine.
    memory: VecDeque<SavedState>,
    /// Represents the mode used for the execution of this turing machine
    mode: Mode,
    /// First index is the ID to the state that the transition leads to from the current state pointer.
    /// The second one is the index of the transition taken between the two.
    backtracked_info: Option<(usize, usize)>,
}

impl<S, T> TuringMachine<S, T>
where
    S: TuringState,
    T: TuringTransition,
{
    // Create a new turing machine for a given word.
    pub fn new(
        mt: TuringGraph<S, T>,
        word: String,
        mode: Mode,
    ) -> Result<Self, TuringMachineError> {
        let mut s = TuringMachine {
            data: IterationData {
                state_pointer: 0,
                tapes: {
                    let mut v = vec![TuringTape::new(mt.get_k() != 0)];
                    // Creates k tapes
                    for _ in 0..mt.get_k() {
                        v.push(TuringTape::new(false));
                    }
                    v
                },
                word: word.clone(),
                is_first_state: true,
                memory: VecDeque::new(),
                mode,
                backtracked_info: None,
            },
            graph: mt,
            iteration: 0,
            last_iteration: None,
            is_over: false,
        };
        // Add the word to the reading tape
        s.data
            .tapes
            .first_mut()
            .expect("Always one tape present")
            .feed_word(word, s.graph.get_k() != 0)?;

        Ok(s)
    }

    /// Adds a new [SavedState] to the front of the memory stack.
    fn push_to_memory_stack(&mut self, to_save: SavedState) {
        self.get_memory_mut().push_front(to_save);
    }

    /// Resets the turing machine to its initial state and re-feeds it the current stored word.
    pub fn reset(&mut self) {
        let word = self.get_word().clone();
        self.reset_word(&word).unwrap();
    }

    /// Resets the turing machine to its initial state and feeds it the given word.
    pub fn reset_word(&mut self, word: &String) -> Result<(), TuringMachineError> {
        // Reset reading tape
        self.data
            .tapes
            .first_mut()
            .expect("Always one tape present")
            .feed_word(word, self.graph.get_k() != 0)?;

        self.set_word(word);

        // Reset writing tapes
        for i in 1..self.data.tapes.len() {
            self.data.tapes[i] = TuringTape::new(false);
        }

        // Reset state pointers
        self.set_state_pointer(0);

        // Reset first iteration
        self.set_first_iteration(true);

        // Sets the number of iterations to 0
        self.set_iteration(0);

        // Reset backtracking info
        self.set_backtracking_info(None);

        self.set_last_step(None);

        self.set_is_over(false);

        // And clear memory
        self.get_memory_mut().clear();

        Ok(())
    }
    /// Changes the current execution mode of the turing machine.
    pub fn set_mode(&mut self, mode: &Mode) {
        // Change mode
        self.data.mode = mode.clone();
    }

    /// Gets the path to the accepting state if any exists.
    /// In other terms, it will only store the steps that will lead to an accepting path without any backtracking.
    /// Always resets the execution before starting (but doesn't reset after ending).
    /// ## Returns
    /// [None] if no path to an accepting state is found.
    /// [Some] containing a [Vec] of [TuringExecutionSteps] leading to the accepting state.
    ///
    ///
    /// ## Infinite iterations problems
    /// **Beware** that this function will loop forever **if** the related turing machine graph loops for the given input.
    /// In order to prevent this, it is possible to supply a function that will be called before every iteration to check if it is allowed to continue it's execution.
    /// Another mitigation would be to simply change the execution mode of this turing machine.
    pub fn get_path_to_accept<F>(
        &mut self,
        mut exit_condition: F,
    ) -> Option<Vec<TuringExecutionSteps>>
    where
        F: FnMut() -> bool,
    {
        self.reset();
        let mut path = Vec::<TuringExecutionSteps>::new();
        let mut last_step_type = None;
        for step in &mut *self {
            if !exit_condition() {
                return None;
            }
            last_step_type = Some(step.get_current_state().get_type().clone());
            match &step {
                TuringExecutionSteps::FirstIteration {
                    init_state: _,
                    init_tapes: _,
                } => {
                    path.push(step);
                }
                TuringExecutionSteps::TransitionTaken {
                    previous_state: _,
                    reached_state: _,
                    state_pointer: _,
                    transition_index: _,
                    transition_taken: _,
                    tapes: _,
                    iteration: _,
                } => {
                    path.push(step);
                }
                TuringExecutionSteps::Backtracked {
                    previous_state: _,
                    reached_state: _,
                    state_pointer: _,
                    tapes: _,
                    iteration: _,
                    backtracked_iteration,
                } => {
                    // Pop stack until we find the iteration we backtracked to
                    while path.last().unwrap().get_nb_iterations() != *backtracked_iteration {
                        path.pop();
                    }
                }
            }
        }
        // If the last step did not result in an accepting state,
        // then we know that no path results in an accepting state.
        if let Some(t) = last_step_type
            && TuringStateType::Accepting != t
        {
            return None;
        }

        Some(path)
    }
}

impl<S, T> TuringMachine<S, T>
where
    S: TuringState,
    T: TuringTransition,
{
    /// Gets *reference* of the stored turing machine graph.
    pub fn graph_ref(&self) -> &TuringGraph<S, T> {
        &self.graph
    }

    /// Gets *mutable reference* of the stored turing machine graph.
    pub fn graph_mut(&mut self) -> &mut TuringGraph<S, T> {
        &mut self.graph
    }

    /// Gets the stored turing machine graph.
    ///
    /// This will free the turing machine since it will drop the ownership
    pub fn graph(self) -> TuringGraph<S, T> {
        self.graph
    }

    /// Gets the current state pointer of this struct.
    pub fn get_state_pointer(&self) -> usize {
        self.data.state_pointer
    }

    /// Sets a new value to the state pointer.
    fn set_state_pointer(&mut self, new_val: usize) {
        self.data.state_pointer = new_val;
    }

    /// Gets the word that was feed to this machine.
    pub fn get_word(&self) -> &String {
        &self.data.word
    }

    /// Gets the word that was feed to this machine.
    fn set_word(&mut self, word: &String) {
        self.data.word = word.to_string();
    }

    /// Checks if the current iteration is the first iteration or not.
    fn is_first_iteration(&mut self) -> bool {
        self.data.is_first_state
    }

    /// Sets the state of this turing machine to be considered or not its first iteration.
    fn set_first_iteration(&mut self, set: bool) {
        self.data.is_first_state = set;
    }

    /// Fetches the mode of the iterator.
    pub fn get_mode(&self) -> &Mode {
        &self.data.mode
    }

    /// Get the **mutable** stack containing all the [SavedState].
    fn get_memory_mut(&mut self) -> &mut VecDeque<SavedState> {
        &mut self.data.memory
    }

    /// Get the reference to the stack containing all the [SavedState].
    pub fn get_memory(&self) -> &VecDeque<SavedState> {
        &self.data.memory
    }

    fn get_backtracking_info(&self) -> Option<(usize, usize)> {
        self.data.backtracked_info
    }

    fn set_backtracking_info(&mut self, val: Option<(usize, usize)>) {
        self.data.backtracked_info = val;
    }

    fn set_iteration(&mut self, val: usize) {
        self.iteration = val;
    }

    pub fn get_iteration(&self) -> usize {
        self.iteration
    }

    /// Returns the last step that was returned.
    pub fn get_last_step(&self) -> &Option<TuringExecutionSteps> {
        &self.last_iteration
    }

    fn set_last_step(&mut self, step: Option<TuringExecutionSteps>) {
        self.last_iteration = step;
    }

    /// Checks if the iteration is over or not
    pub fn is_over(&self) -> bool {
        self.is_over
    }

    fn set_is_over(&mut self, val: bool) {
        self.is_over = val;
    }
}

#[derive(Clone, Debug)]
pub enum TuringExecutionSteps {
    FirstIteration {
        /// A clone of the initial state
        init_state: TuringStateInfo,
        /// A clone representing the initial state of the tapes.
        init_tapes: Vec<TuringTape>,
    },
    TransitionTaken {
        /// A clone of the state that was just left
        previous_state: TuringStateInfo,

        /// Triplets to identify the transition taken in the graph
        transition_index: (usize, usize, usize),

        // TODO: Remove useless values
        /// A clone of the state that was just reached
        reached_state: TuringStateInfo,
        /// The index of the currently reached state
        state_pointer: usize,

        /// A clone of the transition that was just taken
        transition_taken: TransitionsInfo,
        /// A clone representing the current state of the tapes after taking that transition.
        tapes: Vec<TuringTape>,
        /// The current number of iterations already done
        iteration: usize,
    },
    Backtracked {
        /// A clone of the state that was just left
        previous_state: TuringStateInfo,
        /// A clone of the state that was backtracked to
        reached_state: TuringStateInfo,
        /// The index of the currently reached state
        state_pointer: usize,
        /// A clone representing the current state of the tapes after backtracking.
        tapes: Vec<TuringTape>,
        /// The current number of iterations already done
        iteration: usize,
        /// The number of the iteration that was bactracked to
        backtracked_iteration: usize,
    },
}

impl<S, T> Iterator for &mut TuringMachine<S, T>
where
    S: TuringState,
    T: TuringTransition,
{
    type Item = TuringExecutionSteps;

    fn next(&mut self) -> Option<Self::Item> {
        // Get next step
        let next_step = next_iteration(self);
        if let Some(step) = next_step {
            // Save & return it
            self.set_last_step(Some(step.clone()));

            Some(step)
        } else {
            self.set_is_over(true);
            None
        }
    }
}

fn next_iteration<S, T>(tm: &mut TuringMachine<S, T>) -> Option<TuringExecutionSteps>
where
    S: TuringState,
    T: TuringTransition,
{
    let prev_iter = tm.get_iteration();

    if let Mode::StopAfter(nb) = tm.get_mode()
        && *nb == prev_iter
    {
        return None;
    }

    // Increment nb of iterations already treated
    tm.set_iteration(prev_iter + 1);

    // Fetch the current state
    let curr_state = tm
        .graph_ref()
        .get_state(tm.get_state_pointer())
        .unwrap()
        .get_info()
        .clone();

    let mut transition_index_taken = None;

    // If this iteration is a follow up to a backtracking
    // we simply take the index found at the previous iteration
    if let Some(bracktrack_transition_index) = tm.get_backtracking_info() {
        tm.set_backtracking_info(None);
        transition_index_taken = Some(bracktrack_transition_index)
    } else {
        if tm.is_first_iteration() {
            tm.set_first_iteration(false);

            return Some(TuringExecutionSteps::FirstIteration {
                init_state: curr_state.clone(),
                init_tapes: tm.data.tapes.clone(),
            });
        }

        /* Checks if the state is accepting */
        if let TuringStateType::Accepting = curr_state.get_type() {
            // The iteration is over
            return None;
        }

        // if it's normal or rejecting

        // If one of the transition condition is true,
        // Get all current char read by **all** tapes
        let mut char_vec = Vec::new();
        for tape in &mut tm.data.tapes {
            char_vec.push(tape.read_curr_char());
        }

        let mut next_transitions = VecDeque::from(
            tm.graph_ref()
                .get_valid_transitions_indexes(curr_state.get_id(), char_vec)
                .expect("state present"),
        );

        // If no transitions can be provided or the current state is rejecting,
        // we reached a *dead end*, go back in the exploration if possible (i.e. backtrack)
        if next_transitions.is_empty() {
            if let Mode::StopFirstReject = tm.get_mode() {
                return None;
            }
            // If there are no saved state, this means the backtracking is over, and the execution too
            if tm.get_memory_mut().is_empty() {
                return None;
            }

            // While the memory still has a state saved
            while !tm.get_memory_mut().is_empty() {
                {
                    let saved_state = tm.get_memory_mut().front_mut().unwrap();

                    // Get the next transition to take
                    if let Some(t_i) = saved_state.next_transitions.pop_front() {
                        transition_index_taken = Some(t_i);
                    } else {
                        // If no transition is left to take for this state, we move on to the next one and remove it
                        tm.get_memory_mut().pop_front();
                        continue;
                    }
                }
                // obliged to clone because of the mutable nature
                let saved_state = tm.get_memory_mut().front().unwrap().clone();

                // Go back to the state
                tm.set_state_pointer(saved_state.saved_state_index);

                // Change the context for the reading and writing tapes
                tm.data.tapes = saved_state.saved_tapes;

                // Save the index of the transition found for the next call to `.next()`
                tm.set_backtracking_info(transition_index_taken);

                // If the saved state has no more transitions, it can already be removed
                if saved_state.next_transitions.is_empty() {
                    tm.get_memory_mut().pop_front();
                }

                // Return backtracking info
                return Some(TuringExecutionSteps::Backtracked {
                    previous_state: curr_state.clone(),
                    reached_state: tm
                        .graph_ref()
                        .get_state(saved_state.saved_state_index)
                        .unwrap()
                        .get_info()
                        .clone(),
                    tapes: tm.data.tapes.clone(),
                    iteration: prev_iter,
                    state_pointer: tm.get_state_pointer(),
                    backtracked_iteration: saved_state.iteration,
                });
            }
        }
        // If there are more than 1 transition possible at a time, it means we are in a non deterministic situation.
        // We must save the current state in order to explore all path.
        else if next_transitions.len() >= 2 {
            // take the first transition, save the rest

            transition_index_taken = Some(next_transitions.pop_front().unwrap());

            let to_save = SavedState {
                saved_state_index: tm.get_state_pointer(),
                next_transitions,
                saved_tapes: tm.data.tapes.clone(),
                iteration: prev_iter - 1,
            };

            tm.push_to_memory_stack(to_save);
        } else if next_transitions.len() == 1 {
            transition_index_taken = Some(next_transitions[0]);
        }
    }

    // if a viable transition was found
    let Some((to, index)) = transition_index_taken else {
        // otherwise it's also the end
        return None;
    };
    // Take this transition
    let transition = tm.graph_ref().get_transitions_hashmap()[&(curr_state.get_id(), to)][index]
        .info
        .clone();
    // Apply the transition
    let transition_taken = transition.clone();

    match transition {
        // If the machine only has one tape, it means that the reading tape also can be modified :
        TransitionsInfo::OneTape(transition) => {
            tm.data
                .tapes
                .first_mut()
                .expect("always one tape present")
                .try_apply_transition(
                    transition.chars_read,
                    transition.replace_with,
                    &transition.move_pointer,
                )
                .expect("no errors with graph transition");
        }
        // Else the reading tape can only move and the writing tapes are modified
        TransitionsInfo::MultipleTapes(transition) => {
            tm.data
                .tapes
                .first_mut()
                .expect("one present")
                .try_apply_transition(
                    transition.chars_read[0],
                    transition.chars_read[0], // Replace by self
                    &transition.move_read,
                )
                .expect("no issues with graph transition");
            // to the write ribbons
            for i in 0..tm.graph_ref().get_k() {
                tm.data.tapes[i + 1]
                    .try_apply_transition(
                        transition.chars_read[i + 1],
                        transition.chars_write[i].0,
                        &transition.chars_write[i].1,
                    )
                    .expect("no errors after transitions");
            }
        }
    };

    // Move to the next state
    tm.set_state_pointer(to);

    Some(TuringExecutionSteps::TransitionTaken {
        transition_index: (curr_state.get_id(), index, to),
        previous_state: curr_state.clone(),
        reached_state: tm
            .graph_ref()
            .get_state(tm.get_state_pointer())
            .unwrap()
            .get_info()
            .clone(),
        transition_taken,
        tapes: tm.data.tapes.clone(),
        iteration: prev_iter,
        state_pointer: tm.get_state_pointer(),
    })
}

impl Display for TuringExecutionSteps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TuringExecutionSteps::FirstIteration {
                init_state,
                init_tapes,
            } => {
                let mut write_str_rib = init_tapes[0].to_string();
                for writing_tape in init_tapes.iter().skip(1) {
                    write_str_rib.push_str(format!("\n{}", writing_tape).as_str());
                }

                write!(
                    f,
                    "* Initial state : {init_state}\n* Tapes:\n{write_str_rib}",
                )
            }
            TuringExecutionSteps::TransitionTaken {
                previous_state,
                reached_state,
                transition_index: _,
                transition_taken,
                tapes,
                iteration: _,
                state_pointer: _,
            } => {
                let mut write_str_rib = tapes[0].to_string();
                for writing_tape in tapes.iter().skip(1) {
                    write_str_rib.push_str(format!("\n{}", writing_tape).as_str());
                }
                write!(
                    f,
                    "* Left state : {previous_state}\n* Current state : {reached_state}\n* Took the following transition : {transition_taken}\n* Tapes:\n{write_str_rib}",
                )
            }
            TuringExecutionSteps::Backtracked {
                previous_state,
                reached_state,
                tapes,
                iteration: _,
                state_pointer: _,
                backtracked_iteration,
            } => {
                let mut write_str_rib = tapes[0].to_string();
                for writing_tape in tapes.iter().skip(1) {
                    write_str_rib.push_str(format!("\n{}", writing_tape).as_str());
                }

                write!(
                    f,
                    "* Backtracked from : {previous_state}\n* To  : {reached_state}(back to iteration: {backtracked_iteration})\n* Tapes:\n{write_str_rib}"
                )
            }
        }
    }
}

impl TuringExecutionSteps {
    pub fn get_current_state(&self) -> &TuringStateInfo {
        match self {
            TuringExecutionSteps::FirstIteration {
                init_state,
                init_tapes: _,
            } => init_state,
            TuringExecutionSteps::TransitionTaken {
                previous_state: _,
                reached_state,
                state_pointer: _,
                transition_index: _,
                transition_taken: _,
                tapes: _,
                iteration: _,
            } => reached_state,
            TuringExecutionSteps::Backtracked {
                previous_state: _,
                reached_state,
                state_pointer: _,
                tapes: _,
                iteration: _,
                backtracked_iteration: _,
            } => reached_state,
        }
    }

    pub fn get_previous_state(&self) -> Option<&TuringStateInfo> {
        match self {
            TuringExecutionSteps::FirstIteration {
                init_state: _,
                init_tapes: _,
            } => None,
            TuringExecutionSteps::TransitionTaken {
                previous_state,
                reached_state: _,
                state_pointer: _,
                transition_index: _,
                transition_taken: _,
                tapes: _,
                iteration: _,
            } => Some(previous_state),
            TuringExecutionSteps::Backtracked {
                previous_state,
                reached_state: _,
                state_pointer: _,
                tapes: _,
                iteration: _,
                backtracked_iteration: _,
            } => Some(previous_state),
        }
    }

    pub fn get_nb_iterations(&self) -> usize {
        match self {
            TuringExecutionSteps::FirstIteration {
                init_state: _,
                init_tapes: _,
            } => 0,
            TuringExecutionSteps::TransitionTaken {
                previous_state: _,
                reached_state: _,
                state_pointer: _,
                transition_index: _,
                transition_taken: _,
                tapes: _,
                iteration,
            } => *iteration,
            TuringExecutionSteps::Backtracked {
                previous_state: _,
                reached_state: _,
                state_pointer: _,
                tapes: _,
                iteration,
                backtracked_iteration: _,
            } => *iteration,
        }
    }

    pub fn get_state_pointer(&self) -> usize {
        match self {
            TuringExecutionSteps::FirstIteration {
                init_state: _,
                init_tapes: _,
            } => 0,
            TuringExecutionSteps::TransitionTaken {
                previous_state: _,
                reached_state: _,
                state_pointer,
                transition_index: _,
                transition_taken: _,
                tapes: _,
                iteration: _,
            } => *state_pointer,
            TuringExecutionSteps::Backtracked {
                previous_state: _,
                reached_state: _,
                state_pointer,
                tapes: _,
                iteration: _,
                backtracked_iteration: _,
            } => *state_pointer,
        }
    }

    pub fn get_tapes(&self) -> &Vec<TuringTape> {
        match self {
            TuringExecutionSteps::FirstIteration {
                init_state: _,
                init_tapes,
            } => init_tapes,
            TuringExecutionSteps::TransitionTaken {
                previous_state: _,
                reached_state: _,
                state_pointer: _,
                transition_index: _,
                transition_taken: _,
                tapes,
                iteration: _,
            } => tapes,
            TuringExecutionSteps::Backtracked {
                previous_state: _,
                reached_state: _,
                state_pointer: _,
                tapes,
                iteration: _,
                backtracked_iteration: _,
            } => tapes,
        }
    }
}
