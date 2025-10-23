use ritm_core::{turing_errors::TuringError, turing_state::*};

// ________________________________________ Transitions tests ______________________________
#[test]
fn transition_creation_test() {
    let t1 = TuringTransition::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::Right, TuringDirection::None],
    )
    .unwrap();

    expect_wrong_args_error(TuringTransition::create(
        vec!['ç', 'ç'],
        vec![],
        vec![TuringDirection::Right],
    ));

    expect_wrong_args_error(TuringTransition::create(
        vec![],
        vec!['ç'],
        vec![TuringDirection::Right],
    ));

    expect_wrong_args_error(TuringTransition::create(vec!['ç'], vec!['ç'], vec![]));

    if t1.index_to_state.is_some() {
        panic!("A none value was expected here");
    }

    assert_eq!(t1.chars_read, vec!('ç', 'ç'));
    assert_eq!(t1.move_read, TuringDirection::Right);
    assert_eq!(t1.chars_write, vec!(('ç', TuringDirection::None)));

    assert_eq!(t1.get_number_of_affected_tapes(), 2)
}
#[test]
fn create_ill_transitions() {
    match TuringTransition::create(
        vec!['$', '_'],
        vec!['_'],
        vec![TuringDirection::Right, TuringDirection::None],
    ) {
        Ok(_) => panic!("Expected an error"),
        Err(te) => expect_ill_action_error(te),
    }

    match TuringTransition::create(
        vec!['ç', '_'],
        vec!['_'],
        vec![TuringDirection::Left, TuringDirection::None],
    ) {
        Ok(_) => panic!("Expected an error"),
        Err(te) => expect_ill_action_error(te),
    }

    match TuringTransition::create(
        vec!['_', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Left],
    ) {
        Ok(_) => panic!("Expected an error"),
        Err(te) => expect_ill_action_error(te),
    }

    match TuringTransition::create(
        vec!['_', '_'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Left],
    ) {
        Ok(_) => panic!("Expected an error"),
        Err(te) => expect_ill_action_error(te),
    }

    match TuringTransition::create(
        vec!['_', 'ç'],
        vec!['_'],
        vec![TuringDirection::None, TuringDirection::Left],
    ) {
        Ok(_) => panic!("Expected an error"),
        Err(te) => expect_ill_action_error(te),
    }
}

#[test]
fn transition_eq() {
    let mut t1 = TuringTransition::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();

    assert_ne!(
        t1,
        TuringTransition::create(
            vec!('ç', 'ç'),
            vec!('ç'),
            vec!(TuringDirection::Right, TuringDirection::Right)
        )
        .unwrap()
    );

    assert_ne!(
        t1,
        TuringTransition::create(
            vec!('ç', 'v'),
            vec!('_'),
            vec!(TuringDirection::Right, TuringDirection::Right)
        )
        .unwrap()
    );

    assert_ne!(
        t1,
        TuringTransition::create(
            vec!('ç', 'v'),
            vec!('t'),
            vec!(TuringDirection::Right, TuringDirection::Right)
        )
        .unwrap()
    );

    assert_ne!(
        t1,
        TuringTransition::create(
            vec!('ç', 'v', 'p'),
            vec!('t', 'x'),
            vec!(
                TuringDirection::Right,
                TuringDirection::Right,
                TuringDirection::Left
            )
        )
        .unwrap()
    );

    // The pointed index should not be part of the comparison
    t1.index_to_state = Some(1);

    assert_eq!(
        t1,
        TuringTransition::create(
            vec!('ç', 'ç'),
            vec!('ç'),
            vec!(TuringDirection::None, TuringDirection::Right)
        )
        .unwrap()
    );
}

// ________________________________________ States tests ______________________________

#[test]
fn create_state() {
    let s = TuringState::new(TuringStateType::Normal, String::from("test"));
    // name should be test
    assert_eq!(s.name, String::from("test"));

    // It should be false
    assert_eq!(TuringStateType::Normal, s.state_type);

    let s = TuringState::new(TuringStateType::Accepting, String::from("test2"));

    // It should be true
    assert_eq!(TuringStateType::Accepting, s.state_type);

    // There should be no transitions
    assert!(s.transitions.is_empty());
}

#[test]
fn rename_state() {
    let mut s = TuringState::new(TuringStateType::Normal, String::from("test"));
    s.rename("test2");
    // name should be test2
    assert_eq!(s.name, String::from("test2"));
}

#[test]
fn add_transitions() {
    let mut s = TuringState::new(TuringStateType::Normal, String::from("test"));
    let transition = TuringTransition::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::Right, TuringDirection::Right],
    )
    .unwrap();
    s.add_transition(transition)
        .expect("There shouldn't be an error here");

    // Check that the transition was added
    assert_eq!(
        s.transitions.first().unwrap(),
        &TuringTransition::create(
            vec!('ç', 'ç'),
            vec!('ç'),
            vec!(TuringDirection::Right, TuringDirection::Right)
        )
        .unwrap()
    );

    // Check that we cannot add a transition that has a different size
    let transition2 = TuringTransition::create(
        vec!['ç', 'ç', 'ç'],
        vec!['ç', 'ç'],
        vec![
            TuringDirection::Right,
            TuringDirection::Right,
            TuringDirection::None,
        ],
    )
    .unwrap();
    expect_incompatible_transition_error(s.add_transition(transition2));
}

#[test]
fn remove_transitions_using_index() {
    let mut s = TuringState::new(TuringStateType::Normal, String::from("test"));
    // add transitions
    s.add_transition(
        TuringTransition::create(
            vec!['ç', 'ç'],
            vec!['ç'],
            vec![TuringDirection::Right, TuringDirection::Right],
        )
        .unwrap(),
    )
    .expect("There shouldn't be an error");
    s.add_transition(
        TuringTransition::create(
            vec!['ç', '_'],
            vec!['0'],
            vec![TuringDirection::Right, TuringDirection::Left],
        )
        .unwrap(),
    )
    .expect("There shouldn't be an error");

    // Remove both of them
    s.remove_transition_with_index(0).unwrap();
    s.remove_transition_with_index(0).unwrap();

    assert!(s.transitions.is_empty());

    // Add them back
    s.add_transition(
        TuringTransition::create(
            vec!['ç', 'ç'],
            vec!['ç'],
            vec![TuringDirection::Right, TuringDirection::Right],
        )
        .unwrap(),
    )
    .expect("There shouldn't be an error");
    s.add_transition(
        TuringTransition::create(
            vec!['ç', '_'],
            vec!['0'],
            vec![TuringDirection::Right, TuringDirection::Left],
        )
        .unwrap(),
    )
    .expect("There shouldn't be an error");

    expect_out_of_range_transition_error(s.remove_transition_with_index(2));
}

#[test]
fn remove_transitions_using_ref() {
    let mut s = TuringState::new(TuringStateType::Normal, String::from("test"));
    let t1 = TuringTransition::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::Right, TuringDirection::Right],
    )
    .unwrap();
    let t2 = TuringTransition::create(
        vec!['ç', '_'],
        vec!['0'],
        vec![TuringDirection::Right, TuringDirection::Left],
    )
    .unwrap();

    // add transitions
    s.add_transition(t1.clone())
        .expect("There shouldn't be an error");
    s.add_transition(t2.clone())
        .expect("There shouldn't be an error");

    // Remove the first one
    s.remove_transition(&t1);
    // check that it was removed, but not the other one
    assert!(s.get_valid_transitions(&vec!('ç', 'ç')).is_empty());
    assert!(!s.get_valid_transitions(&vec!('ç', '_')).is_empty());

    // Nothing should happen
    s.remove_transition(&t1);

    // Remove the second one
    s.remove_transition(&t2);
    assert!(s.get_valid_transitions(&vec!('ç', '_')).is_empty());
}

#[test]
fn get_valid_transitions() {
    let mut s = TuringState::new(TuringStateType::Normal, String::from("test"));
    let t1 = TuringTransition::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();
    let t2 = TuringTransition::create(
        vec!['ç', '_'],
        vec!['0'],
        vec![TuringDirection::None, TuringDirection::Left],
    )
    .unwrap();
    let t3 = TuringTransition::create(
        vec!['ç', '_'],
        vec!['0'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();
    // there should be no values
    assert!(s.get_valid_transitions(&vec!('ç')).is_empty());

    // add transitions
    s.add_transition(t1.clone())
        .expect("There shouldn't be an error");
    s.add_transition(t2.clone())
        .expect("There shouldn't be an error");
    s.add_transition(t3.clone())
        .expect("There shouldn't be an error");

    assert_eq!(s.get_valid_transitions(&vec!('ç', 'ç')), vec!(&t1));
    assert_eq!(s.get_valid_transitions(&vec!('ç', '_')), vec!(&t2, &t3));
}

#[test]
fn update_transitions() {
    let mut s = TuringState::new(TuringStateType::Normal, String::from("test"));
    let t1 = TuringTransition::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::Right, TuringDirection::Right],
    )
    .unwrap();
    let t2 = TuringTransition::create(
        vec!['ç', '_'],
        vec!['c'],
        vec![TuringDirection::None, TuringDirection::None],
    )
    .unwrap();
    let t3 = TuringTransition::create(
        vec!['_', '_'],
        vec!['b'],
        vec![TuringDirection::Right, TuringDirection::Left],
    )
    .unwrap();

    s.add_transition(t1.clone())
        .expect("There shouldn't be an error here");
    s.add_transition(t2.clone())
        .expect("There shouldn't be an error here");
    s.add_transition(t3.clone())
        .expect("There shouldn't be an error here");

    s.transitions[0].index_to_state = Some(2); // t1 points to 2
    s.transitions[1].index_to_state = Some(1); // t2 points to 1
    s.transitions[2].index_to_state = Some(2); // t3 also points to 2

    // check that the transition are indeed going to the right index
    assert_eq!(s.get_transitions_to(1), vec!(&t2));
    assert_eq!(s.get_transitions_to(2), vec!(&t1, &t3));

    // Change all that were pointing to 2 to 3
    s.update_transitions(2, 3);

    // Check that the change indeed took place :
    assert_eq!(s.get_transitions_to(1), vec!(&t2));
    assert_eq!(s.get_transitions_to(3), vec!(&t1, &t3));

    s.update_transitions(1, 3);
    assert_eq!(s.get_transitions_to(3), vec!(&t1, &t2, &t3));
}

fn expect_ill_action_error(te: TuringError) {
    match te {
        TuringError::IllegalActionError { cause: _ } => {}
        _ => panic!(
            "Exepected an IllegalActionError, but received the following error : {:?}",
            te
        ),
    }
}

fn expect_wrong_args_error<O>(res: Result<O, TuringError>) {
    if let Err(e) = res {
        match e {
            TuringError::TransitionArgsError { reason: _ } => (),

            _ => panic!("Wrong error was returned"),
        }
    } else {
        panic!("Should have thrown an error")
    }
}

fn expect_incompatible_transition_error<O>(res: Result<O, TuringError>) {
    if let Err(e) = res {
        match e {
            TuringError::IncompatibleTransitionError {
                expected: _,
                received: _,
            } => (),

            _ => panic!("Wrong error was returned"),
        }
    } else {
        panic!("Should have thrown an error")
    }
}

fn expect_out_of_range_transition_error<O>(res: Result<O, TuringError>) {
    if let Err(e) = res {
        match e {
            TuringError::OutOfRangeTransitionError {
                accessed_index: _,
                states_len: _,
            } => (),
            _ => panic!("Wrong error was returned"),
        }
    } else {
        panic!("Should have thrown an error")
    }
}
