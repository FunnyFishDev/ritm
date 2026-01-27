// accepts word that are like : x#x

accepting = q_a;

q_i { ç, ç -> R, ç, R } q_1;

q_1 { 1,_ -> R, 1, R 
    | 0,_ -> R, 0, R } q_1;

q_1 { #,_ -> N, _, L } q_2;

q_2 { #, 1 -> N, 1, L 
    | #, 0 -> N, 0, L } q_2;

q_2 { #,ç -> R, ç, R } q_3;

q_3 { 1, 1 -> R, 1, R
    | 0, 0 -> R, 0, R  } q_3;

q_3 { $, _ -> N, _, N } q_a;
