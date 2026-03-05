use std::fmt::{Debug, Display};
use thiserror::Error;

use crate::turing_tape::{self, INIT_CHAR};

pub const ANY_CHAR_SYMBOL: char = '@';

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

pub trait TuringTransition: Clone + Default + Debug + PartialEq {}

#[derive(Debug, Clone, PartialEq)]
/// A struct representing a transition for a turing machine that has strictly more than **1 tape** :
/// * `a_0, a_1, ..., a_{n-1} -> D_0, b_1, D_1, b_2, D_2, ..., b_{n-1}, D_{n-1}`
/// - With :
///     * `a_i` : The character *i* being read.
///     * `D_i` : Direction to take by taking this transition, see [TuringDirection] for more information.
///     * `b_i` : The character to replace the character *i* with.
/// ## Comparisons
/// In order to simplify the graph exploration, when compared, only the [`TuringTransitionInfo`] fields will be compared.
pub struct TuringTransitionWrapper<T: TuringTransition> {
    pub inner_transition: T,
    pub info: TransitionsInfo,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransitionsInfo {
    OneTape(TransitionOneRibbonInfo),
    MultipleTapes(TransitionMultRibbonInfo),
}

impl TransitionsInfo {
    pub fn has_one_ribbon(&self) -> bool {
        match self {
            TransitionsInfo::OneTape(_transition_one_ribbon_info) => true,
            TransitionsInfo::MultipleTapes(_transition_mult_ribbon_info) => false,
        }
    }

    pub fn get_nb_ribbons(&self) -> usize {
        match self {
            TransitionsInfo::OneTape(_transition_one_ribbon_info) => 1,
            TransitionsInfo::MultipleTapes(transition_mult_ribbon_info) => {
                transition_mult_ribbon_info.get_number_of_affected_tapes()
            }
        }
    }

    pub fn is_valid(&self, chars_to_read: &[char]) -> bool {
        match self {
            TransitionsInfo::OneTape(transition_one_ribbon_info) => {
                chars_to_read.len() == 1
                    && transition_one_ribbon_info
                        .chars_read
                        .matches(chars_to_read[0])
            }
            TransitionsInfo::MultipleTapes(transition_mult_ribbon_info) => {
                transition_mult_ribbon_info
                    .match_symbols
                    .matches_all(chars_to_read)
            }
        }
    }

    pub fn get_match_symbols(&self) -> Vec<MatchSymbol> {
        match self {
            TransitionsInfo::OneTape(transition_one_ribbon_info) => {
                vec![transition_one_ribbon_info.chars_read.clone()]
            }
            TransitionsInfo::MultipleTapes(transition_mult_ribbon_info) => {
                transition_mult_ribbon_info.match_symbols.clone()
            }
        }
    }

    pub fn get_move_read(&self) -> TuringDirection {
        match self {
            TransitionsInfo::OneTape(transition_one_ribbon_info) => {
                transition_one_ribbon_info.move_pointer.clone()
            }
            TransitionsInfo::MultipleTapes(transition_mult_ribbon_info) => {
                transition_mult_ribbon_info.move_read.clone()
            }
        }
    }
}

impl From<TransitionMultRibbonInfo> for TransitionsInfo {
    fn from(value: TransitionMultRibbonInfo) -> Self {
        Self::MultipleTapes(value)
    }
}
impl From<TransitionOneRibbonInfo> for TransitionsInfo {
    fn from(value: TransitionOneRibbonInfo) -> Self {
        Self::OneTape(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransitionMultRibbonInfo {
    /// The chars that have to be read in order apply the rest of the transition : `a_0,..., a_{n-1}`
    pub match_symbols: Vec<MatchSymbol>,
    /// The move to take after writing/reading the character : `D_0`
    pub move_read: TuringDirection,
    /// The character to replace the character just read : `(b_1, D_1),..., (b_{n-1}, D_{n-1})`
    pub chars_write: Vec<(char, TuringDirection)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransitionOneRibbonInfo {
    /// The chars that has to be read for this transition to be valid
    pub chars_read: MatchSymbol,
    /// The move to take after reading the character
    pub move_pointer: TuringDirection,
    /// The character to replace the character just read
    pub replace_with: char,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchSymbol {
    Char(char),
    Range(char, char),
    Any,
}

pub trait MatchesAll {
    fn matches_all(&self, other: &[char]) -> bool;
}

impl MatchSymbol {
    pub fn matches(&self, value: char) -> bool {
        match self {
            MatchSymbol::Char(c) => value == *c,
            MatchSymbol::Range(min_c, max_c) => {
                let min: u32 = (*min_c).into();
                let max: u32 = (*max_c).into();
                let val: u32 = value.into();
                min <= val || val <= max
            }
            MatchSymbol::Any => true,
        }
    }
}

impl Display for MatchSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MatchSymbol::Char(c) => c.to_string(),
                MatchSymbol::Range(min_c, max_c) => format!("{{{min_c}}}..{{{max_c}}}"),
                MatchSymbol::Any => '@'.to_string(),
            }
        )
    }
}

impl MatchesAll for Vec<MatchSymbol> {
    fn matches_all(&self, other: &[char]) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for (i, m) in self.iter().enumerate() {
            if !m.matches(other[i]) {
                return false;
            }
        }

        true
    }
}

impl TransitionOneRibbonInfo {
    pub fn new(
        match_symbol: MatchSymbol,
        move_pointer: TuringDirection,
        replace_with: char,
    ) -> Result<Self, TuringTransitionError> {
        check_ill_directions(
            &match_symbol,
            INIT_CHAR,
            &move_pointer,
            &TuringDirection::Left,
        )?;
        check_ill_replacement(&match_symbol, replace_with, 0)?;

        Ok(Self {
            chars_read: match_symbol,
            replace_with,
            move_pointer,
        })
    }
}

impl Default for TransitionOneRibbonInfo {
    fn default() -> Self {
        Self {
            chars_read: MatchSymbol::Char('ç'),
            replace_with: 'ç',
            move_pointer: TuringDirection::None,
        }
    }
}

impl<T: TuringTransition> From<TransitionsInfo> for TuringTransitionWrapper<T> {
    fn from(value: TransitionsInfo) -> Self {
        TuringTransitionWrapper {
            inner_transition: T::default(),
            info: value,
        }
    }
}

impl<T: TuringTransition> From<TransitionMultRibbonInfo> for TuringTransitionWrapper<T> {
    fn from(value: TransitionMultRibbonInfo) -> Self {
        TuringTransitionWrapper {
            inner_transition: T::default(),
            info: value.into(),
        }
    }
}

impl<T: TuringTransition> From<TransitionOneRibbonInfo> for TuringTransitionWrapper<T> {
    fn from(value: TransitionOneRibbonInfo) -> Self {
        TuringTransitionWrapper {
            inner_transition: T::default(),
            info: value.into(),
        }
    }
}

impl TransitionMultRibbonInfo {
    /// Creates a new [`TransitionMultRibbonInfo`].
    pub fn new(
        symbols_match: Vec<MatchSymbol>,
        move_read: TuringDirection,
        chars_read_write: Vec<(char, TuringDirection)>,
    ) -> Result<Self, TuringTransitionError> {
        let mut directions = Vec::with_capacity(chars_read_write.len() + 1);
        directions.push(move_read);

        let mut chars_write = Vec::with_capacity(chars_read_write.len());

        chars_read_write.into_iter().for_each(|(c, dir)| {
            chars_write.push(c);
            directions.push(dir);
        });

        Self::create(symbols_match, chars_write, directions)
    }

    /// Simplifies the creation of a new [TuringTransition] of the form :
    /// * `a_0, a_1, ..., a_{n-1} -> D_0, b_1, D_1, b_2, D_2, ..., b_{n-1}, D_{n-1}`
    ///
    /// ## Args :
    /// * **symbols_match** : Determines the symbols that have to be read in order to take this transition : `a_0,..., a_{n-1}`
    /// * **chars_write** : The characters to replace the characters read : `b_1, ..., b_{n-1}`
    /// * **directions** : The directions to move the pointers of the tapes : `D_0, ..., D_{n-1}`
    pub fn create(
        symbols_match: Vec<MatchSymbol>,
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
        let move_read = move_read.expect("Value present").clone();

        if chars_write.len() + 1 != directions.len() {
            return Err(TuringTransitionError::TransitionArgsError("The number of character to write must be equal to the number of directions minus one (for the reading tape)".to_string()));
        }
        if symbols_match.len() != directions.len() {
            return Err(TuringTransitionError::TransitionArgsError(
                "The number of characters to read must be equal to the number of given directions"
                    .to_string(),
            ));
        }
        for i in 1..directions.len() {
            chars_write_dir.push((
                *chars_write.get(i - 1).expect("Value present"),
                directions.get(i).expect("Value present").clone(),
            ));
        }

        // Check for illegal actions
        //  Only applies to the reading tape
        check_ill_directions(
            symbols_match.first().expect("Value present"),
            turing_tape::END_CHAR,
            &move_read,
            &TuringDirection::Right,
        )?;

        //  Applies to all tapes, therefore we need to iterate over all of them

        // check for reading first
        check_ill_directions(
            symbols_match.first().expect("Value present"),
            turing_tape::INIT_CHAR,
            &move_read,
            &TuringDirection::Left,
        )?;
        // then for writting tapes
        for i in 1..symbols_match.len() {
            let char_read = symbols_match.get(i).expect("Value present");

            let (char_relacement, char_dir) = chars_write_dir.get(i - 1).expect("value present");

            check_ill_directions(
                char_read,
                turing_tape::INIT_CHAR,
                char_dir,
                &TuringDirection::Left,
            )?;

            check_ill_replacement(char_read, *char_relacement, i)?;
        }

        Ok(Self {
            match_symbols: symbols_match,
            move_read,
            chars_write: chars_write_dir,
        })
    }

    /// Creates a valid default transition using the given number of working ribbons.
    /// For example if *k* is equal to 3:
    ///   * `{ ç, ç, ç, ç -> N, ç, N, ç, N, ç, N }`
    pub fn create_default(nb_working_ribbons: usize) -> Self {
        let mut chars_read = Vec::new();
        let mut chars_write = Vec::new();

        chars_read.push(MatchSymbol::Char('ç'));

        (0..nb_working_ribbons).for_each(|_| {
            chars_read.push(MatchSymbol::Char('ç'));
            chars_write.push(('ç', TuringDirection::None));
        });

        Self {
            match_symbols: chars_read,
            move_read: TuringDirection::None,
            chars_write,
        }
    }

    /// Returns the number of tapes that are going to be affected by this transition. (k + 1)
    pub fn get_number_of_affected_tapes(&self) -> usize {
        self.chars_write.len() + 1
    }
}

fn check_ill_directions(
    curr_char: &MatchSymbol,
    cond_char: char,
    curr_dir: &TuringDirection,
    forbidden_dir: &TuringDirection,
) -> Result<(), TuringTransitionError> {
    if curr_char.matches(cond_char) && curr_dir == forbidden_dir {
        Err(TuringTransitionError::IllegalActionError(format!(
            "Detected the couple : (\"{curr_char}\", \"{curr_dir}\"), this could result in going out of bounds of the tape. Change the given direction to None for example."
        )))
    } else {
        Ok(())
    }
}

fn check_ill_replacement(
    curr_char: &MatchSymbol,
    char_replacement: char,
    tape_id: usize,
) -> Result<(), TuringTransitionError> {
    if curr_char.matches(turing_tape::INIT_CHAR) {
        if !curr_char.matches(char_replacement) {
            return Err(TuringTransitionError::IllegalActionError(format!(
                "Tried to replace a special character ('{curr_char}') with another character ('{char_replacement}') for the tape {tape_id}",
            )));
        }
    } else if char_replacement == turing_tape::INIT_CHAR {
        return Err(TuringTransitionError::IllegalActionError(format!(
            "Tried to replace a normal character ('{curr_char}') with a special character ('{char_replacement}') for the tape {tape_id}",
        )));
    };
    Ok(())
}

impl Display for TransitionsInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TransitionsInfo::OneTape(transition_one_ribbon_info) =>
                    transition_one_ribbon_info.to_string(),
                TransitionsInfo::MultipleTapes(transition_mult_ribbon_info) =>
                    transition_mult_ribbon_info.to_string(),
            }
        )
    }
}

impl Display for TransitionMultRibbonInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut char_read = self.match_symbols[0].to_string();
        for i in 1..self.match_symbols.len() {
            char_read.push_str(format!(", {}", self.match_symbols[i]).as_str());
        }

        let mut char_written = format!("{}", self.move_read);

        for (c, dir) in &self.chars_write {
            char_written.push_str(format!(", {}, {}", c, dir).as_str());
        }

        write!(f, "{} -> {}", char_read, char_written)
    }
}

impl Display for TransitionOneRibbonInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -> {}, {}",
            self.chars_read, self.replace_with, self.move_pointer
        )
    }
}
