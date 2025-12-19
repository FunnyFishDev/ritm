use std::{
    fmt::Debug,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, atomic::AtomicBool},
};

use ritm_core::{SimpleTuringGraph, SimpleTuringMachine, turing_machine::Mode};
use rustyline::{Editor, history::FileHistory};

use crate::ripl_error::RiplError;

pub mod modes;

pub mod ripl_error;

pub struct DataStorage {
    pub graph: Option<SimpleTuringGraph>,
    pub iterator: Option<SimpleTuringMachine>,
    pub is_running: Arc<AtomicBool>,
    pub curr_path: Option<PathBuf>,
    pub clear_after_step: bool,
    pub exec_mode: Mode,
}

pub fn query_usize(rl: &mut Editor<(), FileHistory>, query: String) -> Result<usize, RiplError> {
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

                // Read requested nb
                let index_res = l.parse();
                if index_res.is_err() {
                    return Err(RiplError::CouldNotParseStringIntError { value: l });
                }
                return Ok(index_res.unwrap());
            }
            Err(e) => {
                return Err(RiplError::CouldNotParseStringError {
                    value: e.to_string(),
                });
            }
        }
    }
}

pub fn query_prim<E: FromStr>(
    rl: &mut Editor<(), FileHistory>,
    query: String,
) -> Result<E, RiplError>
where
    <E as FromStr>::Err: Debug,
{
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

                // Read requested nb
                let index_res = l.parse();
                if index_res.is_err() {
                    return Err(RiplError::CouldNotParseStringIntError { value: l });
                }
                return Ok(index_res.unwrap());
            }
            Err(e) => {
                return Err(RiplError::CouldNotParseStringError {
                    value: e.to_string(),
                });
            }
        }
    }
}

pub fn query_float(rl: &mut Editor<(), FileHistory>, query: String) -> Result<f32, RiplError> {
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

                // Read requested nb
                let index_res = l.parse();
                if index_res.is_err() {
                    return Err(RiplError::CouldNotParseStringIntError { value: l });
                }
                return Ok(index_res.unwrap());
            }
            Err(e) => {
                return Err(RiplError::CouldNotParseStringError {
                    value: e.to_string(),
                });
            }
        }
    }
}

pub fn query_string(rl: &mut Editor<(), FileHistory>, query: String) -> Result<String, RiplError> {
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
                return Ok(l.trim().to_string());
            }
            Err(e) => {
                return Err(RiplError::CouldNotParseStringError {
                    value: e.to_string(),
                });
            }
        }
    }
}
