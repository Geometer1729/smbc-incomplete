#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
//#[allow(non_upper_case_globals)]
//#[allow(non_upper_case_globals)]

#[path = "types.rs"]
mod types;
#[path = "cannon.rs"]
mod cannon;
#[path = "moves.rs"]
mod moves;

use crate::moves::moves::*;
//use std::collections::HashMap;

fn main() {
    let t = genTable();
    println!("{}",t.len());
    println!("{:?}",t.get(&start));
    let ms = moves(start);
    for m in ms {
        println!("{:?},{:?}",m,t.get(&m));
    }

}


