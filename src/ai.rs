use crate::moves::*;
use crate::table::*;
use crate::types::Outcome::*;
use crate::types::Player::*;
use crate::types::Pref::*;
use crate::types::*;

use std::iter::*;

pub fn simple(table: &Table, obj: Objective, pos: Pos) -> Pos {
    let m = *moves(pos).iter()
        .max_by_key(
            |&m|pref(obj,cannon_lookup(table,*m))
        ).unwrap();
    println!("Move selected:\n{}",m);
    println!("Eval of:{:?}",pref(obj,cannon_lookup(table,m)));
    m
}

fn to_ind(obj: Outcome) -> usize {
    match obj {
        Win { with: X } => 0,
        Draw => 1,
        Win { with: O } => 2,
    }
}

fn pref(obj: Objective, eval: Eval) -> Pref {
    let w = to_ind(obj[0]);
    let _d = to_ind(obj[1]);
    let l = to_ind(obj[2]);
    if !eval[1][w] {
        Winable
    } else if eval[0][l] {
        Lost
    } else {
        Drawable
    }
}
