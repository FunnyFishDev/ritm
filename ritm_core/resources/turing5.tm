accepting = q_a;

// Only accepts words : x1y with |x| = |y|

q_i { ç, ç -> R, ç, R } q_1;

q_1 { 0, _ -> R, a, R
    | 1, _ -> R, a, R } q_1;

q_1 { 1, _ -> R, _, L } q_2;

q_2 { 0, a -> R, a, L 
    | 1, a -> R, a, L } q_2;

q_2 { $, ç -> N, ç, N } q_a;