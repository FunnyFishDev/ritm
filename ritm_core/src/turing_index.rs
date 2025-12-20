use crate::turing_transition::TuringTransitionInfo;
use std::fmt::Display;

/// Can be used to try to find a state in the graph.
#[derive(Debug, Clone, PartialEq)]
pub enum TuringStateIndex {
    /// Represents the index value of the state to get.
    ID(usize),
    /// Represents the name of the state to get.
    Value(String),
}

impl TuringStateIndex {
    pub fn from(index: impl Into<TuringStateIndex>) -> Self {
        index.into()
    }
}

impl Display for TuringStateIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TuringStateIndex::ID(val) => val.to_string(),
                TuringStateIndex::Value(val) => val.to_string(),
            }
        )
    }
}

impl From<String> for TuringStateIndex {
    fn from(value: String) -> Self {
        TuringStateIndex::Value(value)
    }
}
impl From<&String> for TuringStateIndex {
    fn from(value: &String) -> Self {
        value.to_string().into()
    }
}

impl From<&str> for TuringStateIndex {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<usize> for TuringStateIndex {
    fn from(value: usize) -> Self {
        TuringStateIndex::ID(value)
    }
}

impl From<&usize> for TuringStateIndex {
    fn from(value: &usize) -> Self {
        (*value).into()
    }
}

/// Can be used to try to find a transition in the graph
#[derive(Debug, Clone, PartialEq)]
pub enum TuringTransitionIndex {
    /// Represents the index value of the transition to get in the list of transitions present.
    ID(usize),
    /// Represents the value of the transition to try to get.
    Value(TuringTransitionInfo),
}

impl TuringTransitionIndex {
    pub fn from(index: impl Into<TuringTransitionIndex>) -> Self {
        index.into()
    }
}

impl Display for TuringTransitionIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TuringTransitionIndex::ID(val) => val.to_string(),
                TuringTransitionIndex::Value(val) => val.to_string(),
            }
        )
    }
}

impl<T> From<T> for TuringTransitionIndex
where
    T: Into<TuringTransitionInfo>,
{
    fn from(value: T) -> Self {
        TuringTransitionIndex::Value(value.into())
    }
}

impl From<&TuringTransitionIndex> for TuringTransitionIndex {
    fn from(value: &TuringTransitionIndex) -> Self {
        value.clone()
    }
}

impl From<usize> for TuringTransitionIndex {
    fn from(value: usize) -> Self {
        TuringTransitionIndex::ID(value)
    }
}

impl From<&usize> for TuringTransitionIndex {
    fn from(value: &usize) -> Self {
        (*value).into()
    }
}
