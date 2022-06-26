use crate::moves::*;
use crate::table::*;
use crate::types::Outcome::*;
use crate::types::Player::*;
use crate::types::Pref::*;
use crate::types::*;

use std::iter::*;

pub fn simple(table: &Table, obj: Objective, pos: Pos) -> Pos {
    let ms = moves(pos);
    let evals = ms.iter().map(|&p| cannon_lookup(&table, p));
    let mut labeled = zip(ms.clone(), evals);
    match labeled.find(|(_, eval)| pref(obj, *eval) == Winable) {
        Some((m, _)) => m,
        None => match labeled.find(|(_, eval)| pref(obj, *eval) == Drawable) {
            Some((m, _)) => m,
            None => ms[0].clone(),
        },
    }
}

fn toInd(obj: Outcome) -> usize {
    match obj {
        Win { with: X } => 0,
        Draw => 1,
        Win { with: O } => 2,
    }
}

fn pref(obj: Objective, eval: Eval) -> Pref {
    let w = toInd(obj[0]);
    let _d = toInd(obj[1]);
    let l = toInd(obj[2]);
    if !eval[3 + w] {
        Winable
    } else if eval[l] {
        Lost
    } else {
        Drawable
    }
}
