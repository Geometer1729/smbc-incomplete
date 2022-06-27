use crate::types::{Outcome::*,Player::*, Square::*,*};

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

pub fn eval_pos(p: Pos) -> Option<Outcome> {
    let b = p.board;
    if WIN_SETS
        .iter()
        .any(|ws| ws.iter().all(|&s| b[s] == Taken { by: X }))
    {
        Some(Win{with:X})
    } else if WIN_SETS
        .iter()
        .any(|ws| ws.iter().all(|&s| b[s] == Taken { by: O }))
    {
        Some(Win{with:O})
    } else if p.board.iter().all(|&x|x != Open) {
        Some(Draw)
    } else {
        None
    }
}

const WIN_SETS: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

pub const X_WON: Eval = [[true, false, false],[false, true, true]];

pub const DRAW: Eval = [[false, true, false],[true, false, true]];

pub const O_WON: Eval = [[false, false, true],[true, true, false]];
