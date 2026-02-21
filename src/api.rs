use crate::eval::*;
use crate::format::*;
use crate::moves::*;
use crate::table::*;
use crate::types::{Player::*, *};

use std::future::Future;
use std::io::*;

pub trait API: Send {
    fn rend(&mut self, _p: Pos) -> impl Future<Output = ()> + Send {
        async {}
    }
    fn ask(&mut self, p: Pos) -> impl Future<Output = Pos> + Send;
    fn close(&mut self, o: Outcome) -> impl Future<Output = ()> + Send {
        async move { println!("Game over with: {}", o) }
    }
}

pub struct Cnsl<'a>(pub &'a Table);

impl API for Cnsl<'_> {
    async fn rend(&mut self, pos: Pos) {
        let table = &self.0;
        println!("{}", pos);
        println!("{}", EvalShowable(cannon_lookup(&table, pos)));
    }
    async fn ask(&mut self, pos: Pos) -> Pos {
        let table = &self.0;
        let ms: Vec<Pos> = moves(pos);
        let mut display_moves: Vec<String> = Vec::new();

        for (i, m) in ms.iter().enumerate() {
            let eval = cannon_lookup(&table, *m);
            display_moves.push(format!("{}:\n{}\n{}", i, m, EvalShowable(eval)));
        }
        println!("{}", join_gridy(display_moves));
        let mut input = String::new();
        ask_range(ms, &mut input)
    }
}

fn ask_range<A>(ms: Vec<A>, input: &mut String) -> A
where
    A: Copy,
{
    stdin().read_line(input).unwrap();
    match input.trim().parse::<usize>() {
        Ok(ind) if ind < ms.len() => ms[ind],
        _ => {
            println!("Index did not parse or was out of bounds");
            input.clear();
            ask_range(ms, input)
        }
    }
}

pub async fn host_game(x: &mut impl API, o: &mut impl API, p: Pos) -> Outcome {
    x.rend(p).await;
    o.rend(p).await;
    let new_pos = match p.turn {
        X => x.ask(p).await,
        O => o.ask(p).await,
    };
    if !moves(p).iter().any(|&m| m == new_pos) {
        panic!("illegal move encountered");
    }
    match eval_pos(new_pos) {
        Some(outcome) => {
            x.rend(new_pos).await;
            o.rend(new_pos).await;
            x.close(outcome).await;
            o.close(outcome).await;
            outcome
        }
        None => Box::pin(host_game(x, o, new_pos)).await,
    }
}
