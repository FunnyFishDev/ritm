use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use crate::{
    DataStorage,
    modes::choice_modes::{ModeEvent, Modes},
    query_string, query_usize,
    ripl_error::{RiplError, print_error_help},
};
use colored::Colorize;
use ritm_core::{SimpleTuringGraph, turing_parser::parse_turing_graph_file_path};
use rustyline::{Editor, history::FileHistory};
use strum_macros::EnumIter;

#[derive(EnumIter)]
pub enum StartingMode {
    CreateTM,
    LoadTM,
}

impl Display for StartingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                StartingMode::CreateTM => "Create new Turing Machine",
                StartingMode::LoadTM => "Load an existing Turing Machine",
            }
        )
    }
}

impl ModeEvent for StartingMode {
    fn print_help(&self) {
        let tm_it = "Turing Machine".italic().bold();
        print!("-> ");
        println!(
            "{}",
            match self {
                StartingMode::CreateTM => format!(
                    "Creates a new {tm_it} by specifying the {}",
                    "number of writting tapes".purple()
                ),
                StartingMode::LoadTM => format!(
                    "Loads a new {tm_it} by specifying a {} to it from",
                    "file path".purple()
                ),
            }
            .green()
        );
    }

    fn choose_option(&self, rl: &mut Editor<(), FileHistory>, storage: &mut DataStorage) -> Modes {
        let res = match self {
            StartingMode::CreateTM => create_tm(rl),
            StartingMode::LoadTM => query_load_tm(rl, &storage.curr_path),
        };
        if let Err(e) = res {
            print_error_help(e);
            return Modes::Start;
        }
        // Store the created/loaded turing machine graph
        storage.graph = Some(res.unwrap());
        // Change the mode to allow modifying this tm graph
        Modes::Modify
    }

    fn get_help_color(str: colored::ColoredString) -> colored::ColoredString {
        str.blue()
    }
}

fn create_tm(rl: &mut Editor<(), FileHistory>) -> Result<SimpleTuringGraph, RiplError> {
    let res = query_usize(
        rl,
        format!(
            "Enter the numbers of {} of the Turing machine ({}) :",
            "writting tapes".blue(),
            "k".blue().italic()
        ),
    )?;

    let tm = SimpleTuringGraph::new(res, true);
    if let Err(e) = tm {
        return Err(RiplError::EncounteredTuringError { error: e.into() });
    }

    Ok(tm.unwrap())
}

fn query_load_tm(
    rl: &mut Editor<(), FileHistory>,
    current_path: &Option<PathBuf>,
) -> Result<SimpleTuringGraph, RiplError> {
    let path_str = query_string(
        rl,
        format!("Enter the {} the Turing machine to read:", "path".blue()),
    )?;

    load_tm(current_path, &path_str)
}

pub fn load_tm(
    current_path: &Option<PathBuf>,
    given_input: &String,
) -> Result<SimpleTuringGraph, RiplError> {
    // Check if the path is absolute or not
    let path = Path::new(&given_input);

    let abs_path = {
        if !path.is_absolute() {
            // Create the absolute path
            match current_path {
                Some(curr_path) => {
                    let abs_path = curr_path.join(path);
                    abs_path.to_str()
                }
                None => path.to_str(),
            };
        }
        path.to_str()
    };
    if abs_path.is_none() {
        return Err(RiplError::FileError { file_path: None });
    }
    if !Path::new(abs_path.unwrap()).exists() {
        return Err(RiplError::FileNotExistError {
            file_path: abs_path.unwrap().to_string(),
        });
    }

    let tm = parse_turing_graph_file_path(abs_path.unwrap().to_string());
    if let Err(e) = tm {
        return Err(RiplError::EncounteredParsingError { error: e });
    }
    Ok(tm.unwrap())
}
