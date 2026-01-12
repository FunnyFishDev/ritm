use colored::{ColoredString, Colorize};
use rustyline::{Editor, history::FileHistory};
use std::fmt::Display;
use strum::IntoEnumIterator;

use crate::DataStorage;

pub trait ModeEvent {
    fn print_help(&self);
    fn choose_option(&self, rl: &mut Editor<(), FileHistory>, storage: &mut DataStorage) -> Modes;

    fn get_help_color(str: ColoredString) -> ColoredString;
}

#[derive(Debug, PartialEq, Clone)]
pub enum Modes {
    Start,
    Modify,
    Execute,
}

// _________________________________________________________________________

pub fn print_help<E>()
where
    E: IntoEnumIterator + Display + ModeEvent,
{
    for (i, mode) in E::iter().enumerate() {
        println!(
            "{}: {}",
            E::get_help_color(i.to_string().bold()),
            mode.to_string().italic()
        );
    }
}

pub fn print_help_gen<E>(vec: &[E])
where
    E: IntoEnumIterator + Display + ModeEvent,
{
    for (i, mode) in vec.iter().enumerate() {
        println!(
            "{}: {}",
            E::get_help_color(i.to_string().bold()),
            mode.to_string().italic()
        );
    }
}

pub fn collect_enum_values<E>() -> Vec<E>
where
    E: IntoEnumIterator + Display + ModeEvent,
{
    E::iter().collect()
}
