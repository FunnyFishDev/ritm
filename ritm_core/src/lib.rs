use crate::{
    turing_graph::{TuringGraph, TuringState},
    turing_machine::TuringMachines,
    turing_transition::TuringTransition,
};

pub mod turing_graph;

pub mod turing_machine;

pub mod turing_tape;

pub mod turing_index;

pub mod turing_parser;

pub mod turing_transition;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EmptyState;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EmptyTransition;

impl TuringState for EmptyState {
    fn new_init() -> Self {
        Self
    }

    fn new_accepting() -> Self {
        Self
    }
}

impl TuringTransition for EmptyTransition {}

pub type SimpleTuringGraph = TuringGraph<EmptyState, EmptyTransition>;

pub type SimpleTuringMachine = TuringMachines<EmptyState, EmptyTransition>;
