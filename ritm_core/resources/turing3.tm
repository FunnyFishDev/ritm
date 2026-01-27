accepting = q_a;

// Turing machine that only accepts words of the form : xx
q_i {ç, ç -> R, ç, R} q_1;

// Get to the end of the word
q_1 {0, _ -> R, 0, R
    |1, _ -> R, 1, R} q_1;

// At some point, leave q1 to q2
q_1 {0, _ -> N, _, L
    |1, _ -> N, _, L} q_2;

// Get to the end of the writing tape
q_2 { 0, 0 -> N, 0, L
    | 0, 1 -> N, 1, L
    | 1, 0 -> N, 0, L
    | 1, 1 -> N, 1, L} q_2;


// When at the end of the writing tape
q_2 { 0, ç -> N, ç, R 
    | 1, ç -> N, ç, R } q_3;

// Move at the end of both tape
q_3 { 0, 0 -> R, 0, R 
    | 1, 1 -> R, 1, R } q_3;


// When both ends are reached AT THE SAME TIME, accept the word
q_3 { $, _ -> N, _, N } q_a;