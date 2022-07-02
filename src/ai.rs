use crate::moves::*;
use crate::table::*;
use crate::types::Outcome::*;
use crate::types::Player::*;
use crate::types::Pref::*;
use crate::types::*;
use crate::api::API;

use std::iter::*;

pub struct SimpleAi<'a> {pub table:&'a Table,pub obj:Objective}

impl API for SimpleAi<'_> {
    fn rend(&self,_:Pos) {}
    fn ask(&self,pos:Pos) -> Pos {
        let m = *moves(pos).iter()
            .max_by_key(
                |&m|pref(self.obj,cannon_lookup(self.table,*m))
            ).unwrap();
        println!("Simple Ai selected:\n{}",m);
        println!("Which has an eval of:{:?}",pref(self.obj,cannon_lookup(self.table,m)));
        m
    }
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
