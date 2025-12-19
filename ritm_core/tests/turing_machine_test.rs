use std::any::Any;

use ritm_core::{
    SimpleTuringGraph, SimpleTuringMachine, turing_graph::TuringStateType, turing_machine::{Mode, TuringExecutionSteps, TuringMachines}, turing_tape::TuringTape, turing_transition::{TuringDirection, TuringTransition, TuringTransitionInfo}
};

const TM_ACCEPT_XX: &str = "// Turing machine that only accepts words of the form : xx
q_i {ç, ç -> R, ç, R} q_1;

q_1 {0, _ -> R, 0, R
    |1, _ -> R, 1, R} q_1;
q_1 {0, _ -> N, _, L
    |1, _ -> N, _, L} q_2;

q_2 { 0, 0 -> N, 0, L
    | 0, 1 -> N, 1, L
    | 1, 0 -> N, 0, L
    | 1, 1 -> N, 1, L} q_2;
q_2 { 0, ç -> N, ç, R
    | 1, ç -> N, ç, R } q_3;

q_3 { 0, 0 -> R, 0, R
    | 1, 1 -> R, 1, R } q_3;
q_3 { $, _ -> N, _, N } q_a;";

const TM_INF: &str = "// Turing machine is infinite
q_i {ç, ç -> N, ç, N} q_i;";

#[test]
fn save_all_accept() {
    let tm_graph = get_test_non_deter_graph();

    // let mut turing_machine = TuringMachine::new(tm_graph, String::from("010"), Mode::SaveAll).unwrap();

    let mut turing_machine =
        TuringMachines::new(tm_graph, String::from("0111110"), Mode::SaveAll).unwrap();

    // let mut turing_machine = TuringMachines::new(get_smaller_non_deter_graph(), String::from("0000000000000001"), Mode::SaveAll).unwrap();
    let mut saved_state = None;
    let mut counter = 0;
    for steps in &mut turing_machine {
        println!("_______________\nExec. step ::\n{}", steps);
        counter += 1;

        if counter == 1000 {
            return;
        }
        saved_state = Some(steps);
    }

    if let Some(state) = saved_state {
        match state {
            TuringExecutionSteps::FirstIteration {
                init_state: _,
                init_reading_tape: _,
                init_write_tapes: _,
            } => panic!("Wrong outcome"),
            TuringExecutionSteps::TransitionTaken {
                previous_state: _,
                reached_state,
                transition_index_taken: _,
                transition_taken: _,
                reading_tape: _,
                writing_tapes: _,
                iteration: _,
                state_pointer: _,
            } => {
                assert_eq!(TuringStateType::Accepting, reached_state.get_type())
            }
            TuringExecutionSteps::Backtracked {
                previous_state: _,
                reached_state: _,
                reading_tape: _,
                writing_tapes: _,
                iteration: _,
                state_pointer: _,
                backtracked_iteration: _,
            } => panic!("Wrong outcome"),
        }
    }
}

#[test]
fn save_all_not_accept() {
    let tm_graph = get_test_non_deter_graph();

    let mut turing_machine =
        TuringMachines::new(tm_graph, String::from("0100"), Mode::SaveAll).unwrap();

    let mut saved_state = None;
    for steps in &mut turing_machine {
        println!("_______________\nExec. step ::\n{}", steps);
        saved_state = Some(steps);
    }

    if let Some(state) = saved_state {
        match state {
            TuringExecutionSteps::FirstIteration {
                init_state: _,
                init_reading_tape: _,
                init_write_tapes: _,
            } => panic!("Wrong outcome"),
            TuringExecutionSteps::TransitionTaken {
                previous_state: _,
                reached_state,
                transition_index_taken: _,
                transition_taken: _,
                reading_tape: _,
                writing_tapes: _,
                iteration: _,
                state_pointer: _,
            } => {
                assert_ne!(TuringStateType::Accepting, reached_state.get_type())
            }
            TuringExecutionSteps::Backtracked {
                previous_state: _,
                reached_state: _,
                reading_tape: _,
                writing_tapes: _,
                iteration: _,
                state_pointer: _,
                backtracked_iteration: _,
            } => panic!("Wrong outcome"),
        }
    }
}

#[test]
fn stop_after() {
    let mut turing_machine = get_small_inf_machine(Mode::StopAfter(10000));

    let mut counter = 0;
    for _steps in &mut turing_machine {
        counter += 1;
    }

    assert_eq!(counter, 10000)
}

#[test]
fn stop_first_reject() {
    let tm_graph = get_test_non_deter_graph();

    let mut turing_machine =
        TuringMachines::new(tm_graph, String::from("010"), Mode::StopFirstReject).unwrap();

    // let mut turing_machine = TuringMachines::new(get_smaller_non_deter_graph(), String::from("0000000000000001"), Mode::SaveAll).unwrap();
    let mut counter = 0;
    let mut last_step = None;
    for steps in &mut turing_machine {
        counter += 1;

        if counter == 1000 {
            return;
        }

        last_step = Some(steps);
    }

    if let Some(step) = last_step
        && let TuringExecutionSteps::TransitionTaken {
            previous_state: _,
            reached_state,
            transition_index_taken: _,
            transition_taken: _,
            reading_tape: _,
            writing_tapes: _,
            iteration: _,
            state_pointer: _,
        } = step
    {
        assert_ne!(reached_state.get_type(), TuringStateType::Accepting);
        return;
    }
    panic!("The iteration didn't stop like it was supposed to");
}

/// Gets a graph that forces one complete descent before doing one backtracking and finishing.
/// Feed it `0...0` in order for it to suceed
fn _get_smaller_non_deter_graph() -> SimpleTuringGraph {
    let q2 = &String::from("q2");
    let mut graph = SimpleTuringGraph::new(1, true).unwrap();

    graph.add_state(q2, TuringStateType::Normal);

    let mut transition = TuringTransitionInfo::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::Right, TuringDirection::Right],
    )
    .unwrap();
    graph
        .append_transition("i", transition.clone(), q2)
        .unwrap();

    transition = TuringTransitionInfo::create(
        vec!['0', '_'],
        vec!['_'],
        vec![TuringDirection::Right, TuringDirection::None],
    )
    .unwrap();
    graph
        .append_transition(q2, transition.clone(), q2)
        .unwrap();

    transition = TuringTransitionInfo::create(
        vec!['0', '_'],
        vec!['_'],
        vec![TuringDirection::Right, TuringDirection::None],
    )
    .unwrap();
    graph
        .append_transition(q2, transition.clone(), "a")
        .unwrap();

    graph
}

/// Gets a non determinist Turing machine graph.
/// Return a turing machine that checks if a given inputed value is of the form `x1y` with `|x| = |y|`.
fn get_test_non_deter_graph() -> SimpleTuringGraph {
    let q1 = &String::from("q1");
    let q2 = &String::from("q2");
    let mut graph = SimpleTuringGraph::new(1, true).unwrap();

    graph.add_state(q1, TuringStateType::Normal);
    graph.add_state(q2, TuringStateType::Normal);

    // q_0 -> {ç, ç, => R, ç, R} -> q_1
    let mut transition = TuringTransitionInfo::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::Right, TuringDirection::Right],
    )
    .unwrap();
    graph
        .append_transition("i", transition.clone(), q1)
        .unwrap();

    // q_1 -> {0, _ => R, a, R} -> q_1
    transition = TuringTransitionInfo::create(
        vec!['0', '_'],
        vec!['a'],
        vec![TuringDirection::Right, TuringDirection::Right],
    )
    .unwrap();
    graph
        .append_transition(q1, transition.clone(), q1)
        .unwrap();
    // q_1 -> {1, _ => R, a, R} -> q_1
    transition = TuringTransitionInfo::create(
        vec!['1', '_'],
        vec!['a'],
        vec![TuringDirection::Right, TuringDirection::Right],
    )
    .unwrap();
    graph
        .append_transition(q1, transition.clone(), q1)
        .unwrap();

    // q_1 -> {1, _ => R, _, L} -> q_2
    transition = TuringTransitionInfo::create(
        vec!['1', '_'],
        vec!['_'],
        vec![TuringDirection::Right, TuringDirection::Left],
    )
    .unwrap();
    graph
        .append_transition(q1, transition.clone(), q2)
        .unwrap();

    // q_2 -> {0, a => R, a, L} -> q_2
    transition = TuringTransitionInfo::create(
        vec!['0', 'a'],
        vec!['a'],
        vec![TuringDirection::Right, TuringDirection::Left],
    )
    .unwrap();
    graph
        .append_transition(q2, transition.clone(), q2)
        .unwrap();

    // q_2 -> {1, a => R, a, L} -> q_2
    transition = TuringTransitionInfo::create(
        vec!['1', 'a'],
        vec!['a'],
        vec![TuringDirection::Right, TuringDirection::Left],
    )
    .unwrap();
    graph
        .append_transition(q2, transition.clone(), q2)
        .unwrap();

    // q_2 -> {$, ç => N, ç, N} -> a
    transition = TuringTransitionInfo::create(
        vec!['$', 'ç'],
        vec!['ç'],
        vec![TuringDirection::None, TuringDirection::None],
    )
    .unwrap();
    graph
        .append_transition(q2, transition.clone(), "a")
        .unwrap();

    graph
}

fn get_small_inf_machine(mode: Mode) -> SimpleTuringMachine {
    let q1 = &String::from("q1");

    let mut graph = SimpleTuringGraph::new(1, true).unwrap();
    graph.add_state(q1, TuringStateType::Normal);

    // q_0 -> {ç, ç, => R, ç, R} -> q_1
    let transition = TuringTransitionInfo::create(
        vec!['ç', 'ç'],
        vec!['ç'],
        vec![TuringDirection::Right, TuringDirection::Right],
    )
    .unwrap();
    graph
        .append_transition("i", transition.clone(), q1)
        .unwrap();

    // q_1 -> {1, _, => N, _, N} -> q_1
    let transition = TuringTransitionInfo::create(
        vec!['1', '_'],
        vec!['_'],
        vec![TuringDirection::None, TuringDirection::None],
    )
    .unwrap();
    graph
        .append_transition(q1, transition.clone(), q1)
        .unwrap();

    TuringMachines::new(graph, String::from("1"), mode).unwrap()
}

// #[test]
// fn get_path_to_accept_test() {
//     let tm = parse_turing_graph_string(TM_ACCEPT_XX.to_string()).unwrap();

//     let mut tm = TuringMachines::new(tm, String::from("1010"), Mode::SaveAll).unwrap();

//     let mut count = 0;
//     let path = tm
//         .get_path_to_accept(|| {
//             count += 1;
//             count <= 2000
//         })
//         .unwrap();

//     let mut path_iter = path.iter();
//     // Skip first step
//     let first_step = path_iter.next().unwrap();

//     let mut reading_tape = first_step.get_reading_tape().clone();
//     let mut writting_tapes = first_step.get_writing_tapes().clone();

//     let mut last_step_type = first_step.get_current_state().get_type();
//     // Check that the path leads to the correct output.
//     tm.reset();
//     for step in path_iter {
//         last_step_type = step.get_current_state().get_type();
//         match &step {
//             TuringExecutionSteps::TransitionTaken {
//                 previous_state: _,
//                 reached_state: _,
//                 state_pointer: _,
//                 transition_index_taken: _,
//                 transition_taken,
//                 reading_tape: _,
//                 writing_tapes: _,
//                 iteration: _,
//             } => {
//                 assert!(
//                     reading_tape
//                         .try_apply_transition(
//                             transition_taken.chars_read[0],
//                             ' ',
//                             &transition_taken.move_read
//                         )
//                         .unwrap()
//                 );
//                 #[allow(clippy::needless_range_loop)]
//                 for i in 0..(transition_taken.get_number_of_affected_tapes() - 1) {
//                     assert!(
//                         writting_tapes[i]
//                             .try_apply_transition(
//                                 transition_taken.chars_read[i + 1],
//                                 transition_taken.chars_write[i].0,
//                                 &transition_taken.chars_write[i].1
//                             )
//                             .unwrap()
//                     );
//                 }
//             }
//             TuringExecutionSteps::Backtracked {
//                 previous_state: _,
//                 reached_state: _,
//                 state_pointer: _,
//                 reading_tape: _,
//                 writing_tapes: _,
//                 iteration: _,
//                 backtracked_iteration: _,
//             } => {
//                 panic!("No backtracking step was supposed to be found here");
//             }
//             TuringExecutionSteps::FirstIteration {
//                 init_state: _,
//                 init_reading_tape: _,
//                 init_write_tapes: _,
//             } => {
//                 panic!("Wrong step struct found");
//             }
//         }
//     }
//     // Of course the last state must also be the accepting one
//     assert_eq!(TuringStateType::Accepting, last_step_type)
// }

// #[test]
// fn get_path_to_accept_exit_condition_test() {
//     // Checks that the exist condition works by using an infinite turing machine
//     let tm = parse_turing_graph_string(TM_INF.to_string()).unwrap();

//     // Here the mode will not allow the machine to end, therefore only the exit condition can force the execution to stop
//     let mut tm = TuringMachines::new(tm, String::from("1"), Mode::SaveAll).unwrap();
//     let mut count = 0;
//     let path = tm.get_path_to_accept(|| {
//         count += 1;
//         count <= 10000
//     });

//     if path.is_some() {
//         panic!("Expected no path to be found but a value was returned.");
//     }
// }

// #[test]
// fn get_path_to_accept_exit_mode_test() {
//     // Checks that the exist condition works by using an infinite turing machine
//     let tm = parse_turing_graph_string(TM_INF.to_string()).unwrap();

//     let mut tm = TuringMachines::new(tm, String::from("1"), Mode::StopAfter(10)).unwrap();
//     // No exit condition, therefore it could loop forever, if not for the mode
//     let path = tm.get_path_to_accept(|| true);

//     if path.is_some() {
//         panic!("Expected no path to be found but a value was returned.");
//     }
// }

// #[test]
// fn get_path_to_accept_rejected_test() {
//     let tm = parse_turing_graph_string(TM_ACCEPT_XX.to_string()).unwrap();

//     let mut tm = TuringMachines::new(tm, String::from("10101"), Mode::SaveAll).unwrap();

//     // Checks that it returns none when no path exists (no inf loop here)

//     let path = tm.get_path_to_accept(|| true);

//     if path.is_some() {
//         panic!("Expected no path to be found but a value was returned.");
//     }
// }
