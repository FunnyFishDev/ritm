use ritm_core::{
    turing_graph::{TuringGraphError, TuringMachineGraph},
    turing_state::TuringStateType,
    turing_transition::{TuringDirection, TuringTransitionWrapper},
};

#[test]
fn create_graph_test() {
    let graph = TuringMachineGraph::new(2).unwrap();

    assert_eq!(*graph.get_name_index_hashmap().get("i").unwrap(), 0);
    assert_eq!(*graph.get_name_index_hashmap().get("a").unwrap(), 1);

    assert!(matches!(
        TuringMachineGraph::new(0),
        Err(TuringGraphError::NotEnoughTapesError)
    ));
    // Check the final states
    assert_eq!(
        TuringStateType::Accepting,
        graph.get_state_from_name("a").unwrap().state_type
    );
    assert_eq!(
        TuringStateType::Normal,
        graph.get_state_from_name("i").unwrap().state_type
    );

    TuringMachineGraph::new(1).unwrap();
}

#[test]
fn delete_init_nodes_test() {
    let mut graph = TuringMachineGraph::new(2).unwrap();

    assert!(matches!(
        graph.remove_state_with_name("i"),
        Err(TuringGraphError::ImmutableStateError { state: _ })
    ));

    assert!(matches!(
        graph.remove_state_with_name("a"),
        Err(TuringGraphError::ImmutableStateError { state: _ })
    ));
}

#[test]
fn add_nodes() {
    let mut graph = TuringMachineGraph::new(1).unwrap();

    // Check they already exists
    assert_eq!(graph.add_state("i"), 0);
    assert_eq!(graph.add_state("a"), 1);

    // Add new ones
    assert_eq!(graph.add_state("b"), 2);
    assert_eq!(graph.add_state("c"), 3);
    assert_eq!(graph.add_state("d"), 4);
    // Check they got the correct index
    assert_eq!(graph.add_state("b"), 2);
    assert_eq!(graph.add_state("c"), 3);
    assert_eq!(graph.add_state("d"), 4);
}

#[test]
fn get_nodes_test() {
    let mut graph = TuringMachineGraph::new(1).unwrap();
    // Add new nodes
    assert_eq!(graph.add_state("b"), 2);
    assert_eq!(graph.add_state("c"), 3);
    assert_eq!(graph.add_state("d"), 4);

    // check they get be obtained
    assert_eq!(graph.get_state(2).unwrap().name.clone(), "b");
    assert_eq!(graph.get_state(3).unwrap().name.clone(), "c");
    assert_eq!(graph.get_state(4).unwrap().name.clone(), "d");

    // check they get be obtained
    assert_eq!(graph.get_state_from_name("b").unwrap().name.clone(), "b");
    assert_eq!(graph.get_state_from_name("c").unwrap().name.clone(), "c");
    assert_eq!(graph.get_state_from_name("d").unwrap().name.clone(), "d");

    // Check they aren't final
    assert_eq!(
        TuringStateType::Normal,
        graph.get_state_from_name("b").unwrap().state_type
    );
    assert_eq!(
        TuringStateType::Normal,
        graph.get_state_from_name("c").unwrap().state_type
    );
    assert_eq!(
        TuringStateType::Normal,
        graph.get_state_from_name("d").unwrap().state_type
    );
}

#[test]
fn add_transition() {
    let mut graph = TuringMachineGraph::new(1).unwrap();

    graph
        .append_rule_state_by_name(
            "i",
            TuringTransitionWrapper::create(
                vec!['ç', 'ç'],
                vec!['ç'],
                vec![TuringDirection::None, TuringDirection::Right],
            )
            .unwrap(),
            "a",
        )
        .expect("no errors were expected");

    // e, is not part of the graph
    assert!(matches!(
        graph.append_rule_state_by_name(
            "e",
            TuringTransitionWrapper::create(
                vec!['ç', 'ç'],
                vec!['ç'],
                vec![TuringDirection::None, TuringDirection::Right],
            )
            .unwrap(),
            "a",
        ),
        Err(TuringGraphError::UnknownStateNameError { state_name } ) if state_name == "e"
    ));

    assert!(matches!(
        graph.append_rule_state_by_name(
            "a",
            TuringTransitionWrapper::create(
                vec!['ç', 'ç'],
                vec!['ç'],
                vec![TuringDirection::None, TuringDirection::Right],
            )
            .unwrap(),
            "o",
        ),
        Err(TuringGraphError::UnknownStateNameError { state_name } ) if state_name == "o"
    ));

    // add e and o to the graph
    graph.add_state("e");
    graph.add_state("o");

    // Check that the transition didn't already exists
    // Check that the transition was really added
    if !graph
        .get_transition_indexes_by_name("e", "o")
        .expect("a value was expected here")
        .is_empty()
    {
        panic!("No values were expected");
    }
    // add transition
    graph
        .append_rule_state_by_name(
            "e",
            TuringTransitionWrapper::create(
                vec!['ç', 'ç'],
                vec!['ç'],
                vec![TuringDirection::None, TuringDirection::Right],
            )
            .unwrap(),
            "o",
        )
        .expect("no errors were expected");

    // Check that the transition was really added
    if graph
        .get_transition_indexes_by_name("e", "o")
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
    let t1 = TuringTransitionWrapper::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();
    let t2 = TuringTransitionWrapper::create(
        vec!['ç', '_'],
        vec!['_'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();

    graph
        .append_rule_state_by_name("i", t1.clone(), "a")
        .unwrap();
    graph
        .append_rule_state_by_name("i", t2.clone(), "a")
        .unwrap();

    assert!(matches!(
        graph.remove_transition("i", &t1, "d"),
        Err(TuringGraphError::UnknownStateNameError { state_name } ) if state_name == "d"
    ));

    assert!(matches!(
        graph.remove_transition("d", &t1, "a"),
        Err(TuringGraphError::UnknownStateNameError { state_name } ) if state_name == "d"
    ));

    // Remove transition
    graph.remove_transition("i", &t1, "a").unwrap();

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
    let t1 = TuringTransitionWrapper::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();
    let t2 = TuringTransitionWrapper::create(
        vec!['ç', '_'],
        vec!['_'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();
    let t3 = TuringTransitionWrapper::create(
        vec!['_', '_'],
        vec!['_'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();

    graph
        .append_rule_state_by_name("i", t1.clone(), "a")
        .unwrap();
    graph
        .append_rule_state_by_name("i", t2.clone(), "a")
        .unwrap();
    graph
        .append_rule_state_by_name("i", t3.clone(), "i")
        .unwrap(); // i -> i

    // Removes all transitions btw 'i' and 'a'
    graph.remove_transitions("i", "a").unwrap();

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
    let t1 = TuringTransitionWrapper::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();
    let t2 = TuringTransitionWrapper::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();
    let t3 = TuringTransitionWrapper::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();

    let _ = graph.add_state("t");
    let ind_p = graph.add_state("p");
    let ind_q = graph.add_state("q");

    graph
        .append_rule_state_by_name("t", t1.clone(), "a")
        .unwrap(); // t -> a
    graph
        .append_rule_state_by_name("a", t2.clone(), "t")
        .unwrap(); // a -> t
    graph
        .append_rule_state_by_name("p", t2.clone(), "t")
        .unwrap(); // a -> t
    graph
        .append_rule_state_by_name("q", t3.clone(), "t")
        .unwrap(); // q -> t
    graph
        .append_rule_state_by_name("q", t3.clone(), "p")
        .unwrap(); // q -> p

    assert!(matches!(
        graph.remove_state_with_name("o"),
        Err(TuringGraphError::UnknownStateNameError { state_name } ) if state_name == "o"
    ));

    // remove 't'
    graph.remove_state_with_name("t").unwrap();

    // check that it was removed
    assert!(matches!(
        graph.remove_state_with_name("t"),
        Err(TuringGraphError::UnknownStateNameError { state_name } ) if state_name == "t"
    ));

    if graph.get_name_index_hashmap().get("t").is_some() {
        panic!("No index should have been returned")
    }

    // Check all the related transitions to 't' are also gone
    assert!(
        graph
            .get_state_from_name("p")
            .unwrap()
            .get_valid_transitions(&vec!('ç', 'ç'))
            .is_empty()
    );
    assert!(
        graph
            .get_state_from_name("a")
            .unwrap()
            .get_valid_transitions(&vec!('ç', 'ç'))
            .is_empty()
    );

    assert_eq!(
        graph
            .get_state_from_name("q")
            .unwrap()
            .get_valid_transitions(&vec!('ç', 'ç'))
            .len(),
        1
    );
    assert_eq!(
        *graph
            .get_state_from_name("q")
            .unwrap()
            .get_valid_transitions(&vec!('ç', 'ç'))
            .first()
            .unwrap(),
        &t3
    ); // only q -> p, should be left

    // Check that the indexes of 'q' and 'p' are also changed
    assert_eq!(graph.add_state("p"), ind_p - 1);
    assert_eq!(graph.add_state("q"), ind_q - 1);

    let ind_p = ind_p - 1;
    let ind_q = ind_q - 1;

    // Important to also make sure that the transition also changed

    assert_eq!(
        graph.get_transitions_by_index(ind_q, ind_p).unwrap(),
        vec!(&t3)
    );
}
