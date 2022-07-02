use crate::eval::*;
use crate::format::*;
use crate::moves::*;
use crate::table::*;
use crate::types::{*,Player::*};
use async_trait::async_trait;
use async_recursion::async_recursion;

use std::io::*;

#[async_trait]
pub trait API {
    async fn rend(&mut self,p:Pos) -> ();
    async fn ask(&mut self,p:Pos) -> Pos;
}

pub struct Cnsl<'a>(pub &'a Table);

#[async_trait]
impl API for Cnsl<'_> {
    async fn rend(&mut self,pos:Pos){
        let table = &self.0;
        println!("{}",pos);
        println!("{}",EvalShowable(cannon_lookup(&table,pos)));
    }
    async fn ask(&mut self,pos:Pos) -> Pos {
        let table = &self.0;
        let ms : Vec<Pos>  = moves(pos);
        let mut display_moves : Vec<String> = Vec::new();

        for (i, m) in ms.iter().enumerate() {
            let eval = cannon_lookup(&table, *m);
            display_moves.push(format!("{}:\n{}\n{}", i, m, EvalShowable(eval)));
        }
        println!("{}", join_gridy(display_moves));
        let mut input = String::new();
        ask_range(ms,&mut input)
    }
}

fn ask_range<A>(ms:Vec<A>,input : &mut String) -> A
    where A : Copy,
{
    stdin().read_line(input).unwrap();
    match input.trim().parse::<usize>() {
        Ok(ind) if ind < ms.len() => ms[ind],
        _ => {
            println!("Index did not parse or was out of bounds");
            input.clear();
            ask_range(ms,input)
        }
    }
}

#[async_recursion]
pub async fn host_game<X,O> (x:&mut X,o:&mut O, p: Pos) -> Outcome
    where
        X: API + Send,
        O: API + Send,
{
    x.rend(p).await;
    o.rend(p).await;
    let new_pos =
        match p.turn {
            X => x.ask(p).await,
            O => o.ask(p).await
        };
    if !moves(p).iter().any(|&m|m==new_pos) {
        panic!("illegal move encountered");
    }
    //eval_pos(new_pos).unwrap_or_else(async ||host_game(x,o,new_pos).await)
    match eval_pos(new_pos) {
        Some(outcome) => outcome,
        None =>  host_game(x,o,new_pos).await
    }
}
