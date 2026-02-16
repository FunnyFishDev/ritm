use ritm_core::{
    EmptyState, EmptyTransition, SimpleTuringGraph,
    turing_graph::TuringStateType,
    turing_machine::TuringMachineError,
    turing_parser::{self, TuringParserError, parse_transition_string, parse_turing_graph_string},
    turing_transition::{TuringDirection, TuringTransitionError, TransitionMultRibbonInfo},
};

#[test]
fn test_parse_mt_valid() {
    let machine = String::from(
        "q_i {ç, ç -> R, ç, R} q_1;
                                        q1 {0, _ -> R, a, R 
                                          | 1, _ -> R, a, R} q1;",
    );

    let res = parse_turing_graph_string::<EmptyState, EmptyTransition>(machine);
    let parsed_graph = res.unwrap();

    // Compare to a real turing machine
    let mut graph = SimpleTuringGraph::default();

    let q1 = &String::from("1");
    graph.add_state(q1, TuringStateType::Normal);

    // q_i -> {ç, ç, => R, ç, R} -> q_1
    let mut transition = TransitionMultRibbonInfo::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::Right, TuringDirection::Right],
    )
    .unwrap();
    graph
        .append_transition("i", transition.clone(), q1)
        .unwrap();

    transition = TransitionMultRibbonInfo::create(
        vec!['0', '_'],
        vec!['a'],
        vec![TuringDirection::Right, TuringDirection::Right],
    )
    .unwrap();
    graph.append_transition(q1, transition.clone(), q1).unwrap();

    transition = TransitionMultRibbonInfo::create(
        vec!['1', '_'],
        vec!['a'],
        vec![TuringDirection::Right, TuringDirection::Right],
    )
    .unwrap();
    graph.append_transition(q1, transition.clone(), q1).unwrap();

    assert_eq!(parsed_graph.get_k(), graph.get_k());
    assert_eq!(parsed_graph.get_state_hashmap(), graph.get_state_hashmap());
}

#[test]
fn test_parse_mt_not_valid() {
    let machine_str = String::from("q_i {ç, ç -> R, ç, R} q_1");

    if let Ok(t) = parse_turing_graph_string::<EmptyState, EmptyTransition>(machine_str) {
        panic!(
            "The parser should have returned an error not this value:  {:?}",
            t
        )
    }

    let machine_str = String::from("q_i ç, ç -> R, ç, R} q_1;");

    if let Ok(t) = parse_turing_graph_string::<EmptyState, EmptyTransition>(machine_str) {
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
        TransitionMultRibbonInfo::new(
            vec!('0', 'a'),
            TuringDirection::Right,
            vec!(('a', TuringDirection::Left))
        )
        .expect("no issues")
    );
    assert_eq!(
        transitions[1],
        TransitionMultRibbonInfo::new(
            vec!('1', 'b'),
            TuringDirection::None,
            vec!(('p', TuringDirection::Right))
        )
        .expect("no issues")
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
        TransitionMultRibbonInfo::new(
            vec!('0', 'a'),
            TuringDirection::Right,
            vec!(('a', TuringDirection::Left))
        )
        .expect("no issues")
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

    let res = parse_turing_graph_string::<EmptyState, EmptyTransition>(machine);

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

    let res = parse_turing_graph_string::<EmptyState, EmptyTransition>(machine);

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

    let res = parse_turing_graph_string::<EmptyState, EmptyTransition>(machine);

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

    let res = parse_turing_graph_string::<EmptyState, EmptyTransition>(machine);
    match res {
        Ok(_) => panic!("An error was expected"),
        Err(TuringParserError::TuringError {
            line_col_pos: _,
            turing_error,
            value: _,
        }) => {
            matches!(
                *turing_error,
                TuringMachineError::GraphError(
                    ritm_core::turing_graph::TuringGraphError::IncompatibleTransitionError {
                        expected: _,
                        received: _
                    }
                )
            );
        }
        Err(_) => unreachable!("This is not suppose to happen"),
    }
}

#[test]
fn test_parse_graph_bad_transition() {
    let machine = String::from(
        "q_i {ç, ç -> R, ç, R} q_1;
                                        q1 {0, _ -> R, a, R 
                                          | 1, _ -> R, a, R, a, R} q1;",
    );

    let res = parse_turing_graph_string::<EmptyState, EmptyTransition>(machine);

    match res {
        Ok(_) => panic!("An error was expected"),
        Err(e) => match e {
            TuringParserError::TuringError {
                line_col_pos: _,
                turing_error,
                value: _,
            } => {
                assert!(matches!(
                    *turing_error,
                    TuringMachineError::TransitionError(
                        TuringTransitionError::TransitionArgsError(_val)
                    )
                ))
            }
            _ => panic!("An EncounteredTuringError was expected"),
        },
    }
}

#[test]
fn test_rename_init() {
    let machine = String::from(
        "initial = q_init;
        q_init {ç, ç -> R, ç, R} q_1;
        q1 {  0, _ -> R, a, R 
           |  1, _ -> R, a, R} q1;",
    );

    let res = parse_turing_graph_string::<EmptyState, EmptyTransition>(machine);
    let parsed_graph = res.expect("no errors");

    assert_eq!(
        parsed_graph.get_state("init").expect("no problem").get_id(),
        0
    )
}

#[test]
fn test_graph_to_string() {
    // This test checks that we can parse a graph, turn it into a string, parse it again and end up with the same graph
    let machine = String::from(
        "initial = q_init;
        accepting = q_a, q_e, q_i;
        q_init {ç, ç -> R, ç, R} q_1;
        q1 {  0, _ -> R, a, R 
           |  1, _ -> R, a, R} q1;",
    );

    let parsed_graph =
        parse_turing_graph_string::<EmptyState, EmptyTransition>(machine).expect("no errors");

    let str_graph = turing_parser::graph_to_string(&parsed_graph);

    assert_eq!(
        parsed_graph,
        parse_turing_graph_string::<EmptyState, EmptyTransition>(str_graph).expect("no errors")
    );
}

#[test]
fn test_add_accepting() {
    let machine = String::from(
        "initial = qinit;
        accepting = q_1, q_2;
        q_init {ç, ç -> R, ç, R} q_1;
        q1 {  0, _ -> R, a, R 
           |  1, _ -> R, a, R} q3;",
    );

    let res = parse_turing_graph_string::<EmptyState, EmptyTransition>(machine);
    let parsed_graph = res.expect("no errors");

    assert_eq!(
        parsed_graph
            .get_state("1")
            .expect("no problem")
            .get_info()
            .get_type(),
        TuringStateType::Accepting,
    );
    assert_eq!(
        parsed_graph
            .get_state("2")
            .expect("no problem")
            .get_info()
            .get_type(),
        TuringStateType::Accepting,
    );
    assert_eq!(
        parsed_graph
            .get_state("3")
            .expect("no problem")
            .get_info()
            .get_type(),
        TuringStateType::Normal,
    )
}

#[test]
fn test_parse_machine_k_ribbons() {
    // This test checks that we can parse a graph, turn it into a string, parse it again and end up with the same graph
    let machine = String::from(
        "accepting = q_a;
         q_i {ç, ç, ç -> R, ç, R, ç, R} q_1;
         q1 {  0, _, _ -> R, a, R, a, R 
            |  1, _, _ -> R, a, R, a, R} q1;",
    );

    let parsed_graph =
        parse_turing_graph_string::<EmptyState, EmptyTransition>(machine).expect("no errors");

    let str_graph = turing_parser::graph_to_string(&parsed_graph);

    assert_eq!(
        parsed_graph,
        parse_turing_graph_string::<EmptyState, EmptyTransition>(str_graph).expect("no errors")
    );
}
