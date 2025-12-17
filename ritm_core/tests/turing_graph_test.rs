use ritm_core::{
    SimpleTuringGraph,
    turing_graph::{TuringGraphError, TuringGraphIndex, TuringMachineGraph, TuringStateType},
    turing_transition::{
        TuringDirection, TuringTransition, TuringTransitionInfo, TuringTransitionWrapper,
    },
};

#[test]
fn create_graph_test() {
    let graph = SimpleTuringGraph::new(2, true).unwrap();

    assert_eq!(graph.get_state("i").expect("present").get_id(), 0);
    assert_eq!(graph.get_state("a").expect("present").get_id(), 1);

    assert!(matches!(
        SimpleTuringGraph::new(0, false),
        Err(TuringGraphError::NotEnoughTapesError)
    ));
    // Check the final states
    assert_eq!(
        TuringStateType::Accepting,
        *graph.get_state("a").unwrap().get_type()
    );
    assert_eq!(
        TuringStateType::Normal,
        *graph.get_state("i").unwrap().get_type()
    );

    SimpleTuringGraph::new(1, true).unwrap();
}

#[test]
fn create_graph_no_accepting_test() {
    let graph = SimpleTuringGraph::new(2, false).unwrap();

    assert_eq!(
        graph.get_state("i").expect("present").get_info().get_id(),
        0
    );
    assert!(matches!(
        graph.get_state("a"),
        Err(TuringGraphError::UnknownStateIndex { accessed_index: _ })
    ));
}

#[test]
fn delete_init_nodes_test() {
    let mut graph = SimpleTuringGraph::new(2, true).unwrap();

    assert!(matches!(
        graph.remove_state("i"),
        Err(TuringGraphError::ImmutableStateError { state: _ })
    ));

    graph.remove_state("a").expect("no errors");

    assert!(matches!(
        graph.remove_state("a"),
        Err(TuringGraphError::UnknownStateIndex { accessed_index: _ })
    ));
}

#[test]
fn add_nodes() {
    let mut graph = SimpleTuringGraph::new(1, true).unwrap();

    // Check they already exists
    assert_eq!(graph.add_state("i", TuringStateType::Normal), 0);
    assert_eq!(graph.add_state("a", TuringStateType::Normal), 1);

    // Add new ones
    assert_eq!(graph.add_state("b", TuringStateType::Normal), 2);
    assert_eq!(graph.add_state("c", TuringStateType::Normal), 3);
    assert_eq!(graph.add_state("d", TuringStateType::Accepting), 4);
    // Check they got the correct index
    assert_eq!(graph.add_state("b", TuringStateType::Accepting), 2);
    assert_eq!(graph.add_state("c", TuringStateType::Accepting), 3);
    assert_eq!(graph.add_state("d", TuringStateType::Accepting), 4);
}

#[test]
fn get_nodes_test() {
    let mut graph = SimpleTuringGraph::new(1, true).unwrap();
    // Add new nodes
    assert_eq!(graph.add_state("b", TuringStateType::Normal), 2);
    assert_eq!(graph.add_state("c", TuringStateType::Normal), 3);
    assert_eq!(graph.add_state("d", TuringStateType::Accepting), 4);

    // check they get be obtained using an id
    assert_eq!(graph.get_state(2).unwrap().get_name().clone(), "b");
    assert_eq!(graph.get_state(3).unwrap().get_name().clone(), "c");
    assert_eq!(graph.get_state(4).unwrap().get_name().clone(), "d");

    // or their name
    assert_eq!(graph.get_state("b").unwrap().get_name().clone(), "b");
    assert_eq!(graph.get_state("c").unwrap().get_name().clone(), "c");
    assert_eq!(graph.get_state("d").unwrap().get_name().clone(), "d");

    // Check they aren't final
    assert_eq!(
        TuringStateType::Normal,
        graph.get_state("b").unwrap().get_type().clone()
    );
    assert_eq!(
        TuringStateType::Normal,
        graph.get_state("c").unwrap().get_type().clone()
    );
    assert_eq!(
        TuringStateType::Accepting,
        graph.get_state("d").unwrap().get_type().clone()
    );
}

#[test]
fn add_transitions() {
    let mut graph = SimpleTuringGraph::new(1, true).unwrap();
    let transition = TuringTransitionWrapper::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();

    graph
        .append_transition(0, transition.clone(), "a")
        .expect("no errors were expected");

    // e, is not part of the graph
    assert!(matches!(
        graph.append_transition(
            "e",
            transition.clone(),
            1,
        ),
        Err(TuringGraphError::UnknownStateIndex { accessed_index } ) if match &accessed_index {
            ritm_core::turing_graph::TuringGraphIndex::ID(_) => panic!("this is not an id"),
            ritm_core::turing_graph::TuringGraphIndex::Name(val) => val,
                    } == "e"
    ));

    // "o" not part either
    assert!(matches!(
        graph.append_transition(
            "a",
            transition.clone(),
            "o",
        ),
        Err(TuringGraphError::UnknownStateIndex { accessed_index } ) if match &accessed_index {
            ritm_core::turing_graph::TuringGraphIndex::ID(_) => panic!("this is not an id"),
            ritm_core::turing_graph::TuringGraphIndex::Name(val) => val,
                    } == "o"
    ));

    // add e and o to the graph
    graph.add_state("e", TuringStateType::Normal);
    graph.add_state("o", TuringStateType::Normal);

    // Check that the transition didn't already exists
    assert!(
        graph
            .get_transitions("e", "o")
            .expect("value expected here")
            .is_none()
    );

    // add transition
    graph
        .append_transition("e", transition.clone(), "o")
        .expect("no errors were expected");

    assert!(
        graph
            .get_transitions("e", "o")
            .expect("value expected here")
            .is_some_and(|val| { val.len() == 1 && val[0].info == transition.info })
    );

    // Add the same transition again

    assert!(matches!(
        graph.append_transition("e", transition.clone(), "o"),
        Err(TuringGraphError::AlreadyPresentTransitionError {
            from: _,
            to: _,
            transition: _
        })
    ))
}

#[test]
fn delete_transitions() {
    let mut graph = SimpleTuringGraph::new(1, true).unwrap();
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

    graph.append_transition("i", t1.clone(), "a").unwrap();
    graph.append_transition("i", t2.clone(), "a").unwrap();

    assert!(matches!(
        graph.remove_transition("i", &t1, "d"),
        Err(TuringGraphError::UnknownStateIndex {
            accessed_index
         } ) if accessed_index == TuringGraphIndex::from("d")
    ));

    assert!(matches!(
        graph.remove_transition("d", &t1, "a"),
        Err(TuringGraphError::UnknownStateIndex { accessed_index } ) if accessed_index == TuringGraphIndex::from("d")
    ));

    // Remove transition
    graph.remove_transition("i", &t1, "a").unwrap();

    // Check that it was indeed removed
    assert!(
        graph
            .get_valid_transitions("i", vec!('ç', 'ç'))
            .expect("no errors")
            .is_empty()
    );

    // and that the other one is still present
    assert_eq!(
        graph
            .get_transitions(0, 1)
            .expect("no errors")
            .expect("present")
            .first()
            .expect("at least one element")
            .info,
        t2.info
    );
}

// #[test]
// fn delete_all_transitions_two_nodes() {
//     let mut graph = SimpleTuringGraph::new(1).unwrap();
//     let t1 = TuringTransition::create(
//         vec!['ç', 'ç'],
//         vec!['ç'],
//         vec![TuringDirection::None, TuringDirection::Right],
//     )
//     .unwrap();
//     let t2 = TuringTransition::create(
//         vec!['ç', '_'],
//         vec!['_'],
//         vec![TuringDirection::None, TuringDirection::Right],
//     )
//     .unwrap();
//     let t3 = TuringTransition::create(
//         vec!['_', '_'],
//         vec!['_'],
//         vec![TuringDirection::None, TuringDirection::Right],
//     )
//     .unwrap();

//     graph
//         .append_rule_state_by_name("i", t1.clone(), "a")
//         .unwrap();
//     graph
//         .append_rule_state_by_name("i", t2.clone(), "a")
//         .unwrap();
//     graph
//         .append_rule_state_by_name("i", t3.clone(), "i")
//         .unwrap(); // i -> i

//     // Removes all transitions btw 'i' and 'a'
//     graph.remove_transitions("i", "a").unwrap();

//     // (note: index of 'i' is 0)
//     assert!(graph.get_state(0).unwrap().get_transitions_to(1).is_empty());

//     // check that i -> i, is still here
//     assert_eq!(
//         *graph
//             .get_state(0)
//             .unwrap()
//             .get_transitions_to(0)
//             .first()

//     ); // only q -> p, should be left

//     // Check that the indexes of 'q' and 'p' are also changed
//     assert_eq!(graph.add_state("p"), ind_p - 1);
//     assert_eq!(graph.add_state("q"), ind_q - 1);

//     let ind_p = ind_p - 1;
//     let ind_q = ind_q - 1;

//     // Important to also make sure that the transition also changed

//     assert_eq!(
//         graph.get_transitions_by_index(ind_q, ind_p).unwrap(),
//         vec!(&t3)
//     );
// }
