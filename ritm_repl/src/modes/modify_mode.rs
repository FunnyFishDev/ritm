use std::{
    fmt::Display,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use ritm_core::{
    SimpleTuringGraph,
    turing_graph::TuringStateType,
    turing_machine::TuringMachine,
    turing_parser::{self, parse_transition_string},
    turing_transition::TransitionMultRibbonInfo,
};
use rustyline::{Editor, history::FileHistory};
use strum_macros::EnumIter;

use colored::{ColoredString, Colorize};

use crate::{
    DataStorage,
    modes::{
        choice_modes::{ModeEvent, Modes},
        execute_mode,
    },
    query_string,
    ripl_error::{RiplError, print_error_help},
};

#[derive(EnumIter)]
pub enum ModifyTuringMode {
    PrintSummary,
    AddState,
    AddTransitions,
    RemoveTransitions,
    RemoveState,
    SaveTM,
    FeedWord,
    UnloadTM,
}

impl Display for ModifyTuringMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ModifyTuringMode::PrintSummary => "Print a summary of the Turing Machine",
                ModifyTuringMode::AddState => "Add a state",
                ModifyTuringMode::AddTransitions => "Add one or multiple transition",
                ModifyTuringMode::RemoveTransitions => "Remove one or multiple transition",
                ModifyTuringMode::RemoveState => "Remove a state",
                ModifyTuringMode::SaveTM => "Save this TM as a file",
                ModifyTuringMode::FeedWord => "Feed a word and start executing this Turing Machine",
                ModifyTuringMode::UnloadTM => "Unload the current Turing Machine",
            }
        )
    }
}

impl ModeEvent for ModifyTuringMode {
    fn print_help(&self) {
        let tm_it_bold = "Turing Machine".italic().bold();
        print!("-> ");
        println!("{}", match self {
                ModifyTuringMode::PrintSummary => format!("Prints a detailed overview of the current {tm_it_bold}"),
                ModifyTuringMode::AddState => format!("Adds a {} to the current {tm_it_bold}", "state".purple()),
                ModifyTuringMode::AddTransitions => format!("Adds one or multiple {} to the current {tm_it_bold}", "transitions".purple()),
                ModifyTuringMode::RemoveTransitions => format!("Removes one or multiple {} from the current {tm_it_bold}", "transitions".purple()),
                ModifyTuringMode::RemoveState => format!("Removes a {} from the current {tm_it_bold}", "state".purple()),
                ModifyTuringMode::SaveTM => format!("Saves the current {tm_it_bold} as a file"),
                ModifyTuringMode::FeedWord => format!("Feeds a word to the {tm_it_bold} and starts executing it"),
                ModifyTuringMode::UnloadTM => "Unloads the current Turing Machine and go back to the turing machine creation phase".to_string()
            }.green())
    }

    fn choose_option(&self, rl: &mut Editor<(), FileHistory>, storage: &mut DataStorage) -> Modes {
        let tm = storage.graph.as_mut().unwrap();
        match self {
            ModifyTuringMode::PrintSummary => {
                println!("{}", tm.to_string().blue());
            }
            ModifyTuringMode::AddState => {
                let res = get_state_name(rl);
                if let Err(e) = res {
                    print_error_help(e);
                } else {
                    let name = &res.unwrap();
                    println!(
                        "{}",
                        match tm.try_add_state(name, TuringStateType::Normal) {
                            Ok(_) => {
                                format!("Successfully added the state \'q_{}\'.", name.yellow())
                                    .green()
                            }
                            Err(err) => {
                                format!("Ran into an error : {err}").red()
                            }
                        }
                    )
                }
            }
            ModifyTuringMode::AddTransitions => {
                if let Err(e) = add_transition(rl, tm) {
                    print_error_help(e);
                }
            }
            ModifyTuringMode::RemoveTransitions => {
                if let Err(e) = remove_transition(rl, tm) {
                    print_error_help(e);
                }
            }
            ModifyTuringMode::RemoveState => {
                let res = get_state_name(rl);
                if let Err(e) = res {
                    print_error_help(e);
                } else {
                    let name = &res.unwrap();
                    if let Err(e) = tm.remove_state(name) {
                        print_error_help(RiplError::EncounteredTuringError { error: e.into() });
                    } else {
                        println!("{}", format!("Successfully removed the state \'q_{}\' and all related transitions.", name.yellow()).green())
                    }
                }
            }
            ModifyTuringMode::SaveTM => {
                if let Err(e) = save_tm(rl, tm, &storage.curr_path) {
                    print_error_help(e);
                }
            }
            ModifyTuringMode::FeedWord => {
                let res = query_string(
                    rl,
                    "Enter the word to feed to this Turing machine: ".to_string(),
                );
                if let Err(e) = res {
                    print_error_help(e);
                } else {
                    let res =
                        TuringMachine::new(tm.clone(), res.unwrap(), storage.exec_mode.clone());
                    if let Err(e) = res {
                        print_error_help(RiplError::EncounteredTuringError { error: e });
                    } else {
                        storage.iterator = Some(res.unwrap());
                        execute_mode::next_step(
                            rl,
                            storage.iterator.as_mut().unwrap(),
                            storage.clear_after_step,
                        );
                        return Modes::Execute;
                    }
                }
            }
            ModifyTuringMode::UnloadTM => {
                // If it does, ask user for confirmation before deleting the current graph it
                let choice = query_string(
                    rl,
                    format!(
                        "Unloading this graph will delete it. Are you sure ? {}: ",
                        "Y(es) or N(o)".italic().blue()
                    ),
                );
                match choice {
                    Ok(choice) => {
                        let choice = choice.to_lowercase();
                        if choice.eq("y") || choice.eq("yes") {
                            storage.graph = None;
                            return Modes::Start;
                        }
                    }
                    Err(e) => {
                        print_error_help(e);
                    }
                }
            }
        }
        Modes::Modify
    }

    fn get_help_color(str: colored::ColoredString) -> colored::ColoredString {
        str.purple()
    }
}

fn get_state_name(rl: &mut Editor<(), FileHistory>) -> Result<String, RiplError> {
    let name_res = query_string(rl, format!("Enter the {} of the state: ", "name".blue()));

    match name_res {
        Ok(name) => {
            if name.starts_with("q_") {
                Ok(name.strip_prefix("q_").unwrap().to_string())
            } else if name.starts_with("q") {
                Ok(name.strip_prefix("q").unwrap().to_string())
            } else {
                Ok(name)
            }
        }
        Err(e) => Err(e),
    }
}

fn add_transition(
    rl: &mut Editor<(), FileHistory>,
    turing_graph: &mut SimpleTuringGraph,
) -> Result<(), RiplError> {
    let (q1, vec_tm, q2) = query_transition(
        rl,
        format!(
            "Enter one or multiple {} to add to the graph: ",
            "transitions".blue()
        ),
    )?;

    for transition in vec_tm {
        if let Err(e) = turing_graph.append_transition(&q1, transition.clone(), &q2) {
            print_error_help(RiplError::EncounteredTuringError { error: e.into() });
        } else {
            println!(
                "{}{}",
                "Successfully added the transition : ".green(),
                format_transition(&q1, &transition, &q2)
            )
        }
    }

    Ok(())
}

fn remove_transition(
    rl: &mut Editor<(), FileHistory>,
    turing_graph: &mut SimpleTuringGraph,
) -> Result<(), RiplError> {
    let (q1, vec_tm, q2) = query_transition(
        rl,
        format!(
            "Enter one or multiple {} to {} from the the graph: ",
            "transitions".blue(),
            "remove".bold()
        ),
    )?;

    for transition in vec_tm {
        if let Err(e) = turing_graph.remove_transition((&q1, transition.clone(), &q2)) {
            print_error_help(RiplError::EncounteredTuringError { error: e.into() });
        } else {
            println!(
                "{}{}",
                "Successfully removed the transition : ".green(),
                format_transition(&q1, &transition, &q2)
            )
        }
    }

    Ok(())
}

fn format_transition(
    from: &String,
    transition: &TransitionMultRibbonInfo,
    to: &String,
) -> ColoredString {
    format!("q_{} {}{}{} q_{}", from, "{", transition, "}", to).yellow()
}

pub fn query_transition(
    rl: &mut Editor<(), FileHistory>,
    query: String,
) -> Result<(String, Vec<TransitionMultRibbonInfo>, String), RiplError> {
    println!("{}", query);
    loop {
        let readline = rl.readline("==> ");
        match readline {
            Ok(l) => {
                let l = l.trim().to_string();
                if l.is_empty() {
                    continue;
                }
                rl.add_history_entry(l.to_string()).unwrap();

                let res = parse_transition_string(l);
                if let Err(e) = res {
                    return Err(RiplError::EncounteredParsingError { error: e });
                }
                return Ok(res.unwrap());
            }
            Err(e) => {
                return Err(RiplError::CouldNotParseStringError {
                    value: e.to_string(),
                });
            }
        }
    }
}

fn save_tm(
    rl: &mut Editor<(), FileHistory>,
    tm: &SimpleTuringGraph,
    current_path: &Option<PathBuf>,
) -> Result<(), RiplError> {
    let tm_string = turing_parser::graph_to_string(tm);
    if tm_string.is_empty() {
        println!(
            "{}",
            "Nothing to save because this machine has no transitions !".red()
        );
        return Ok(());
    }
    loop {
        println!(
            "Enter the {} of the {} to create: ",
            "path".bold().blue(),
            "file".bold()
        );
        let readline = match current_path {
            Some(p) => rl.readline_with_initial(
                "==> ",
                (
                    p.as_path()
                        .join("turing_machine")
                        .to_str()
                        .unwrap()
                        .to_string()
                        .as_str(),
                    ".tm",
                ),
            ),
            None => rl.readline("==> "),
        };
        match readline {
            Ok(l) => {
                let l = l.trim().to_string();
                if l.is_empty() {
                    continue;
                }
                rl.add_history_entry(l.to_string()).unwrap();

                let path = Path::new(&l);
                // Check that no file with this name exists

                if path.exists() {
                    // If it does, ask user for confirmation before overwritting it
                    let choice = query_string(
                        rl,
                        format!(
                            "A file with this name already exists, rewrite it ? {}: ",
                            "Y(es) or N(o)".italic().blue()
                        ),
                    );
                    match choice {
                        Ok(choice) => {
                            let choice = choice.to_lowercase();
                            if !choice.eq("y") && !choice.eq("yes") {
                                continue;
                            }
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }

                let file = File::create(path);
                match file {
                    Ok(mut f) => {
                        if let Err(e) = f.write_all(tm_string.as_bytes()) {
                            return Err(RiplError::FileError {
                                file_path: Some(e.to_string()),
                            });
                        }
                    }
                    Err(e) => {
                        return Err(RiplError::FileError {
                            file_path: Some(e.to_string()),
                        });
                    }
                }

                println!(
                    "{}{}",
                    "Saved the file at the location : ".green(),
                    path.to_str().unwrap()
                );
                return Ok(());
            }
            Err(e) => {
                return Err(RiplError::CouldNotParseStringError {
                    value: e.to_string(),
                });
            }
        }
    }
}
