use crate::types::*;
use crate::moves::*;
use crate::types::Pref::*;
use crate::types::Objective::*;
use crate::types::Player::*;
use crate::table::*;

use std::iter::*;

pub fn simple(table:&Table,obj:Objective,pos:Pos) -> Pos {
    let ms = moves(pos);
    let evals = ms.iter().map(|&p|cannon_lookup(&table,p));
    let mut labeled = zip(ms.clone(),evals);
    match labeled.find(|(_,eval)|pref(obj,*eval) == Winable) {
        Some((m,_)) => m,
        None => match labeled.find(|(_,eval)|pref(obj,*eval) == HalfWinable){
            Some((m,_)) => m,
            None => ms[0].clone()
        }
    }
}

fn toInd(obj:Objective) -> (usize,usize,usize) {
    match obj {
        Win{with:X} => (0,1,2),
        Draw => (1,0,2),
        Win{with:O} => (2,0,1)
    }
}

fn pref(obj:Objective,eval:Eval) -> Pref {
    let (i,o1,o2) = toInd(obj);
    if !eval[3+i] {
        Winable
    } else if eval[o1] && eval [o2] {
        Lost
    } else {
        HalfWinable
    }
}

