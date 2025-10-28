use std::fmt::Display;
use thiserror::Error;

use crate::turing_tape;

#[derive(Debug, Error)]
pub enum TuringTransitionError {
    #[error("Trying to construct a transition with an incorrect number of arguments: \"{0}\"")]
    TransitionArgsError(String),
    #[error("Tried to create an illegal transition: \"{0}\"")]
    IllegalActionError(String),
}

#[derive(Debug, Clone, PartialEq)]
/// Represents the direction of a movement that the pointer of a tape can take after reading/writing a character
pub enum TuringDirection {
    Left,
    Right,
    None,
}

impl TuringDirection {
    /// Return the integer value of the direction.
    ///
    /// Left values are negatives, right values are positives and none is represented by zero.
    pub fn get_value(&self) -> i8 {
        match self {
            Self::Left => -1,
            Self::Right => 1,
            Self::None => 0,
        }
    }
}

impl Display for TuringDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Left => "L",
                Self::Right => "R",
                Self::None => "N",
            }
        )
    }
}

#[derive(Debug)]
/// A struct representing a transition for a turing machine that has strictly more than **1 tape** :
/// * `a_0, a_1, ..., a_{n-1} -> D_0, b_1, D_1, b_2, D_2, ..., b_{n-1}, D_{n-1}`
/// - With :
///     * `a_i` : The character *i* being read.
///     * `D_i` : Direction to take by taking this transition, see [TuringDirection] for more information.
///     * `b_i` : The character to replace the character *i* with.
pub struct TuringTransition {
    /// The chars that have to be read in order apply the rest of the transition : `a_0,..., a_{n-1}`
    pub chars_read: Vec<char>,
    /// The move to take after writing/reading the character : `D_0`
    pub move_read: TuringDirection,
    /// The character to replace the character just read : `(b_1, D_1),..., (b_{n-1}, D_{n-1})`
    pub chars_write: Vec<(char, TuringDirection)>,
    /// The index of the state to go to after passing through this state.
    pub index_to_state: Option<usize>,
}

impl TuringTransition {
    /// Creates a new [TuringTransitions].
    pub fn new(
        char_read: Vec<char>,
        move_read: TuringDirection,
        chars_read_write: Vec<(char, TuringDirection)>,
    ) -> Self {
        Self {
            chars_read: char_read,
            move_read,
            chars_write: chars_read_write,
            index_to_state: None,
        }
    }

    /// Simplifies the creation of a new [TuringTransition] of the form :
    /// * `a_0, a_1, ..., a_{n-1} -> D_0, b_1, D_1, b_2, D_2, ..., b_{n-1}, D_{n-1}`
    ///
    /// ## Args :
    /// * **chars_read** : The characters that have to be read in order to take this transition : `a_0,..., a_{n-1}`
    /// * **chars_write** : The characters to replace the characters read : `b_1, ..., b_{n-1}`
    /// * **directions** : The directions to move the pointers of the tapes : `D_0, ..., D_{n-1}`
    pub fn create(
        chars_read: Vec<char>,
        chars_write: Vec<char>,
        directions: Vec<TuringDirection>,
    ) -> Result<Self, TuringTransitionError> {
        let mut chars_write_dir: Vec<(char, TuringDirection)> = vec![];
        let move_read = directions.first();

        if move_read.is_none() {
            return Err(TuringTransitionError::TransitionArgsError(
                "At least one direction must be given".to_string(),
            ));
        }
        let move_read = move_read.unwrap().clone();

        if chars_write.len() + 1 != directions.len() {
            return Err(TuringTransitionError::TransitionArgsError("The number of character to write must be equal to the number of directions minus one (for the reading tape)".to_string()));
        }
        if chars_read.len() != directions.len() {
            return Err(TuringTransitionError::TransitionArgsError(
                "The number of characters to read must be equal to the number of given directions"
                    .to_string(),
            ));
        }
        for i in 1..directions.len() {
            chars_write_dir.push((
                *chars_write.get(i - 1).unwrap(),
                directions.get(i).unwrap().clone(),
            ));
        }

        // Check for illegal actions
        let ill_act_error = |c: char,
                             inc_char: char,
                             d: &TuringDirection,
                             inc_dir: &TuringDirection|
         -> Result<(), TuringTransitionError> {
            if inc_char == c && inc_dir == d {
                Err(TuringTransitionError::IllegalActionError(format!(
                    "Detected the couple : (\"{}\", \"{}\"), this could result in going out of bounds of the tape. Change the given direction to None for example.",
                    c, d
                )))
            } else {
                Ok(())
            }
        };

        //  Only applies to the reading tape
        ill_act_error(
            *chars_read.first().unwrap(),
            turing_tape::END_CHAR,
            &move_read,
            &TuringDirection::Right,
        )?;

        //  Applies to all tapes, therefore we need to iterate over all of them

        // check for reading first
        ill_act_error(
            *chars_read.first().unwrap(),
            turing_tape::INIT_CHAR,
            &move_read,
            &TuringDirection::Left,
        )?;
        // then for writting tapes
        for i in 1..chars_read.len() {
            let char_read = chars_read.get(i).unwrap();

            let (char_relacement, char_dir) = chars_write_dir.get(i - 1).unwrap();

            ill_act_error(
                *char_read,
                turing_tape::INIT_CHAR,
                char_dir,
                &TuringDirection::Left,
            )?;

            if *char_read == turing_tape::INIT_CHAR {
                if *char_read != *char_relacement {
                    return Err(TuringTransitionError::IllegalActionError(format!(
                        "Tried to replace a special character ('{}') with another character ('{}') for the writing tape {}",
                        char_read,
                        char_relacement,
                        i - 1
                    )));
                }
            } else if *char_relacement == turing_tape::INIT_CHAR {
                return Err(TuringTransitionError::IllegalActionError(format!(
                    "Tried to replace a normal character ('{}') with a special character ('{}') for the writing tape {}",
                    char_read,
                    char_relacement,
                    i - 1
                )));
            }
        }

        Ok(Self {
            chars_read,
            move_read,
            chars_write: chars_write_dir,
            index_to_state: None,
        })
    }

    /// Returns the number of tapes that are going to be affected by this transition.
    pub fn get_number_of_affected_tapes(&self) -> usize {
        self.chars_write.len() + 1
    }
}

impl Clone for TuringTransition {
    fn clone(&self) -> Self {
        Self {
            chars_read: self.chars_read.clone(),
            move_read: self.move_read.clone(),
            chars_write: self.chars_write.clone(),
            index_to_state: self.index_to_state,
        }
    }
}

impl Display for TuringTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut char_read = String::from(self.chars_read[0]);
        for i in 1..self.chars_read.len() {
            char_read.push_str(format!(", {}", self.chars_read[i]).as_str());
        }

        let mut char_written = format!("{}", self.move_read);

        for (c, dir) in &self.chars_write {
            char_written.push_str(format!(", {}, {}", c, dir).as_str());
        }

        write!(f, "{} -> {}", char_read, char_written)
    }
}

impl PartialEq for TuringTransition {
    /// Checks if two [TuringTransition] are equivalent. Note that the `index_to_state` field is not part of this comparison.
    fn eq(&self, other: &Self) -> bool {
        self.chars_read == other.chars_read
            && self.move_read == other.move_read
            && self.chars_write == other.chars_write
    }
}
