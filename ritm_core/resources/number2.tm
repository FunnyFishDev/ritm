// Increments the binary number given as input by one

initial = q_init;
accepting = q_accept;

q_init { ç -> ç, R } q_1;

q_1 { 1 -> 1, R } q_1;
q_1 { _ -> 0, L } q_2;
q_2 { 1 -> 0, L } q_2;
q_2 { ç -> ç, R } q_3;
q_3 { 0 -> 1, L } q_accept;


q_1 { 0 -> 0, R } q_7;
q_7 { 0 -> 0, R
    | 1 -> 1, R } q_7;
q_7 { _ -> _, L } q_6;
q_6 { 1 -> 0, L } q_6;
q_6 { 0 -> 1, L } q_5;
q_5 { 0 -> 0, L
    | 1 -> 1, L } q_5;

q_5 { ç -> ç, N } q_accept;