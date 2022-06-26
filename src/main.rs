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
use crate::types::*;
use crate::format::*;
use crate::eval::*;
use crate::types::EvalShowable;

use std::collections::HashMap;
use std::io::*;

fn main() {
    let table = genTable();
    //println!("{}",table.len());
    explore(table,start);
}

fn explore(table:HashMap<Pos,Eval>,pos:Pos) {
    println!("{}",pos);
    println!("{}",EvalShowable(*table.get(&start).unwrap()));
    let ms : Vec<Pos>  = moves(pos);
    let mut display_moves : Vec<String> = Vec::new();

    for (i,m) in ms.iter().enumerate() {
        let eval = cannon_lookup(&table,*m);
        display_moves.push(format!("{}:\n{}\n{}",i,m,EvalShowable(eval)));
    }
    println!("{}",join_gridy(display_moves));
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    let index : usize = input.trim().parse().unwrap();
    println!("went with move:{}\n======================",index);
    let new = ms[index];
    if moves(new).len() == 0 || eval_pos(new).is_some() {
        println!("Game ended");
        return;
    }
    explore(table,new);
}


