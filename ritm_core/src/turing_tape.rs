use std::fmt::{Debug, Display};

use thiserror::Error;

use crate::turing_transition::TuringDirection;

/// Represents the initial character stored at the start of every tape
pub const INIT_CHAR: char = 'ç';

/// Represents the blank character in a tape
pub const BLANK_CHAR: char = '_';

/// Represents the character placed after the content in a [`TuringReadingTape`]
pub const END_CHAR: char = '$';

#[derive(Debug, Error)]
pub enum TuringTapeError {
    #[error(
        "Tried to replace the special character \'{special_char}\' with a normal character \'{replacement_char}\'"
    )]
    SpecialCharReplacementError {
        special_char: char,
        replacement_char: char,
    },
    #[error(
        "Tried to replace the character \'{og_char}\' with a special character \'{special_char}\'"
    )]
    SpecialCharPlacementError { og_char: char, special_char: char },
    #[error(
        "The given input \"{word}\" contains the following illegal character : \'{illegal_char}\'"
    )]
    IllegalInputError { word: String, illegal_char: char },
    #[error(
        "A transition tried to move a pointer out of the tape. Tried to access {accessed_index} but the tape has a size of {tape_size}"
    )]
    OutofRangeError {
        accessed_index: usize,
        tape_size: usize,
    },
}

/// A trait used to implement Turing tapes
pub trait TuringTape: Display + Clone {
    /// Creates a new [TuringTape]
    fn new() -> Self;

    /// Tries to apply the transition to the given tape.
    ///
    /// The transition is applied if the character being pointed by the head of this tape is the same as the given `if_read` character.
    ///
    /// ## Returns
    /// A [bool] if everything went correctly :
    /// * `Some(true)` if the transition went smoothly.
    /// * `Some(false)` if the transition could not be taken.
    ///
    /// A [TuringError] if an error happened, like for example, it was not possible to move at the given direction. Or if a special character (like [INIT_CHAR], [END_CHAR]) is used to replace a *non* special one.
    fn try_apply_transition(
        &mut self,
        if_read: char,
        replace_by: char,
        move_to: &TuringDirection,
    ) -> Result<bool, TuringTapeError>;

    /// Returns the current character being read by the tape
    fn read_curr_char(&self) -> char;

    /// Returns the vector of char as stored by the tape
    fn get_contents(&self) -> &Vec<char>;

    /// Returns the index of the char being pointed by the tape
    fn get_pointer(&self) -> usize;
}

#[derive(Debug, Clone)]
/// Represents a tape made to write and read characters.
pub struct TuringWritingTape {
    chars_vec: Vec<char>,
    pointer: usize,
}

#[derive(Debug, Clone)]
/// Represents a turing made to store and read a word but cannot modify it.
pub struct TuringReadingTape {
    chars_vec: Vec<char>,
    pointer: usize,
}

impl TuringTape for TuringWritingTape {
    /// Creates a new [TuringWritingTape]
    fn new() -> Self {
        Self {
            chars_vec: vec![INIT_CHAR, BLANK_CHAR],
            pointer: 0,
        }
    }

    fn try_apply_transition(
        &mut self,
        if_read: char,
        replace_by: char,
        move_to: &TuringDirection,
    ) -> Result<bool, TuringTapeError> {
        // if the correct symbol was read
        if self.chars_vec[self.pointer] == if_read {
            let new_pointer = (self.pointer as isize) + (move_to.get_value() as isize);

            if new_pointer < 0 {
                return Err(TuringTapeError::OutofRangeError {
                    accessed_index: new_pointer as usize,
                    tape_size: self.chars_vec.len(),
                });
            }
            // In a write tape, we have an *infinite size*, so we can simulate this by adding when needed a new empty char
            if new_pointer >= self.chars_vec.len() as isize {
                self.chars_vec.push('_');
            }

            check_replacement_validity(self.chars_vec[self.pointer], replace_by)?;

            // Replace the current char read
            self.chars_vec[self.pointer] = replace_by;

            // Move to the new position
            self.pointer = new_pointer as usize;
            return Ok(true);
        }
        Ok(false)
    }

    fn read_curr_char(&self) -> char {
        self.chars_vec[self.pointer]
    }

    fn get_contents(&self) -> &Vec<char> {
        &self.chars_vec
    }

    fn get_pointer(&self) -> usize {
        self.pointer
    }
}

impl TuringTape for TuringReadingTape {
    /// Creates a new [TuringReadingTape] only containing the [INIT_CHAR] and [END_CHAR].
    fn new() -> Self {
        Self {
            chars_vec: vec![INIT_CHAR, END_CHAR],
            pointer: 0,
        }
    }

    fn try_apply_transition(
        &mut self,
        if_read: char,
        _: char,
        move_to: &TuringDirection,
    ) -> Result<bool, TuringTapeError> {
        // if the correct symbol was read
        if self.chars_vec[self.pointer] == if_read {
            // Move to the new position
            let new_pointer = (self.pointer as isize) + (move_to.get_value() as isize);

            // If the pointer points out of the bounds of the reading tape
            if new_pointer < 0 || new_pointer >= self.chars_vec.len() as isize {
                return Err(TuringTapeError::OutofRangeError {
                    accessed_index: new_pointer as usize,
                    tape_size: self.chars_vec.len(),
                });
            }
            self.pointer = new_pointer as usize;
            return Ok(true);
        }
        Ok(false)
    }

    fn read_curr_char(&self) -> char {
        self.chars_vec[self.pointer]
    }

    fn get_contents(&self) -> &Vec<char> {
        &self.chars_vec
    }

    fn get_pointer(&self) -> usize {
        self.pointer
    }
}

impl TuringReadingTape {
    /// Feed a word into the reading tape, and also adds [`INIT_CHAR`] and [`END_CHAR`] to the extremities of it
    pub fn feed_word(&mut self, word: String) -> Result<(), TuringTapeError> {
        check_word_validity(&word)?;

        self.chars_vec.clear();
        self.chars_vec.push(INIT_CHAR);
        for ch in word.chars() {
            self.chars_vec.push(ch);
        }
        self.chars_vec.push(END_CHAR);
        self.pointer = 0;
        Ok(())
    }
}

fn check_replacement_validity(og_char: char, new_char: char) -> Result<(), TuringTapeError> {
    if og_char == new_char {
        return Ok(());
    }

    if new_char == INIT_CHAR || new_char == END_CHAR {
        return Err(TuringTapeError::SpecialCharPlacementError {
            og_char,
            special_char: new_char,
        });
    }

    if og_char == INIT_CHAR || new_char == END_CHAR {
        return Err(TuringTapeError::SpecialCharReplacementError {
            special_char: og_char,
            replacement_char: new_char,
        });
    }

    Ok(())
}

impl Display for TuringReadingTape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            tape_to_string(&self.chars_vec, self.pointer, false)
        )
    }
}

/// Turns a character vector into an easy string to read for humans, displaying with an arrow the current char being pointed
fn tape_to_string(chars_vec: &[char], pointer: usize, is_inf: bool) -> String {
    let mut res: String = String::from("[");
    let mut pointing: String = String::from(" ");

    for (count, c) in chars_vec.iter().enumerate() {
        res.push_str(&format!("{c},"));
        if count == pointer {
            pointing.push_str("↑ ");
        } else {
            pointing.push_str("  ");
        }
    }

    res.pop();
    if is_inf {
        res += ",...";
    }
    res += "]\n";

    res.push_str(&pointing);
    res
}

impl Display for TuringWritingTape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", tape_to_string(&self.chars_vec, self.pointer, true))
    }
}

fn check_word_validity(word: &String) -> Result<(), TuringTapeError> {
    let forbidden_chars = vec![INIT_CHAR, BLANK_CHAR, END_CHAR];
    for char in forbidden_chars {
        if word.contains(char) {
            return Err(TuringTapeError::IllegalInputError {
                word: word.to_string(),
                illegal_char: char,
            });
        }
    }
    Ok(())
}

// Keeping the unit test here because we need to access private fields
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_reading_tape() {
        let tape = TuringReadingTape::new();

        assert_eq!(tape.pointer, 0);
        assert_eq!(tape.chars_vec, vec!(INIT_CHAR, END_CHAR));
        let tape = TuringWritingTape::new();

        assert_eq!(tape.pointer, 0);
        assert_eq!(tape.chars_vec, vec!(INIT_CHAR, BLANK_CHAR));
    }

    #[test]
    fn test_feed_word_tape() {
        let mut tape = TuringReadingTape::new();

        tape.feed_word("test".to_string()).unwrap();

        assert_eq!(
            tape.chars_vec,
            vec!(INIT_CHAR, 't', 'e', 's', 't', END_CHAR)
        );
    }

    #[test]
    fn test_feed_word_tape_illegal_char() {
        let mut tape = TuringReadingTape::new();

        match tape.feed_word("dsdçaaz".to_string()) {
            Ok(_) => panic!("Exepected an error"),
            Err(te) => expect_ill_action_error(te),
        }
        match tape.feed_word("dsdaaz$".to_string()) {
            Ok(_) => panic!("Exepected an error"),
            Err(te) => expect_ill_action_error(te),
        }
        match tape.feed_word("_dsdaaz".to_string()) {
            Ok(_) => panic!("Exepected an error"),
            Err(te) => expect_ill_action_error(te),
        }
    }

    fn expect_ill_action_error(te: TuringTapeError) {
        match te {
            TuringTapeError::IllegalInputError {
                word: _,
                illegal_char: _,
            }
            | TuringTapeError::SpecialCharPlacementError {
                og_char: _,
                special_char: _,
            }
            | TuringTapeError::SpecialCharReplacementError {
                special_char: _,
                replacement_char: _,
            } => {}
            _ => panic!(
                "Exepected an IllegalActionError, but received the following error : {:?}",
                te
            ),
        }
    }

    #[test]
    fn test_transition_read_tape() {
        let mut tape = TuringReadingTape::new();

        tape.feed_word("test".to_string()).unwrap();

        tape.try_apply_transition(INIT_CHAR, INIT_CHAR, &TuringDirection::Right)
            .unwrap();
        assert_eq!(tape.pointer, 1);
        tape.try_apply_transition('t', 'p', &TuringDirection::Left)
            .unwrap();
        assert_eq!(tape.pointer, 0);
        tape.try_apply_transition(INIT_CHAR, INIT_CHAR, &TuringDirection::None)
            .unwrap();
        assert_eq!(tape.pointer, 0);

        assert_eq!(
            tape.chars_vec,
            vec!(INIT_CHAR, 't', 'e', 's', 't', END_CHAR)
        );

        tape.try_apply_transition(INIT_CHAR, '_', &TuringDirection::Right)
            .unwrap();
        tape.try_apply_transition('t', '_', &TuringDirection::Right)
            .unwrap();
        tape.try_apply_transition('e', '_', &TuringDirection::Right)
            .unwrap();
        tape.try_apply_transition('s', '_', &TuringDirection::Right)
            .unwrap();
        tape.try_apply_transition('t', '_', &TuringDirection::Right)
            .unwrap();

        match tape.try_apply_transition(END_CHAR, '_', &TuringDirection::Right) {
            Ok(b) => {
                panic!(
                    "Transition should have returned a TuringError and not : {}",
                    b
                )
            }
            Err(e) => match e {
                TuringTapeError::OutofRangeError {
                    accessed_index: _,
                    tape_size: _,
                } => (),
                _ => panic!("OutofRangeTapeError was expected"),
            },
        }
    }

    #[test]
    fn test_illegal_replacement() {
        let mut tape = TuringWritingTape::new();

        tape.try_apply_transition(INIT_CHAR, INIT_CHAR, &TuringDirection::Right)
            .unwrap();
        assert_eq!(tape.pointer, 1);

        if tape
            .try_apply_transition(BLANK_CHAR, INIT_CHAR, &TuringDirection::Right)
            .is_ok()
        {
            panic!("An error should have been returned");
        }

        tape.try_apply_transition(BLANK_CHAR, BLANK_CHAR, &TuringDirection::Left)
            .unwrap();
        tape.try_apply_transition(BLANK_CHAR, BLANK_CHAR, &TuringDirection::Left)
            .unwrap();

        match tape.try_apply_transition(INIT_CHAR, 'p', &TuringDirection::Right) {
            Ok(_) => panic!("Exected an error"),
            Err(te) => expect_ill_action_error(te),
        }

        match tape.try_apply_transition(INIT_CHAR, END_CHAR, &TuringDirection::Right) {
            Ok(_) => panic!("Exected an error"),
            Err(te) => expect_ill_action_error(te),
        }

        match tape.try_apply_transition(INIT_CHAR, BLANK_CHAR, &TuringDirection::Right) {
            Ok(_) => panic!("Exected an error"),
            Err(te) => expect_ill_action_error(te),
        }
    }
}
