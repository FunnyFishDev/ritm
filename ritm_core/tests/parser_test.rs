use ritm_core::{
    turing_errors::{TuringError, TuringParserError},
    turing_graph::TuringMachineGraph,
    turing_parser::{parse_transition_string, parse_turing_graph_string},
    turing_state::{TuringDirection, TuringTransition},
};

#[test]
fn test_parse_mt_valid() {
    let machine = String::from(
        "q_i {ç, ç -> R, ç, R} q_1;
                                        q1 {0, _ -> R, a, R 
                                          | 1, _ -> R, a, R} q1;",
    );

    let res = parse_turing_graph_string(machine);
    let parsed_graph = res.unwrap();

    // Compare to a real turing machine
    let mut graph = TuringMachineGraph::new(1).unwrap();

    let q1 = &String::from("1");
    graph.add_state(q1);

    // q_i -> {ç, ç, => R, ç, R} -> q_1
    let mut transition = TuringTransition::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::Right, TuringDirection::Right],
    )
    .unwrap();
    graph
        .append_rule_state_by_name(&String::from("i"), transition.clone(), q1)
        .unwrap();

    transition = TuringTransition::create(
        vec!['0', '_'],
        vec!['a'],
        vec![TuringDirection::Right, TuringDirection::Right],
    )
    .unwrap();
    graph
        .append_rule_state_by_name(q1, transition.clone(), q1)
        .unwrap();

    transition = TuringTransition::create(
        vec!['1', '_'],
        vec!['a'],
        vec![TuringDirection::Right, TuringDirection::Right],
    )
    .unwrap();
    graph
        .append_rule_state_by_name(q1, transition.clone(), q1)
        .unwrap();

    assert_eq!(parsed_graph.get_k(), graph.get_k());
    assert_eq!(parsed_graph.get_states(), graph.get_states());
}

#[test]
fn test_parse_mt_not_valid() {
    let machine_str = String::from("q_i {ç, ç -> R, ç, R} q_1");

    if let Ok(t) = parse_turing_graph_string(machine_str) {
        panic!(
            "The parser should have returned an error not this value:  {:?}",
            t
        )
    }

    let machine_str = String::from("q_i ç, ç -> R, ç, R} q_1;");

    if let Ok(t) = parse_turing_graph_string(machine_str) {
        panic!(
            "The parser should have returned an error not this value:  {:?}",
            t
        )
    }
}

#[test]
fn test_parse_transition_valid_mult() {
    let transition_str = String::from(
        "q_i { 0, a -> R, a, L
                                            | 1, b -> N, p, R} q_2",
    );

    let (from, transitions, to) = parse_transition_string(transition_str).unwrap();

    assert_eq!(String::from("i"), from);
    assert_eq!(String::from("2"), to);

    assert_eq!(transitions.len(), 2);
    assert_eq!(
        transitions[0],
        TuringTransition::new(
            vec!('0', 'a'),
            TuringDirection::Right,
            vec!(('a', TuringDirection::Left))
        )
    );
    assert_eq!(
        transitions[1],
        TuringTransition::new(
            vec!('1', 'b'),
            TuringDirection::None,
            vec!(('p', TuringDirection::Right))
        )
    );
}

#[test]
fn test_parse_transition_valid_single() {
    let transition_str = String::from("qi { 0, a -> R, a, L } q2");

    let (from, transitions, to) = parse_transition_string(transition_str).unwrap();

    assert_eq!(String::from("i"), from);
    assert_eq!(String::from("2"), to);

    assert_eq!(transitions.len(), 1);
    assert_eq!(
        transitions[0],
        TuringTransition::new(
            vec!('0', 'a'),
            TuringDirection::Right,
            vec!(('a', TuringDirection::Left))
        )
    );
}

#[test]
fn test_parse_transition_fail() {
    let transition_str = String::from(
        "q_2 { 0, a -> R, a, L
                                            | 1 -> R, a, L} q_2",
    );

    if let Ok(t) = parse_transition_string(transition_str) {
        panic!(
            "The parser should have returned an error not this value:  {:?}",
            t
        )
    }

    let transition_str = String::from(
        "q_2 { 0, a -> R, a, L
                                            | 1, a-> R, a, L} q_2;",
    );

    if let Ok(t) = parse_transition_string(transition_str) {
        panic!(
            "The parser should have returned an error not this value:  {:?}",
            t
        )
    }

    let transition_str = String::from("");

    if let Ok(t) = parse_transition_string(transition_str) {
        panic!(
            "The parser should have returned an error not this value:  {:?}",
            t
        )
    }
}

#[test]
fn test_parser_missing_semicolon() {
    let machine = String::from(
        "q_i {ç, ç -> R, ç, R} q_1;
                                        q1 {0, _ -> R, a, R 
                                          | 1, _ -> R, a, R} q1",
    );

    let res = parse_turing_graph_string(machine);

    match res {
        Ok(_) => panic!("An error was expected"),
        Err(e) => match e {
            TuringParserError::ParsingError {
                line_col_pos: _,
                value: _,
                missing_value,
            } => {
                assert_eq!(
                    missing_value.expect("Expected a missing char to be found"),
                    String::from(";")
                )
            }
            _ => panic!("A parsing error was expected"),
        },
    }
}

#[test]
fn test_parser_missing_left_bracket() {
    let machine = String::from(
        "q_i ç, ç -> R, ç, R} q_1;
                                        q1 {0, _ -> R, a, R 
                                          | 1, _ -> R, a, R} q1;",
    );

    let res = parse_turing_graph_string(machine);

    match res {
        Ok(_) => panic!("An error was expected"),
        Err(e) => match e {
            TuringParserError::ParsingError {
                line_col_pos: _,
                value: _,
                missing_value,
            } => {
                assert_eq!(
                    missing_value.expect("Expected a missing char to be found"),
                    String::from("{")
                )
            }
            _ => panic!("A parsing error was expected"),
        },
    }
}

#[test]
fn test_parser_missing_right_bracket() {
    let machine = String::from(
        "q_i {ç, ç -> R, ç, R} q_1;
                                        q1 {0, _ -> R, a, R 
                                          | 1, _ -> R, a, R q1;",
    );

    let res = parse_turing_graph_string(machine);

    match res {
        Ok(_) => panic!("An error was expected"),
        Err(e) => match e {
            TuringParserError::ParsingError {
                line_col_pos: _,
                value: _,
                missing_value,
            } => {
                assert_eq!(
                    missing_value.expect("Expected a missing char to be found"),
                    String::from("}")
                )
            }
            _ => panic!("A parsing error was expected"),
        },
    }
}

#[test]
fn test_parse_graph_incompatible_transition() {
    let machine = String::from(
        "q_i {ç, ç -> R, ç, R} q_1;
                                        q1 {0, _ -> R, a, R 
                                          | 1, _, _ -> R, a, R, a, R} q1;",
    );

    let res = parse_turing_graph_string(machine);
    match res {
        Ok(_) => panic!("An error was expected"),
        Err(e) => match e {
            TuringParserError::EncounteredTuringError {
                line_col_pos: _,
                turing_error,
                value: _,
            } => match turing_error {
                TuringError::IncompatibleTransitionError {
                    expected: _,
                    received: _,
                } => (),
                _ => panic!("An IncompatibleTransitionError was expected"),
            },
            _ => panic!("An EncounteredTuringError was expected"),
        },
    }
}

#[test]
fn test_parse_graph_bad_transition() {
    let machine = String::from(
        "q_i {ç, ç -> R, ç, R} q_1;
                                        q1 {0, _ -> R, a, R 
                                          | 1, _ -> R, a, R, a, R} q1;",
    );

    let res = parse_turing_graph_string(machine);
    match res {
        Ok(_) => panic!("An error was expected"),
        Err(e) => match e {
            TuringParserError::EncounteredTuringError {
                line_col_pos: _,
                turing_error,
                value: _,
            } => match turing_error {
                TuringError::TransitionArgsError { reason: _ } => (),
                _ => panic!("An IncompatibleTransitionError was expected"),
            },
            _ => panic!("An EncounteredTuringError was expected"),
        },
    }
}
