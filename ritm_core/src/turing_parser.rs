use pest::{Parser, error::Error, iterators::Pair};
use pest_derive::Parser;
use std::{fmt::Display, fs, io};
use thiserror::Error;

use crate::{
    turing_graph::TuringMachineGraph,
    turing_machine::TuringMachineError,
    turing_transition::{TuringDirection, TuringTransition},
};

#[derive(Debug, Error)]
pub enum TuringParserError {
    FileError {
        given_path: String,
        error: io::Error,
    },
    /// Error when failing to parse a given string value
    ParsingError {
        line_col_pos: Option<(usize, usize)>,
        value: String,
        missing_value: Option<String>,
    },
    /// Error when a [`TuringMachineError`] was encountered **while** parsing a string value
    TuringError {
        line_col_pos: Option<(usize, usize)>,
        turing_error: TuringMachineError,
        value: String,
    },
}

impl Display for TuringParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", {
            match self {
                TuringParserError::FileError { given_path, error } => format!(
                    "Ran into an error trying to open the file at \"{}\". The reason being : {}",
                    given_path, error
                ),
                TuringParserError::ParsingError {
                    line_col_pos,
                    value,
                    missing_value,
                } => format!(
                    "Impossible to parse the given input.\n{}{}",
                    get_arrow_under(value, line_col_pos),
                    {
                        if let Some(token) = missing_value {
                            format!("\nThis token might be missing : \"{}\"", token)
                        } else {
                            String::new()
                        }
                    }
                ),
                TuringParserError::TuringError {
                    line_col_pos,
                    turing_error,
                    value,
                } => format!(
                    "Encountered an error at the following line: \n{}\nReason: {}",
                    get_arrow_under(value, line_col_pos),
                    turing_error
                ),
            }
        })
    }
}

fn get_arrow_under(value: &String, line_col_pos: &Option<(usize, usize)>) -> String {
    if let Some((line, col)) = line_col_pos {
        let line_str = (line).to_string();
        format!(
            "{line_str}: {value}\n{}{}^",
            String::from(" ").repeat(line_str.len() + 2),
            String::from("-").repeat(col - 1)
        )
    } else {
        value.to_string()
    }
}

#[derive(Parser)]
#[grammar = "turing_machine.pest"]
pub struct TuringGrammar;

/// Parses a turing machine graph from the content of a file.
///
/// Important to note that if the given string is empty, then an empty [`TuringMachineGraph`] with a *k* of 1 is returned.
pub fn parse_turing_graph_file_path(
    file_path: String,
) -> Result<TuringMachineGraph, TuringParserError> {
    if file_path.trim().is_empty() {
        match TuringMachineGraph::new(1) {
            Ok(tm) => return Ok(tm),
            Err(e) => {
                return Err(TuringParserError::TuringError {
                    line_col_pos: None,
                    turing_error: e.into(),
                    value: String::new(),
                });
            }
        };
    }
    match fs::read_to_string(&file_path) {
        Ok(unparsed_file) => parse_turing_graph_string(unparsed_file),
        Err(error) => Err(TuringParserError::FileError {
            given_path: file_path,
            error,
        }),
    }
}

/// Parses a turing machine graph from the content of a string.
///
/// Important to note that if the given string is empty, then an empty [TuringMachineGraph] with a *k* of 1 is returned.
pub fn parse_turing_graph_string(
    turing_mach: String,
) -> Result<TuringMachineGraph, TuringParserError> {
    let file = TuringGrammar::parse(Rule::turing_machine, &turing_mach);
    if let Err(e) = file {
        return Err(TuringParserError::ParsingError {
            line_col_pos: get_line_col(&e),
            value: e.line().to_string(),
            missing_value: get_expected_value(&e),
        });
    }
    let file = file.unwrap().next().unwrap(); // get and unwrap the `file` rule; never fails

    let mut turing_machine: Option<TuringMachineGraph> = None;

    for turing_machine_rule in file.into_inner() {
        let rule_cp = turing_machine_rule.clone();
        // Inside the 'turing_machine' rule, only two things can be matched : a transition (or multiple in one), and EOI
        match turing_machine_rule.as_rule() {
            // For every rule matched :
            Rule::transition => {
                let (from_var, transitions, to_var) = parse_transition(turing_machine_rule)?;

                /* Add the colected transitions to the MT */

                // If the MT doesn't already exists, create it
                if turing_machine.is_none() {
                    // With the collected number of tapes
                    let tm = TuringMachineGraph::new(
                        transitions
                            .first()
                            .expect("At least one rule should be given in a transition")
                            .get_number_of_affected_tapes()
                            - 1,
                    );

                    if let Err(e) = tm {
                        return Err(TuringParserError::TuringError {
                            line_col_pos: Some(rule_cp.line_col()),
                            turing_error: e.into(),
                            value: rule_cp.as_str().to_string(),
                        });
                    }
                    turing_machine = Some(tm.unwrap());
                }
                // If the MT existed
                if let Some(mt) = &mut turing_machine {
                    // Add the states to the mt (if they didn't already exists)
                    // and get their index
                    let var1 = mt.add_state(&from_var);
                    let var2 = mt.add_state(&to_var);
                    // Adds all the collected transitions for these states
                    for transition in transitions {
                        if let Err(e) = mt.append_rule_state(var1, transition, var2) {
                            return Err(TuringParserError::TuringError {
                                line_col_pos: Some(rule_cp.line_col()),
                                turing_error: e.into(),
                                value: rule_cp.as_str().to_string(),
                            });
                        }
                    }
                }
            }
            Rule::semicolon => {}
            // The file has ended, this means we reached the last matched rule
            Rule::EOI => {}
            _ => unreachable!(),
        }
    }
    match turing_machine {
        Some(t) => Ok(t),
        // If no parse value was given, simply return a read only one
        None => Ok(TuringMachineGraph::new(1).unwrap()),
    }
}

/// Parses a string containing a transition of the form :
/// * `q_i { transition } q_j`
/// * Or even :  `q_i { transition_0 | ... | transition_n } q_j`
///
/// Where each `transition` follows the form :  `a_0, a_1, ..., a_{n-1} -> D_0, b_1, D_1, b_2, D_2, ..., b_{n-1}, D_{n-1}`.
/// For more information look at the documentation of the structure [`TuringTransition`].
///
/// When giving multiple transitions, each one must affect the same number of tapes or an error will be returned.
pub fn parse_transition_string(
    to_parse: String,
) -> Result<(String, Vec<TuringTransition>, String), TuringParserError> {
    let parsed = TuringGrammar::parse(Rule::transition_only, &to_parse);
    if let Err(e) = parsed {
        return Err(TuringParserError::ParsingError {
            line_col_pos: get_line_col(&e),
            missing_value: get_expected_value(&e),
            value: e.line().to_string(),
        });
    }
    parse_transition(parsed.unwrap().next().unwrap())
}

/// Parses a string containing the content of a transition of the form : `a_0, a_1, ..., a_{n-1} -> D_0, b_1, D_1, b_2, D_2, ..., b_{n-1}, D_{n-1}`
/// For more information look at the documentation of the structure [TuringTransition]
pub fn parse_transition_content_string(
    transition: String,
) -> Result<TuringTransition, TuringParserError> {
    let parsed = TuringGrammar::parse(Rule::turing_machine, &transition);
    if let Err(e) = parsed {
        return Err(TuringParserError::ParsingError {
            line_col_pos: get_line_col(&e),
            missing_value: get_expected_value(&e),
            value: e.line().to_string(),
        });
    }
    todo!("test");
    // FIXME: test this
}

fn parse_transition(
    rule: Pair<Rule>,
) -> Result<(String, Vec<TuringTransition>, String), TuringParserError> {
    let mut transitions = vec![];
    let mut to_var = String::new();
    let mut from_var = String::new();

    for rule in rule.into_inner() {
        // Inside a rule, there are :
        // * two state names (var1 & var2)
        // * one or more transitions btw them
        match rule.as_rule() {
            // Get var1 & var2
            Rule::state_name => {
                if from_var.is_empty() {
                    from_var = parse_str_token(rule);
                } else {
                    to_var = parse_str_token(rule);
                }
            }
            // Read all transitions
            Rule::transition_content => {
                let rule_cp = rule.clone();
                // Add the transition
                let tr_res = parse_transition_content(rule);
                if let Err(e) = tr_res {
                    // explain in this error that we couldn't create the transition
                    // Return the col and line + the string content of the rule

                    return Err(TuringParserError::TuringError {
                        line_col_pos: Some(rule_cp.line_col()),
                        turing_error: e,
                        value: rule_cp.as_str().to_string(),
                    });
                }
                transitions.push(tr_res.unwrap());
            }
            Rule::left_bracket | Rule::right_bracket => {}
            _ => unreachable!(),
        }
    }

    Ok((from_var, transitions, to_var))
}

fn parse_str_token(rule: Pair<Rule>) -> String {
    match rule.as_rule() {
        Rule::state_name | Rule::str => rule.into_inner().as_str().trim().to_string(),
        _ => unreachable!(),
    }
}

fn parse_transition_content(rule: Pair<Rule>) -> Result<TuringTransition, TuringMachineError> {
    let mut chars_read: Vec<char> = vec![];
    let mut directions: Vec<TuringDirection> = vec![];
    let mut chars_written: Vec<char> = vec![];

    // Parse all the informations
    for transition_rule in rule.into_inner() {
        match transition_rule.as_rule() {
            Rule::to_read => {
                // Parse all the characters to read
                for chars_rule in transition_rule.into_inner() {
                    // turns the rule into a string, then gets the first (and only) char
                    chars_read.push(chars_rule.as_str().chars().next().unwrap());
                }
            }
            Rule::to_write_move => {
                for write_move_rule in transition_rule.into_inner() {
                    match write_move_rule.as_rule() {
                        Rule::dir_left => {
                            directions.push(TuringDirection::Left);
                        }
                        Rule::dir_right => {
                            directions.push(TuringDirection::Right);
                        }
                        Rule::dir_none => {
                            directions.push(TuringDirection::None);
                        }
                        Rule::char | Rule::special_chars => {
                            chars_written.push(write_move_rule.as_str().chars().next().unwrap());
                        }
                        _ => unreachable!(),
                    };
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(TuringTransition::create(
        chars_read,
        chars_written,
        directions,
    )?)
}

fn get_expected_value(error: &Error<Rule>) -> Option<String> {
    match &error.variant {
        pest::error::ErrorVariant::ParsingError {
            positives,
            negatives: _,
        } => {
            for r in positives {
                let char = match r {
                    Rule::left_bracket => Some("{"),
                    Rule::right_bracket => Some("}"),
                    Rule::semicolon => Some(";"),
                    _ => None,
                };
                if let Some(c) = char {
                    return Some(c.to_string());
                }
            }
            None
        }
        pest::error::ErrorVariant::CustomError { message: _ } => None,
    }
}

fn get_line_col(error: &Error<Rule>) -> Option<(usize, usize)> {
    match &error.line_col {
        pest::error::LineColLocation::Pos(p) => Some((p.0, p.1)),
        pest::error::LineColLocation::Span(_, _) => None,
    }
}

/// Turns the given [TuringMachineGraph] into its equivalent [String] value.
/// The returned value can then be parsed by the parser to return the same graph.
///
/// This function is therefore very useful to save graphs.
pub fn graph_to_string(tm: &TuringMachineGraph) -> String {
    let mut res = String::new();

    // Print all transitions btw states
    for (q1, i1) in tm.get_name_index_hashmap() {
        for (q2, i2) in tm.get_name_index_hashmap() {
            let transitions = tm.get_transitions_by_index(*i1, *i2).unwrap();
            if transitions.is_empty() {
                continue;
            }
            res.push_str(format!("q_{} {} ", q1, '{').as_str());
            let spaces = 3 + q1.len();

            for i in 0..transitions.len() - 1 {
                res.push_str(
                    format!("{} \n{}| ", transitions.get(i).unwrap(), " ".repeat(spaces)).as_str(),
                );
            }
            // add last
            res.push_str(format!("{} ", transitions.last().unwrap()).as_str());

            res.push_str(format!("{} q_{};\n\n", "}", q2).as_str());
        }
    }
    if !res.is_empty() {
        // remove extra '\n'
        res.pop().unwrap();
    }

    res
}
