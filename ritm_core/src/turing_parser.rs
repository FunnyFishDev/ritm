use pest::{Parser, error::Error, iterators::Pair};
use pest_derive::Parser;
use std::{fmt::Display, fs, io};
use thiserror::Error;

use crate::{
    turing_graph::{
        DEFAULT_INIT_STATE, TuringGraph, TuringGraphError, TuringState, TuringStateType,
    },
    turing_machine::TuringMachineError,
    turing_transition::{TuringDirection, TuringTransition, TransitionMultRibbonInfo},
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
        turing_error: Box<TuringMachineError>,
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
/// Important to note that if the given string is empty, then a default [`TuringMachineGraph`] will be returned.
pub fn parse_turing_graph_file_path<S, T>(
    file_path: String,
) -> Result<TuringGraph<S, T>, TuringParserError>
where
    S: TuringState,
    T: TuringTransition,
{
    if file_path.trim().is_empty() {
        return Ok(TuringGraph::default());
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
pub fn parse_turing_graph_string<S, T>(
    turing_mach: String,
) -> Result<TuringGraph<S, T>, TuringParserError>
where
    S: TuringState,
    T: TuringTransition,
{
    let file = TuringGrammar::parse(Rule::turing_machine, &turing_mach);
    if let Err(e) = file {
        return Err(TuringParserError::ParsingError {
            line_col_pos: get_line_col(&e),
            value: e.line().to_string(),
            missing_value: get_expected_value(&e),
        });
    }
    let file = file.unwrap().next().unwrap(); // get and unwrap the `file` rule; never fails

    let mut turing_graph: Option<TuringGraph<S, T>> = None;
    let mut init_rule: Option<Pair<Rule>> = None;
    let mut accepting_state_rule: Option<Pair<Rule>> = None;

    for turing_machine_rule in file.into_inner() {
        let rule_cp = turing_machine_rule.clone();
        // Inside the 'turing_machine' rule, only two things can be matched : a transition (or multiple in one), and EOI
        let res = match turing_machine_rule.as_rule() {
            Rule::options => {
                if let Some(option_rule) = turing_machine_rule.into_inner().next() {
                    match &option_rule.as_rule() {
                        Rule::rename_initial => {
                            init_rule = Some(option_rule);
                        }
                        Rule::accepting_states => {
                            accepting_state_rule = Some(option_rule);
                        }
                        _ => unreachable!(),
                    };
                }

                Ok(())
            }
            // For every rule matched :
            Rule::transition_k => {
                let (from_var, transitions, to_var) = parse_transition(turing_machine_rule)?;

                /* Add the colected transitions to the MT */

                // This first transitions will determine the number of ribbons of the machine
                let graph = turing_graph.get_or_insert({
                    let mut g = TuringGraph::new(
                        transitions
                            .first()
                            .expect("at least one")
                            .get_number_of_affected_tapes()
                            - 1,
                        false,
                    )
                    .expect("correct machine");

                    if let Some(init_rule) = init_rule.take()
                        && let Err(e) = parse_rename_init(&mut g, init_rule)
                    {
                        return Err(TuringParserError::TuringError {
                            line_col_pos: Some(rule_cp.line_col()),
                            turing_error: Box::new(e.into()),
                            value: rule_cp.as_str().to_string(),
                        });
                    }
                    if let Some(accepting_state_rule) = accepting_state_rule.take()
                        && let Err(e) = parse_add_accepting_states(&mut g, accepting_state_rule)
                    {
                        return Err(TuringParserError::TuringError {
                            line_col_pos: Some(rule_cp.line_col()),
                            turing_error: Box::new(e.into()),
                            value: rule_cp.as_str().to_string(),
                        });
                    }

                    g
                });

                // Add the states to the mt (if they didn't already exists)
                // and get their index
                let var1 = graph.add_state(&from_var, TuringStateType::Normal);
                let var2 = graph.add_state(&to_var, TuringStateType::Normal);
                let mut inner_res = Ok(());
                // Adds all the collected transitions for these states
                for transition in transitions {
                    if let Err(e) = graph.append_transition(var1, transition, var2) {
                        inner_res = Err(e);
                        break;
                    }
                }
                inner_res
            }
            Rule::semicolon => Ok(()),
            // The input has ended, this means we reached the last matched rule
            Rule::EOI => Ok(()),
            _ => unreachable!(),
        };
        if let Err(e) = res {
            return Err(TuringParserError::TuringError {
                line_col_pos: Some(rule_cp.line_col()),
                turing_error: Box::new(e.into()),
                value: rule_cp.as_str().to_string(),
            });
        }
    }
    Ok(turing_graph.unwrap_or_default())
}

/// Parses a string containing a transition of the form :
/// * `q_i { transition } q_j`
/// * Or even :  `q_i { transition_0 | ... | transition_n } q_j`
///
/// Where each `transition` follows the form :  `a_0, a_1, ..., a_{n-1} -> D_0, b_1, D_1, b_2, D_2, ..., b_{n-1}, D_{n-1}`.
/// For more information look at the documentation of the structure [`TuringTransitionInfo`].
///
/// When giving multiple transitions, each one must affect the same number of tapes or an error will be returned.
pub fn parse_transition_string(
    to_parse: String,
) -> Result<(String, Vec<TransitionMultRibbonInfo>, String), TuringParserError> {
    let parsed = TuringGrammar::parse(Rule::transition_only_k, &to_parse);
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
) -> Result<TransitionMultRibbonInfo, TuringParserError> {
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

fn parse_rename_init<S, T>(
    tm: &mut TuringGraph<S, T>,
    init_rule: Pair<Rule>,
) -> Result<(), TuringGraphError>
where
    S: TuringState,
    T: TuringTransition,
{
    let mut state_name = String::from("i");

    if let Some(rule) = init_rule.into_inner().next() {
        match &rule.as_rule() {
            Rule::state_name => {
                state_name = parse_str_token(rule);
            }
            _ => unreachable!(),
        };
    }

    tm.rename_state(0, state_name)
}

fn parse_add_accepting_states<S, T>(
    tm: &mut TuringGraph<S, T>,
    init_rule: Pair<Rule>,
) -> Result<(), TuringGraphError>
where
    S: TuringState,
    T: TuringTransition,
{
    for rule in init_rule.into_inner() {
        match &rule.as_rule() {
            Rule::state_name => {
                tm.try_add_state(parse_str_token(rule), TuringStateType::Accepting)?;
            }
            _ => unreachable!(),
        };
    }
    Ok(())
}

fn parse_transition(
    rule: Pair<Rule>,
) -> Result<(String, Vec<TransitionMultRibbonInfo>, String), TuringParserError> {
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
            Rule::transition_content_k => {
                let rule_cp = rule.clone();
                // Add the transition
                let tr_res = parse_transition_content(rule);
                if let Err(e) = tr_res {
                    // explain in this error that we couldn't create the transition
                    // Return the col and line + the string content of the rule

                    return Err(TuringParserError::TuringError {
                        line_col_pos: Some(rule_cp.line_col()),
                        turing_error: Box::new(e),
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

fn parse_transition_content(rule: Pair<Rule>) -> Result<TransitionMultRibbonInfo, TuringMachineError> {
    let mut chars_read: Vec<char> = vec![];
    let mut directions: Vec<TuringDirection> = vec![];
    let mut chars_written: Vec<char> = vec![];

    // Parse all the informations
    for transition_rule in rule.into_inner() {
        match transition_rule.as_rule() {
            Rule::to_read_k => {
                // Parse all the characters to read
                for chars_rule in transition_rule.into_inner() {
                    // turns the rule into a string, then gets the first (and only) char
                    chars_read.push(chars_rule.as_str().chars().next().unwrap());
                }
            }
            Rule::to_write_move_k => {
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

    Ok(TransitionMultRibbonInfo::create(
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

/*
initial = qinit;
accepting = q_1, q_2;
q_init {ç, ç -> R, ç, R} q_1;
q1 {  0, _ -> R, a, R
   |  1, _ -> R, a, R} q3;
*/

/// Turns the given [`TuringGraph`] into its equivalent [String] value.
/// The returned value can then be parsed by the parser to return the same graph.
///
/// This function is therefore very useful to save graphs.
pub fn graph_to_string<S, T>(tm: &TuringGraph<S, T>) -> String
where
    S: TuringState,
    T: TuringTransition,
{
    let mut rename_str = String::new();
    {
        // Rename initial state if needed
        let init_state = tm.get_state(0).expect("present");
        if init_state.get_name() != DEFAULT_INIT_STATE {
            rename_str.push_str(format!("initial = q_{};\n", init_state.get_name()).as_str());
        }
    }

    let mut accepting_states = Vec::new();
    for state in tm.get_states() {
        if state.get_type() == TuringStateType::Accepting {
            accepting_states.push(state.get_name());
        }
    }
    if !accepting_states.is_empty() {
        rename_str.push_str("accepting = ");
        for name in accepting_states.iter().take(accepting_states.len() - 1) {
            rename_str.push_str(format!("q_{}, ", name).as_str());
        }
        rename_str
            .push_str(format!("q_{};\n", accepting_states.last().expect("one present")).as_str());
    }

    let mut transitions_str = String::new();
    // Print all transitions btw states
    for ((q1, q2), transitions) in tm.get_transitions_hashmap() {
        if transitions.is_empty() {
            continue;
        }
        let q1 = &tm.get_state_hashmap()[q1];
        let q2 = &tm.get_state_hashmap()[q2];
        transitions_str.push_str(format!("q_{} {} ", q1.get_name(), '{').as_str());
        let spaces = 3 + q1.get_name().len();
        for transition in transitions.iter().take(transitions.len() - 1) {
            transitions_str
                .push_str(format!("{} \n{}| ", transition.info, " ".repeat(spaces)).as_str());
        }
        // add last
        transitions_str.push_str(format!("{} ", transitions.last().unwrap().info).as_str());

        transitions_str.push_str(format!("{} q_{};\n\n", "}", q2.get_name()).as_str());
    }

    if !rename_str.is_empty() {
        rename_str.push('\n');
    }

    format!("{rename_str}{transitions_str}")
}
