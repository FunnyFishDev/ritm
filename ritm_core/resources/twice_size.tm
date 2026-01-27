// L = {a^i u | u \in {u \in {a, b}^* |u| = 2i}}

// exemple of a valid input : aaaaaabab
// i = 3 : aaa, |aaabab| = 6

accepting = q_a;

q_i { ç, ç -> R, ç, R } q_1;


q_1 { a, _ -> R, a, R } q_1;

q_1 { a, _ -> N, _, L
    | b, _ -> N, _, L} q_2;


q_2 { a, a -> R, a, L
    | b, a -> R, a, L } q_2;

q_2 { a, ç -> N, ç, R 
    | b, ç -> N, ç, R } q_3;


q_3 { a, a -> R, a, R
    | b, a -> R, a, R } q_3;


q_3 { $, _ -> N, _, N } q_a;



// empty word
q_i { ç, ç -> R, ç, R } q_4;

q_4 { $, _ -> N, _, N } q_a;