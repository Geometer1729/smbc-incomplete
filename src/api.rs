use crate::eval::*;
use crate::format::*;
use crate::moves::*;
use crate::table::*;
use crate::types::{*,Player::*};

use std::io::*;

pub trait API {
    fn rend(&self,p:Pos) -> ();
    fn ask(&self,p:Pos) -> Pos;
}

pub struct Cnsl<'a>(pub &'a Table);

impl API for Cnsl<'_> {
    fn rend(&self,pos:Pos){
        let table = &self.0;
        println!("{}",pos);
        println!("{}",EvalShowable(cannon_lookup(&table,pos)));
    }
    fn ask(&self,pos:Pos) -> Pos {
        let table = &self.0;
        let ms : Vec<Pos>  = moves(pos);
        let mut display_moves : Vec<String> = Vec::new();

        for (i, m) in ms.iter().enumerate() {
            let eval = cannon_lookup(&table, *m);
            display_moves.push(format!("{}:\n{}\n{}", i, m, EvalShowable(eval)));
        }
        println!("{}", join_gridy(display_moves));
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        let index: usize = input.trim().parse().unwrap();
        ms[index]
    }
}

pub fn host_game<X,O> (x:X, o:O, p: Pos) -> Outcome
    where
        X: API,
        O: API,
{
    x.rend(p);
    o.rend(p);
    let new_pos =
        match p.turn {
            X => x.ask(p),
            O => o.ask(p)
        };
    if !moves(p).iter().any(|&m|m==new_pos) {
        panic!("illegal move encountered");
    }
    match eval_pos(new_pos) {
        Some(outcome) => outcome,
        None =>  host_game(x,o,new_pos)
    }
}
