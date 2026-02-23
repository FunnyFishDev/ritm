use ritm_core::{
    EmptyTransition, SimpleTuringGraph,
    turing_graph::{TuringGraphError, TuringStateType},
    turing_index::TuringStateIndex,
    turing_transition::{
        TransitionMultRibbonInfo, TransitionOneRibbonInfo, TransitionsInfo, TuringDirection,
        TuringTransitionWrapper,
    },
};

#[test]
fn create_graph_test() {
    let graph = SimpleTuringGraph::new(2, true).unwrap();

    assert_eq!(graph.try_get_state("i").expect("present").get_id(), 0);
    assert_eq!(graph.try_get_state("a").expect("present").get_id(), 1);

    // Check the final states
    assert_eq!(
        TuringStateType::Accepting,
        graph.try_get_state("a").unwrap().get_type()
    );
    assert_eq!(
        TuringStateType::Normal,
        graph.try_get_state("i").unwrap().get_type()
    );

    SimpleTuringGraph::new(1, true).unwrap();
}

#[test]
fn create_graph_no_accepting_test() {
    let graph = SimpleTuringGraph::new(2, false).unwrap();

    assert_eq!(
        graph
            .try_get_state("i")
            .expect("present")
            .get_info()
            .get_id(),
        0
    );
    assert!(matches!(
        graph.try_get_state("a"),
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
    assert!(matches!(
        graph.try_add_state("i", TuringStateType::Normal),
        Err(TuringGraphError::AlreadyPresentNameError { name: _, state: _ })
    ));
    assert!(matches!(
        graph.try_add_state("a", TuringStateType::Normal),
        Err(TuringGraphError::AlreadyPresentNameError { name: _, state: _ })
    ));

    // Add new ones
    assert_eq!(
        graph
            .try_add_state("b", TuringStateType::Normal)
            .expect("no issues"),
        2
    );
    assert_eq!(
        graph
            .try_add_state("c", TuringStateType::Normal)
            .expect("no issues"),
        3
    );
    assert_eq!(
        graph
            .try_add_state("d", TuringStateType::Accepting)
            .expect("no issues"),
        4
    );
    // // Check they got added
    assert!(matches!(
        graph.try_add_state("b", TuringStateType::Normal),
        Err(TuringGraphError::AlreadyPresentNameError { name: _, state: _ })
    ));
    assert!(matches!(
        graph.try_add_state("c", TuringStateType::Normal),
        Err(TuringGraphError::AlreadyPresentNameError { name: _, state: _ })
    ));
    assert!(matches!(
        graph.try_add_state("d", TuringStateType::Normal),
        Err(TuringGraphError::AlreadyPresentNameError { name: _, state: _ })
    ));

    assert_eq!(graph.get_state("b").expect("no issues").get_id(), 2);
    assert_eq!(graph.get_state("c").expect("no issues").get_id(), 3);
    assert_eq!(graph.get_state("d").expect("no issues").get_id(), 4);
}

#[test]
fn get_nodes_test() {
    let mut graph = SimpleTuringGraph::new(1, true).unwrap();
    // Add new nodes
    graph
        .try_add_state("b", TuringStateType::Normal)
        .expect("no problem");
    graph
        .try_add_state("c", TuringStateType::Normal)
        .expect("no problem");
    graph
        .try_add_state("d", TuringStateType::Accepting)
        .expect("no problem");

    // check they get be obtained using an id
    assert_eq!(graph.try_get_state(2).unwrap().get_name().clone(), "b");
    assert_eq!(graph.try_get_state(3).unwrap().get_name().clone(), "c");
    assert_eq!(graph.try_get_state(4).unwrap().get_name().clone(), "d");

    // or their name
    assert_eq!(graph.try_get_state("b").unwrap().get_name().clone(), "b");
    assert_eq!(graph.try_get_state("c").unwrap().get_name().clone(), "c");
    assert_eq!(graph.try_get_state("d").unwrap().get_name().clone(), "d");

    // Check they aren't final
    assert_eq!(
        TuringStateType::Normal,
        graph.try_get_state("b").unwrap().get_type().clone()
    );
    assert_eq!(
        TuringStateType::Normal,
        graph.try_get_state("c").unwrap().get_type().clone()
    );
    assert_eq!(
        TuringStateType::Accepting,
        graph.try_get_state("d").unwrap().get_type().clone()
    );
}

#[test]
fn add_transitions_one() {
    let mut graph = SimpleTuringGraph::new(0, true).unwrap();

    graph
        .append_transition(
            "i",
            TransitionOneRibbonInfo::new('ç', TuringDirection::Right, 'ç'),
            "a",
        )
        .expect("no problem");

    let res = graph.append_transition("i", TransitionMultRibbonInfo::create_default(1), "a");
    assert!(matches!(
        res,
        Err(TuringGraphError::IncompatibleTransitionError { expected, received })
        if expected == 1 && received == 2
    ))
}

#[test]
fn add_transitions_k() {
    let mut graph = SimpleTuringGraph::new(1, true).unwrap();
    let transition = TransitionMultRibbonInfo::create(
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
            TuringStateIndex::ID(_) => panic!("this is not an id"),
            TuringStateIndex::Value(val) => val,
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
            TuringStateIndex::ID(_) => panic!("this is not an id"),
            TuringStateIndex::Value(val) => val,
                    } == "o"
    ));

    // add e and o to the graph
    graph
        .try_add_state("e", TuringStateType::Normal)
        .expect("no errors");
    graph
        .try_add_state("o", TuringStateType::Normal)
        .expect("no errors");

    // Check that the transition didn't already exists
    assert!(
        graph
            .get_transitions("e", "o")
            .expect("value expected here")
            .is_none()
    );

    let transition: TransitionsInfo = transition.into();

    // add transition
    graph
        .append_transition("e", transition.clone(), "o")
        .expect("no errors were expected");

    assert!(
        graph
            .get_transitions("e", "o")
            .expect("value expected here")
            .is_some_and(|val| { val.len() == 1 && val[0].info == transition })
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
fn add_default_transitions() {
    let mut graph = SimpleTuringGraph::new(3, true).unwrap();

    graph
        .append_default_transition(0, None, 1)
        .expect("no erros");
    assert_eq!(
        graph
            .get_transitions(0, 1)
            .expect("no errors")
            .expect("present")[0]
            .info,
        TransitionMultRibbonInfo::create_default(3).into()
    );

    // Try to add a default transition already added
    graph
        .append_transition(1, TransitionMultRibbonInfo::create_default(3), 0)
        .expect("no errors");

    assert!(matches!(
        graph.append_default_transition(1, None, 0),
        Err(TuringGraphError::AlreadyPresentTransitionError {
            from: _,
            to: _,
            transition: _
        })
    ));

    let mut graph = SimpleTuringGraph::new(0, true).unwrap();

    graph
        .append_default_transition(0, None, 1)
        .expect("no erros");
    assert_eq!(
        graph
            .get_transitions(0, 1)
            .expect("no errors")
            .expect("present")[0]
            .info,
        TransitionOneRibbonInfo {
            chars_read: 'ç',
            move_pointer: TuringDirection::None,
            replace_with: 'ç'
        }
        .into()
    );

    // Try to add a default transition already added
    graph
        .append_transition(1, TransitionOneRibbonInfo::default(), 0)
        .expect("no errors");

    assert!(matches!(
        graph.append_default_transition(1, None, 0),
        Err(TuringGraphError::AlreadyPresentTransitionError {
            from: _,
            to: _,
            transition: _
        })
    ));
}

#[test]
fn delete_transitions() {
    let mut graph = SimpleTuringGraph::new(1, true).unwrap();
    let t1 = TransitionMultRibbonInfo::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();

    let t2 = TransitionMultRibbonInfo::create(
        vec!['ç', '_'],
        vec!['_'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();

    graph.append_transition("i", t1.clone(), "a").unwrap();
    graph.append_transition("i", t2.clone(), "a").unwrap();

    assert!(matches!(
        graph.remove_transition(("i", t1.clone(), "d")),
        Err(TuringGraphError::UnknownStateIndex {
            accessed_index
         } ) if accessed_index == TuringStateIndex::from("d")
    ));

    assert!(matches!(
        graph.remove_transition(("d", t1.clone(), "a")),
        Err(TuringGraphError::UnknownStateIndex { accessed_index } ) if accessed_index == TuringStateIndex::from("d")
    ));

    // Remove transition
    graph.remove_transition(("i", t1.clone(), "a")).unwrap();

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
        t2.into()
    );
}

#[test]
fn delete_transitions_with_indexes() {
    let mut graph = SimpleTuringGraph::new(1, true).unwrap();
    let t1 = TransitionMultRibbonInfo::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();

    let t2 = TransitionMultRibbonInfo::create(
        vec!['ç', '_'],
        vec!['_'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();

    graph.append_transition("i", t1.clone(), "a").unwrap();
    graph.append_transition("i", t2.clone(), "a").unwrap();

    // Remove transition
    graph.remove_transition(("i", 1, "a")).unwrap();

    // Check that it was indeed removed
    assert!(
        graph
            .get_valid_transitions("i", vec!('ç', '_'))
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
        t1.into()
    );
}

#[test]
fn delete_all_transitions_two_nodes() {
    let mut graph = SimpleTuringGraph::new(1, true).unwrap();
    let t1 = TransitionMultRibbonInfo::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();
    let t2 = TransitionMultRibbonInfo::create(
        vec!['ç', '_'],
        vec!['_'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();
    let t3 = TransitionMultRibbonInfo::create(
        vec!['_', '_'],
        vec!['_'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();

    graph.append_transition("i", t1.clone(), "a").unwrap();
    graph.append_transition("i", t2.clone(), "a").unwrap();
    graph.append_transition("i", t3.clone(), "i").unwrap(); // i -> i

    // Removes all transitions btw 'i' and 'a'
    graph.remove_transitions("i", "a").unwrap();

    // (note: index of 'i' is 0)
    assert!(graph.get_transitions(0, 1).expect("no errors").is_none());

    // check that i -> i, is still here
    assert_eq!(
        graph
            .get_transitions(0, 0)
            .expect("no error")
            .expect("present")[0]
            .info,
        t3.into()
    );
}

#[test]
fn delete_node() {
    let mut graph = SimpleTuringGraph::new(1, true).expect("no errors");
    let t1 = TransitionMultRibbonInfo::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .expect("no errors");
    let t2 = TransitionMultRibbonInfo::create(
        vec!['ç', '_'],
        vec!['_'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .expect("no errors");
    let t3 = TransitionMultRibbonInfo::create(
        vec!['_', '_'],
        vec!['_'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .expect("no errors");

    graph
        .try_add_state("q", TuringStateType::Normal)
        .expect("no problem");

    graph
        .append_transition("i", t1.clone(), "q")
        .expect("no errors"); // i -> q
    graph
        .append_transition("q", t2.clone(), "i")
        .expect("no errors"); // q -> i
    graph
        .append_transition("q", t2.clone(), "q")
        .expect("no errors"); // q -> q
    graph
        .append_transition("i", t3.clone(), "i")
        .expect("no errors"); // i -> i

    let q_index = graph.get_state("q").expect("present").get_id();

    // Removes state q
    graph.remove_state("q").expect("no errors");

    // Check that "q" is gone
    assert!(graph.get_state("q").is_none());
    assert!(graph.get_state(q_index).is_none());

    // Check that no transition with q is present anymore

    assert!({
        let mut not_present = true;
        for (from, to) in graph.get_transitions_hashmap().keys() {
            if *from == q_index || *to == q_index {
                not_present = false;
                break;
            }
        }
        not_present
    });
}

#[test]
fn rename_state() {
    // i and a are present
    let mut graph = SimpleTuringGraph::new(2, true).unwrap();

    // rename i as start :
    graph.rename_state("i", "start").expect("no problem");
    assert_eq!(graph.get_state("start").expect("present").get_id(), 0);
    // State not present
    assert!(matches!(
        graph.rename_state("q", "i"),
        Err(TuringGraphError::UnknownStateIndex { accessed_index: _ })
    ));
    // Name already present
    assert!(matches!(
        graph.rename_state("start", "a"),
        Err(TuringGraphError::AlreadyPresentNameError { name: _, state: _ })
    ));
}

#[test]
fn get_valid_transitions() {
    let mut graph = SimpleTuringGraph::new(1, true).unwrap();

    let t1 = TransitionMultRibbonInfo::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();
    let t2 = TransitionMultRibbonInfo::create(
        vec!['ç', '_'],
        vec!['0'],
        vec![TuringDirection::None, TuringDirection::Left],
    )
    .unwrap();
    let t3 = TransitionMultRibbonInfo::create(
        vec!['ç', '_'],
        vec!['0'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();

    // there should be no values
    assert!(
        graph
            .get_valid_transitions(0, vec!('ç'))
            .expect("no error")
            .is_empty()
    );

    // add transitions
    graph
        .append_transition(0, t1.clone(), 1)
        .expect("no problem");
    graph
        .append_transition(0, t2.clone(), 1)
        .expect("no problem");
    graph
        .append_transition(0, t3.clone(), 1)
        .expect("no problem");

    assert_eq!(
        graph
            .get_valid_transitions(0, vec!('ç', 'ç'))
            .expect("no problem"),
        vec!((
            &TuringTransitionWrapper {
                info: t1.into(),
                inner_transition: EmptyTransition
            },
            1
        ))
    );
    assert_eq!(
        graph
            .get_valid_transitions(0, vec!('ç', '_'))
            .expect("no problem"),
        vec!(
            (
                &TuringTransitionWrapper {
                    info: t2.into(),
                    inner_transition: EmptyTransition
                },
                1
            ),
            (
                &TuringTransitionWrapper {
                    info: t3.into(),
                    inner_transition: EmptyTransition
                },
                1
            )
        )
    );
}

#[test]
fn is_deterministic_test_mult_rib() {
    // Deterministic graph :
    let mut deter_g = SimpleTuringGraph::new(1, false).expect("correct");
    deter_g.add_state("1", TuringStateType::Normal);

    deter_g
        .append_default_transition(0, None, 0)
        .expect("correct");

    assert!(deter_g.is_deterministic());

    deter_g
        .append_transition(
            0,
            TransitionMultRibbonInfo::new(
                vec!['ç', 'ç'],
                TuringDirection::Right,
                vec![('ç', TuringDirection::Right)],
            )
            .expect("correct"),
            1,
        )
        .expect("correct");

    assert!(!deter_g.is_deterministic());
    // Remove one of the problematic transitions
    deter_g.remove_transition((0, 0, 1)).expect("ok");

    assert!(deter_g.is_deterministic());
}

#[test]
fn is_deterministic_test_one_rib() {
    // Deterministic graph :
    let mut deter_g = SimpleTuringGraph::new(0, false).expect("correct");
    deter_g.add_state("1", TuringStateType::Normal);

    deter_g
        .append_default_transition(0, None, 0)
        .expect("correct");

    assert!(deter_g.is_deterministic());

    deter_g
        .append_transition(
            0,
            TransitionOneRibbonInfo::new('ç', TuringDirection::Right, 'ç'),
            "1",
        )
        .expect("correct");

    assert!(!deter_g.is_deterministic());
    // Remove one of the problematic transitions
    deter_g.remove_transition((0, 0, 1)).expect("ok");

    assert!(deter_g.is_deterministic());
}
