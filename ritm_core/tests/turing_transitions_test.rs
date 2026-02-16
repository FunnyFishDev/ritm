use ritm_core::turing_transition::{TuringDirection, TuringTransitionError, TransitionMultRibbonInfo};

// ________________________________________ Transitions tests ______________________________
#[test]
fn transition_creation_test() {
    let t1 = TransitionMultRibbonInfo::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::Right, TuringDirection::None],
    )
    .unwrap();

    assert!(matches!(
        TransitionMultRibbonInfo::create(vec!['ç', 'ç'], vec![], vec![TuringDirection::Right],),
        Err(TuringTransitionError::TransitionArgsError(_val))
    ));

    assert!(matches!(
        TransitionMultRibbonInfo::create(vec![], vec!['ç'], vec![TuringDirection::Right],),
        Err(TuringTransitionError::TransitionArgsError(_val))
    ));

    assert!(matches!(
        TransitionMultRibbonInfo::create(vec!['ç'], vec!['ç'], vec![],),
        Err(TuringTransitionError::TransitionArgsError(_val))
    ));

    assert_eq!(t1.chars_read, vec!('ç', 'ç'));
    assert_eq!(t1.move_read, TuringDirection::Right);
    assert_eq!(t1.chars_write, vec!(('ç', TuringDirection::None)));

    assert_eq!(t1.get_number_of_affected_tapes(), 2)
}

#[test]
fn create_ill_transitions() {
    assert!(matches!(
        TransitionMultRibbonInfo::create(
            vec!['$', '_'],
            vec!['_'],
            vec![TuringDirection::Right, TuringDirection::None],
        ),
        Err(TuringTransitionError::IllegalActionError(_val))
    ));

    assert!(matches!(
        TransitionMultRibbonInfo::create(
            vec!['ç', '_'],
            vec!['_'],
            vec![TuringDirection::Left, TuringDirection::None],
        ),
        Err(TuringTransitionError::IllegalActionError(_val))
    ));

    assert!(matches!(
        TransitionMultRibbonInfo::create(
            vec!['_', 'ç'],
            vec!['ç'],
            vec![TuringDirection::None, TuringDirection::Left],
        ),
        Err(TuringTransitionError::IllegalActionError(_val))
    ));

    assert!(matches!(
        TransitionMultRibbonInfo::create(
            vec!['_', '_'],
            vec!['ç'],
            vec![TuringDirection::None, TuringDirection::Left],
        ),
        Err(TuringTransitionError::IllegalActionError(_val))
    ));

    assert!(matches!(
        TransitionMultRibbonInfo::create(
            vec!['_', 'ç'],
            vec!['_'],
            vec![TuringDirection::None, TuringDirection::Left],
        ),
        Err(TuringTransitionError::IllegalActionError(_val))
    ));
}

#[test]
fn transition_eq() {
    let t1 = TransitionMultRibbonInfo::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::Right],
    )
    .unwrap();

    assert_ne!(
        t1,
        TransitionMultRibbonInfo::create(
            vec!('ç', 'ç'),
            vec!('ç'),
            vec!(TuringDirection::Right, TuringDirection::Right)
        )
        .unwrap()
    );

    assert_ne!(
        t1,
        TransitionMultRibbonInfo::create(
            vec!('ç', 'v'),
            vec!('_'),
            vec!(TuringDirection::Right, TuringDirection::Right)
        )
        .unwrap()
    );

    assert_ne!(
        t1,
        TransitionMultRibbonInfo::create(
            vec!('ç', 'v'),
            vec!('t'),
            vec!(TuringDirection::Right, TuringDirection::Right)
        )
        .unwrap()
    );

    assert_ne!(
        t1,
        TransitionMultRibbonInfo::create(
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

    assert_eq!(
        t1,
        TransitionMultRibbonInfo::create(
            vec!('ç', 'ç'),
            vec!('ç'),
            vec!(TuringDirection::None, TuringDirection::Right)
        )
        .unwrap()
    );
}
