accepting = q_a;

// Initialisation
q_i {ç, ç -> R, ç, R} q_copy;

// Copy to write ribbon
q_copy {0, _ -> R, 0, R} q_copy;
q_copy {1, _ -> R, 1, R} q_copy;
q_copy {$, _ -> L, _, N} q_return;

// Reset reading ribbon position
q_return {0, _ -> L, _, N} q_return;
q_return {1, _ -> L, _, N} q_return;
q_return {ç, _ -> R, _, L} q_check;

// Compare each side until end
q_check {0, 0 -> R, 0, L} q_check;
q_check {1, 1 -> R, 1, L} q_check;
q_check {$, ç -> N, ç, N} q_a;