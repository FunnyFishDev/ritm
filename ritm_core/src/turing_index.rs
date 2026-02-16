use crate::turing_transition::TransitionsInfo;
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

#[derive(Debug, Clone, PartialEq)]
pub struct TuringTransitionIndex {
    pub source_id: TuringStateIndex,
    pub transition_id: TransitionId,
    pub target_id: TuringStateIndex,
}

impl<S, F, T> From<(S, F, T)> for TuringTransitionIndex
where
    S: Into<TuringStateIndex>,
    F: Into<TransitionId>,
    T: Into<TuringStateIndex>,
{
    fn from(value: (S, F, T)) -> Self {
        Self {
            source_id: value.0.into(),
            transition_id: value.1.into(),
            target_id: value.2.into(),
        }
    }
}

impl Display for TuringTransitionIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({} -> {} -> {})",
            self.source_id, self.transition_id, self.target_id
        )
    }
}

/// Can be used to try to find a transition in the graph
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionId {
    /// Represents the index value of the transition to get in the list of transitions present.
    ID(usize),
    /// Represents the value of the transition to try to get.
    Value(TransitionsInfo),
}

impl TransitionId {
    pub fn from(index: impl Into<TransitionId>) -> Self {
        index.into()
    }
}

impl Display for TransitionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TransitionId::ID(val) => val.to_string(),
                TransitionId::Value(val) => val.to_string(),
            }
        )
    }
}

impl<T> From<T> for TransitionId
where
    T: Into<TransitionsInfo>,
{
    fn from(value: T) -> Self {
        TransitionId::Value(value.into())
    }
}

impl From<&TransitionId> for TransitionId {
    fn from(value: &TransitionId) -> Self {
        value.clone()
    }
}

impl From<usize> for TransitionId {
    fn from(value: usize) -> Self {
        TransitionId::ID(value)
    }
}

impl From<&usize> for TransitionId {
    fn from(value: &usize) -> Self {
        (*value).into()
    }
}
