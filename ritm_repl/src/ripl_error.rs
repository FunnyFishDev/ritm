use std::fmt::Display;

use colored::{ColoredString, Colorize};
use ritm_core::{turing_machine::TuringMachineError, turing_parser::TuringParserError};

#[derive(Debug)]
pub enum RiplError {
    OutOfRangeIndexError { index: usize },
    NegativeValueError { value: f32 },
    UnknownCommandError { command: String },
    CouldNotParseStringIntError { value: String },
    CouldNotParseStringError { value: String },
    ArgsNumberError { received: usize, expected: usize },
    EncounteredTuringError { error: TuringMachineError },
    EncounteredParsingError { error: TuringParserError },
    FileError { file_path: Option<String> },
    FileNotExistError { file_path: String },
}

pub fn print_error_help(error: RiplError) {
    println!(
        "{}",
        match error {
            RiplError::OutOfRangeIndexError { index } => format!(
                "The given index \"{}\" is not a valid index",
                as_arg_error(index)
            ),
            RiplError::UnknownCommandError { command } => format!(
                "The given command \"{}\" is not a known command",
                as_arg_error(command)
            ),
            RiplError::CouldNotParseStringIntError { value } => format!(
                "Could not parse the value \"{}\" into a positive integer",
                as_arg_error(value)
            ),
            RiplError::CouldNotParseStringError { value } =>
                format!("Could not parse the value \"{}\"", as_arg_error(value)),
            RiplError::ArgsNumberError { received, expected } => format!(
                "Read wrong number of arguments. Expected {} but read {} args",
                expected.to_string().blue(),
                as_arg_error(received)
            ),
            RiplError::EncounteredTuringError { error } =>
                format!("Ran into the following turing error : \n{}", error),
            RiplError::EncounteredParsingError { error } => format!(
                "Ran into the following error during the parsing : \n{}",
                error
            ),
            RiplError::FileError { file_path } => format!(
                "Ran into an error trying to acess/write into following file : {:?}",
                file_path
            ),
            RiplError::FileNotExistError { file_path } => format!(
                "No file found at the given path : \"{}\"",
                as_arg_error(file_path)
            ),
            RiplError::NegativeValueError { value } =>
                format!("The given value (\"{value}\") should not be negative."),
        }
        .red()
    )
}

fn as_arg_error<E>(val: E) -> ColoredString
where
    E: Display,
{
    val.to_string().green()
}
