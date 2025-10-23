use ritm_core::{
    turing_errors::TuringError,
    turing_graph::TuringMachineGraph,
    turing_state::{
        TuringDirection::{self},
        TuringStateType, TuringTransition,
    },
};

#[test]
fn create_graph_test() {
    let graph = TuringMachineGraph::new(2).unwrap();

    assert_eq!(*graph.get_name_index_hashmap().get("i").unwrap(), 0);
    assert_eq!(*graph.get_name_index_hashmap().get("a").unwrap(), 1);
    assert_eq!(*graph.get_name_index_hashmap().get("r").unwrap(), 2);

    expect_illegal_action_error(TuringMachineGraph::new(0));
    // Check the final states
    assert_eq!(
        TuringStateType::Accepting,
        graph
            .get_state_from_name(&"a".to_string())
            .unwrap()
            .state_type
    );
    assert_eq!(
        TuringStateType::Rejecting,
        graph
            .get_state_from_name(&"r".to_string())
            .unwrap()
            .state_type
    );
    assert_eq!(
        TuringStateType::Normal,
        graph
            .get_state_from_name(&"i".to_string())
            .unwrap()
            .state_type
    );

    TuringMachineGraph::new(1).unwrap();
}

#[test]
fn delete_init_nodes_test() {
    let mut graph = TuringMachineGraph::new(2).unwrap();

    expect_illegal_action_error(graph.remove_state_with_name(&"i".to_string()));
    expect_illegal_action_error(graph.remove_state_with_name(&"a".to_string()));
    expect_illegal_action_error(graph.remove_state_with_name(&"r".to_string()));
}

#[test]
fn add_nodes() {
    let mut graph = TuringMachineGraph::new(1).unwrap();

    // Check they already exists
    assert_eq!(graph.add_state(&"i".to_string()), 0);
    assert_eq!(graph.add_state(&"a".to_string()), 1);
    assert_eq!(graph.add_state(&"r".to_string()), 2);

    // Add new ones
    assert_eq!(graph.add_state(&"b".to_string()), 3);
    assert_eq!(graph.add_state(&"c".to_string()), 4);
    assert_eq!(graph.add_state(&"d".to_string()), 5);
    // Check they got the correct index
    assert_eq!(graph.add_state(&"b".to_string()), 3);
    assert_eq!(graph.add_state(&"c".to_string()), 4);
    assert_eq!(graph.add_state(&"d".to_string()), 5);
}

#[test]
fn get_nodes_test() {
    let mut graph = TuringMachineGraph::new(1).unwrap();
    // Add new nodes
    assert_eq!(graph.add_state(&"b".to_string()), 3);
    assert_eq!(graph.add_state(&"c".to_string()), 4);
    assert_eq!(graph.add_state(&"d".to_string()), 5);

    // check they get be obtained
    assert_eq!(graph.get_state(3).unwrap().name.clone(), "b".to_string());
    assert_eq!(graph.get_state(4).unwrap().name.clone(), "c".to_string());
    assert_eq!(graph.get_state(5).unwrap().name.clone(), "d".to_string());

    // check they get be obtained
    assert_eq!(
        graph
            .get_state_from_name(&"b".to_string())
            .unwrap()
            .name
            .clone(),
        "b".to_string()
    );
    assert_eq!(
        graph
            .get_state_from_name(&"c".to_string())
            .unwrap()
            .name
            .clone(),
        "c".to_string()
    );
    assert_eq!(
        graph
            .get_state_from_name(&"d".to_string())
            .unwrap()
            .name
            .clone(),
        "d".to_string()
    );

    // Check they aren't final
    assert_eq!(
        TuringStateType::Normal,
        graph
            .get_state_from_name(&"b".to_string())
            .unwrap()
            .state_type
    );
    assert_eq!(
        TuringStateType::Normal,
        graph
            .get_state_from_name(&"c".to_string())
            .unwrap()
            .state_type
    );
    assert_eq!(
        TuringStateType::Normal,
        graph
            .get_state_from_name(&"d".to_string())
            .unwrap()
            .state_type
    );
}

#[test]
fn add_transition() {
    let mut graph = TuringMachineGraph::new(1).unwrap();

    graph
        .append_rule_state_by_name(
            &String::from("i"),
            TuringTransition::create(
                vec!['ç', 'ç'],
                vec!['ç'],
                vec![TuringDirection::None, TuringDirection::Right],
            )
            .unwrap(),
            &String::from("a"),
        )
        .expect("no errors were expected");

    // e, is not part of the graph
    expect_unk_name_error(
        graph.append_rule_state_by_name(
            &String::from("e"),
            TuringTransition::create(
                vec!['ç', 'ç'],
                vec!['ç'],
                vec![TuringDirection::None, TuringDirection::Right],
            )
            .unwrap(),
            &String::from("a"),
        ),
    );
    // o, is not part of the graph
    expect_unk_name_error(
        graph.append_rule_state_by_name(
            &String::from("a"),
            TuringTransition::create(
                vec!['ç', 'ç'],
                vec!['ç'],
                vec![TuringDirection::None, TuringDirection::Right],
            )
            .unwrap(),
            &String::from("o"),
        ),
    );
    // add e and o to the graph
    graph.add_state(&String::from("e"));
    graph.add_state(&String::from("o"));

    // Check that the transition didn't already exists
    // Check that the transition was really added
    if !graph
        .get_transition_indexes_by_name(&String::from("e"), &String::from("o"))
        .expect("a value was expected here")
        .is_empty()
    {
        panic!("No values were expected");
    }
    // add transition
    graph
        .append_rule_state_by_name(
            &String::from("e"),
            TuringTransition::create(
                vec!['ç', 'ç'],
                vec!['ç'],
                vec![TuringDirection::None, TuringDirection::Right],
            )
            .unwrap(),
            &String::from("o"),
        )
        .expect("no errors were expected");

    // Check that the transition was really added
    if graph
        .get_transition_indexes_by_name(&String::from("e"), &String::from("o"))
        .expect("a value was expected here")
        .is_empty()
    {
        panic!("A value should be here");
    }

    // Add the same transition again
    // TODO check this
}

#[test]
fn delete_transitions() {
    let mut graph = TuringMachineGraph::new(1).unwrap();
    let t1 = TuringTransition::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();
    let t2 = TuringTransition::create(
        vec!['ç', '_'],
        vec!['_'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();

    graph
        .append_rule_state_by_name(&String::from("i"), t1.clone(), &String::from("a"))
        .unwrap();
    graph
        .append_rule_state_by_name(&String::from("i"), t2.clone(), &String::from("a"))
        .unwrap();

    expect_unk_name_error(graph.remove_transition(&String::from("i"), &t1, &String::from("p")));
    expect_unk_name_error(graph.remove_transition(&String::from("d"), &t1, &String::from("a")));

    // Remove transition
    graph
        .remove_transition(&String::from("i"), &t1, &String::from("a"))
        .unwrap();

    // Check that it was indeed removed
    assert!(
        graph
            .get_state(0)
            .unwrap()
            .get_valid_transitions(&vec!('ç', 'ç'))
            .is_empty()
    );
    // and that the other one is still present
    assert_eq!(
        **graph
            .get_state(0)
            .unwrap()
            .get_valid_transitions(&vec!('ç', '_'))
            .first()
            .unwrap(),
        t2
    );
}

#[test]
fn delete_all_transitions_two_nodes() {
    let mut graph = TuringMachineGraph::new(1).unwrap();
    let t1 = TuringTransition::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();
    let t2 = TuringTransition::create(
        vec!['ç', '_'],
        vec!['_'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();
    let t3 = TuringTransition::create(
        vec!['_', '_'],
        vec!['_'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();

    graph
        .append_rule_state_by_name(&String::from("i"), t1.clone(), &String::from("a"))
        .unwrap();
    graph
        .append_rule_state_by_name(&String::from("i"), t2.clone(), &String::from("a"))
        .unwrap();
    graph
        .append_rule_state_by_name(&String::from("i"), t3.clone(), &String::from("i"))
        .unwrap(); // i -> i

    // Removes all transitions btw 'i' and 'a'
    graph
        .remove_transitions(&String::from("i"), &String::from("a"))
        .unwrap();

    // (note: index of 'i' is 0)
    assert!(graph.get_state(0).unwrap().get_transitions_to(1).is_empty());

    // check that i -> i, is still here
    assert_eq!(
        *graph
            .get_state(0)
            .unwrap()
            .get_transitions_to(0)
            .first()
            .unwrap(),
        &t3
    );
}

#[test]
fn delete_node() {
    let mut graph = TuringMachineGraph::new(1).unwrap();
    let t1 = TuringTransition::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();
    let t2 = TuringTransition::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();
    let t3 = TuringTransition::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();

    let _ = graph.add_state(&String::from("t"));
    let ind_p = graph.add_state(&String::from("p"));
    let ind_q = graph.add_state(&String::from("q"));

    graph
        .append_rule_state_by_name(&String::from("t"), t1.clone(), &String::from("a"))
        .unwrap(); // t -> a
    graph
        .append_rule_state_by_name(&String::from("r"), t2.clone(), &String::from("t"))
        .unwrap(); // r -> t
    graph
        .append_rule_state_by_name(&String::from("q"), t3.clone(), &String::from("t"))
        .unwrap(); // q -> t
    graph
        .append_rule_state_by_name(&String::from("q"), t3.clone(), &String::from("p"))
        .unwrap(); // q -> p

    expect_unk_name_error(graph.remove_state_with_name(&String::from("o")));
    // remove 't'
    graph.remove_state_with_name(&String::from("t")).unwrap();

    // check that it was removed
    expect_unk_name_error(graph.get_state_from_name(&String::from("t")));

    if graph
        .get_name_index_hashmap()
        .get(&String::from("t"))
        .is_some()
    {
        panic!("No index should have been returned")
    }

    // Check all the related transitions to 't' are also gone
    assert!(
        graph
            .get_state_from_name(&String::from("r"))
            .unwrap()
            .get_valid_transitions(&vec!('ç', 'ç'))
            .is_empty()
    );
    assert!(
        graph
            .get_state_from_name(&String::from("a"))
            .unwrap()
            .get_valid_transitions(&vec!('ç', 'ç'))
            .is_empty()
    );

    assert_eq!(
        graph
            .get_state_from_name(&String::from("q"))
            .unwrap()
            .get_valid_transitions(&vec!('ç', 'ç'))
            .len(),
        1
    );
    assert_eq!(
        *graph
            .get_state_from_name(&String::from("q"))
            .unwrap()
            .get_valid_transitions(&vec!('ç', 'ç'))
            .first()
            .unwrap(),
        &t3
    ); // only q -> p, should be left

    // Check that the indexes of 'q' and 'p' are also changed
    assert_eq!(graph.add_state(&String::from("p")), ind_p - 1);
    assert_eq!(graph.add_state(&String::from("q")), ind_q - 1);

    let ind_p = ind_p - 1;
    let ind_q = ind_q - 1;

    // Important to also make sure that the transition also changed

    assert_eq!(
        graph.get_transitions_by_index(ind_q, ind_p).unwrap(),
        vec!(&t3)
    );
}

// TODO : add test for removing states with indexes

fn expect_illegal_action_error<O>(res: Result<O, TuringError>) {
    if let Err(e) = res {
        match e {
            TuringError::IllegalActionError { cause: _ } => (),
            _ => panic!("Wrong error was returned"),
        }
    } else {
        panic!("Should have thrown an error")
    }
}

fn expect_unk_name_error<O>(res: Result<O, TuringError>) {
    if let Err(e) = res {
        match e {
            TuringError::UnknownStateError { state_name: _ } => (),
            _ => panic!("Wrong error was returned"),
        }
    } else {
        panic!("Should have thrown an error")
    }
}
