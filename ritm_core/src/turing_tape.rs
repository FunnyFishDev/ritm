use std::fmt::{Debug, Display};

use thiserror::Error;

use crate::turing_transition::{MatchSymbol, TuringDirection};

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

#[derive(Debug, Clone)]
pub struct TuringTape {
    chars_vec: Vec<char>,
    pointer: usize,
}

impl TuringTape {
    /// Creates a new [TuringTape]
    pub fn new(is_reading_tape: bool) -> Self {
        let last_char = if is_reading_tape {
            END_CHAR
        } else {
            BLANK_CHAR
        };
        Self {
            chars_vec: vec![INIT_CHAR, last_char],
            pointer: 0,
        }
    }

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
    pub fn try_apply_transition(
        &mut self,
        symbol_match: &MatchSymbol,
        replace_by: char,
        move_to: &TuringDirection,
    ) -> Result<bool, TuringTapeError> {
        // if the correct symbol was read
        if symbol_match.matches(self.chars_vec[self.pointer]) {
            let new_pointer = (self.pointer as isize) + (move_to.get_value() as isize);

            if new_pointer < 0 {
                return Err(TuringTapeError::OutofRangeError {
                    accessed_index: new_pointer as usize,
                    tape_size: self.chars_vec.len(),
                });
            }

            // In a writing tape, we have an *infinite size*, so we can simulate this by adding, when needed, a new empty char
            if new_pointer + 1 >= self.chars_vec.len() as isize
                && let Some(last) = self.chars_vec.last()
                && *last != END_CHAR
            {
                self.chars_vec.push('_');
            }

            check_replacement_validity(self.chars_vec[self.pointer], replace_by)?;

            // Replace the current char read
            self.chars_vec[self.pointer] = replace_by;

            // If we have two blank characters following each other, we can remove one
            if self.chars_vec.len() >= 2
                && let Some(last) = self.chars_vec.last()
                && let Some(prev_last) = self.chars_vec.get(self.chars_vec.len() - 2)
                && *last == *prev_last
                && *last == BLANK_CHAR
            {
                self.chars_vec.pop();
            }

            // Move to the new position
            self.pointer = new_pointer as usize;
            return Ok(true);
        }
        Ok(false)
    }

    /// Returns the current character being read by the tape
    pub fn read_curr_char(&self) -> char {
        self.chars_vec[self.pointer]
    }

    /// Returns the vector of char as stored by the tape
    pub fn get_contents(&self) -> &Vec<char> {
        &self.chars_vec
    }

    /// Returns the index of the char being pointed by the tape
    pub fn get_pointer(&self) -> usize {
        self.pointer
    }

    pub fn feed_word(
        &mut self,
        word: impl ToString,
        add_end_char: bool,
    ) -> Result<(), TuringTapeError> {
        let word = word.to_string();
        check_word_validity(&word)?;

        self.chars_vec.clear();
        self.chars_vec.push(INIT_CHAR);
        for ch in word.chars() {
            self.chars_vec.push(ch);
        }
        if add_end_char {
            self.chars_vec.push(END_CHAR);
        } else if let Some(last_char) = self.chars_vec.last()
            && *last_char != BLANK_CHAR
        {
            self.chars_vec.push(BLANK_CHAR);
        }
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

    if og_char == INIT_CHAR || og_char == END_CHAR {
        return Err(TuringTapeError::SpecialCharReplacementError {
            special_char: og_char,
            replacement_char: new_char,
        });
    }

    Ok(())
}

impl Display for TuringTape {
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
        let tape = TuringTape::new(true);

        assert_eq!(tape.pointer, 0);
        assert_eq!(tape.chars_vec, vec!(INIT_CHAR, END_CHAR));
        let tape = TuringTape::new(false);

        assert_eq!(tape.pointer, 0);
        assert_eq!(tape.chars_vec, vec!(INIT_CHAR, BLANK_CHAR));
    }

    #[test]
    fn test_feed_word_tape() {
        let mut tape = TuringTape::new(false);

        tape.feed_word("test".to_string(), true).unwrap();

        assert_eq!(
            tape.chars_vec,
            vec!(INIT_CHAR, 't', 'e', 's', 't', END_CHAR)
        );
    }

    #[test]
    fn test_feed_word_tape_illegal_char() {
        let mut tape = TuringTape::new(true);

        match tape.feed_word("dsdçaaz".to_string(), true) {
            Ok(_) => panic!("Exepected an error"),
            Err(te) => expect_ill_action_error(te),
        }
        match tape.feed_word("dsdaaz$".to_string(), true) {
            Ok(_) => panic!("Exepected an error"),
            Err(te) => expect_ill_action_error(te),
        }
        match tape.feed_word("_dsdaaz".to_string(), true) {
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
        let mut tape = TuringTape::new(false);

        tape.feed_word("test".to_string(), true).unwrap();

        tape.try_apply_transition(
            &MatchSymbol::Char(INIT_CHAR),
            INIT_CHAR,
            &TuringDirection::Right,
        )
        .unwrap();
        assert_eq!(tape.pointer, 1);
        tape.try_apply_transition(&MatchSymbol::Char('t'), 'p', &TuringDirection::Left)
            .unwrap();
        assert_eq!(tape.pointer, 0);
        tape.try_apply_transition(
            &MatchSymbol::Char(INIT_CHAR),
            INIT_CHAR,
            &TuringDirection::None,
        )
        .unwrap();
        assert_eq!(tape.pointer, 0);

        assert_eq!(
            tape.chars_vec,
            vec!(INIT_CHAR, 'p', 'e', 's', 't', END_CHAR)
        );

        assert!(matches!(
            tape.try_apply_transition(&MatchSymbol::Char(INIT_CHAR), '_', &TuringDirection::Right),
            Err(TuringTapeError::SpecialCharReplacementError {
                special_char,
                replacement_char
            }) if special_char == INIT_CHAR && replacement_char == '_'
        ));

        tape.try_apply_transition(&MatchSymbol::Char('ç'), 'ç', &TuringDirection::Right)
            .unwrap();

        tape.try_apply_transition(&MatchSymbol::Char('p'), '_', &TuringDirection::Right)
            .unwrap();
        tape.try_apply_transition(&MatchSymbol::Char('e'), '_', &TuringDirection::Right)
            .unwrap();
        tape.try_apply_transition(&MatchSymbol::Char('s'), '_', &TuringDirection::Right)
            .unwrap();
        tape.try_apply_transition(&MatchSymbol::Char('t'), '_', &TuringDirection::Right)
            .unwrap();

        assert!(matches!(
            tape.try_apply_transition(&MatchSymbol::Char(END_CHAR), '_', &TuringDirection::Right),
            Err(TuringTapeError::SpecialCharReplacementError {
                special_char,
                replacement_char
            }) if special_char == END_CHAR && replacement_char == '_'
        ))
    }

    #[test]
    fn test_illegal_replacement() {
        let mut tape = TuringTape::new(false);

        tape.try_apply_transition(
            &MatchSymbol::Char(INIT_CHAR),
            INIT_CHAR,
            &TuringDirection::Right,
        )
        .unwrap();
        assert_eq!(tape.pointer, 1);

        if tape
            .try_apply_transition(
                &MatchSymbol::Char(BLANK_CHAR),
                INIT_CHAR,
                &TuringDirection::Right,
            )
            .is_ok()
        {
            panic!("An error should have been returned");
        }

        tape.try_apply_transition(
            &MatchSymbol::Char(BLANK_CHAR),
            BLANK_CHAR,
            &TuringDirection::Left,
        )
        .unwrap();
        tape.try_apply_transition(
            &MatchSymbol::Char(BLANK_CHAR),
            BLANK_CHAR,
            &TuringDirection::Left,
        )
        .unwrap();

        match tape.try_apply_transition(&MatchSymbol::Char(INIT_CHAR), 'p', &TuringDirection::Right)
        {
            Ok(_) => panic!("Exected an error"),
            Err(te) => expect_ill_action_error(te),
        }

        match tape.try_apply_transition(
            &MatchSymbol::Char(INIT_CHAR),
            END_CHAR,
            &TuringDirection::Right,
        ) {
            Ok(_) => panic!("Exected an error"),
            Err(te) => expect_ill_action_error(te),
        }

        match tape.try_apply_transition(
            &MatchSymbol::Char(INIT_CHAR),
            BLANK_CHAR,
            &TuringDirection::Right,
        ) {
            Ok(_) => panic!("Exected an error"),
            Err(te) => expect_ill_action_error(te),
        }
    }
}
