#[path = "types.rs"]
mod types;

#[path = "cannon.rs"]
mod cannon;

#[path = "eval.rs"]
mod eval;

pub mod moves{

// why on earth do I need a moves here?
use crate::moves::eval::eval::*;
use crate::types::types::*;
use crate::cannon::cannon::*;
use std::collections::HashMap;

fn moves(p:Pos) -> Vec<Pos> {
    let mut v : Vec<Pos> = Vec::new();
    for i in 0..9 {
        if p.board[i] == Square::Open {
            let b = p.board;
            let t = p.turn;
            let o = other(t);
            let mut b2 = b.clone();
            b2[i] = Square::Taken{by:t};
            let np = Pos{turn:o,board:b2};
            cannon(np);
            v.push(np);
        }
    }
    return v;
}

const start : Pos = Pos
    {turn:Player::X
    ,board: [Square::Open;9]
    };

fn other(s:Player) -> Player {
    match s {
        // so much boiler plate I want to cry
        X => {return Player::O;}
        O => {return Player::X;}
    }
}

pub fn genTable() -> HashMap<Pos,Eval> {
    let mut all = HashMap::new();
    evaluate(&mut all,start);
    return all;
}

fn evaluate(all:&mut HashMap<Pos,Eval>, p:Pos) -> Eval {
    let mut evals = Vec::new();
    let ms = moves(p);
    let mut eval : [bool;6] = [false;6];

    match eval_pos(p) {
        Some(Player::X) => { eval = x_won; }
        Some(Player::O) => { eval = o_won; }
        None => {
            if ms.len() == 0 {
                eval = draw;
            } else {
                for np in ms {
                    evals.push(evaluate(all,np));
                }
                let eval = combine(evals);
            }
        }
    }
    all.insert(p,eval);
    return eval;
}

}
