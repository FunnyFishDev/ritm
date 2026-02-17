// Only accepts words of the form : 0^n 1^n

initial = q_init;
accepting = q_accept;

q_init { ç -> ç, R } q_1;

q_1 { 0 -> a, R } q_2;

q_1 { _ -> 0, N } q_accept;

q_2 { 0 -> 0, R
    | b -> b, R } q_2;

q_2 { 1 -> b, L } q_3;

q_3 { 0 -> 0, L
    | b -> b, L } q_3;

q_3 { a -> a, R } q_1;

q_1 { b -> b, R } q_4;


q_4 { b -> b, R } q_4;


q_4 { _ -> _, N } q_accept;
