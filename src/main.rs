#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

mod cannon;
mod eval;
mod format;
mod moves;
mod table;
mod types;

use crate::moves::*;
use crate::table::*;
use crate::types::EvalShowable;

fn main() {
    let t = genTable();
    //println!("{}",t.len());
    println!("{}",EvalShowable(*t.get(&start).unwrap()));
    let ms = moves(start);
    for (i,m) in ms.iter().enumerate() {
        println!("{}:\n{}\n{}",i,m,EvalShowable(*t.get(&m).unwrap()));
    }
}


