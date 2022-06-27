use crate::types::{Player::*, Square::*, *};

// Eval meaning:
// possible to force win for x
// possible to force win for o
// possible to force draw
// possible to prevent win for x
// possible to prevent win for o
// possible to prevent draw

pub fn combine(opts: Vec<Eval>) -> Eval {
    let mut eval = [[false;3];2];
    for i in 0..3 {
        // outcome can be forced if there's an option where it can't be prevented
        eval[0][i] = opts.iter().any(|ent|!ent[1][i]);
        // outcome can be prevented if there's an option where it can't be forced
        eval[1][i] = opts.iter().any(|ent|!ent[0][i]);
    }
    eval
}

pub fn eval_pos(p: Pos) -> Option<Player> {
    let b = p.board;
    if win_sets
        .iter()
        .any(|ws| ws.iter().all(|&s| b[s] == Taken { by: X }))
    {
        Some(X)
    } else if win_sets
        .iter()
        .any(|ws| ws.iter().all(|&s| b[s] == Taken { by: O }))
    {
        Some(O)
    } else {
        None
    }
}

const win_sets: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

pub const x_won: Eval = [[true, false, false],[false, true, true]];

pub const draw: Eval = [[false, true, false],[true, false, true]];

pub const o_won: Eval = [[false, false, true],[true, true, false]];
