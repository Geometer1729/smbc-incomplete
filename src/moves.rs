pub mod moves{

//use crate::moves::eval::eval::*;

use crate::cannon::cannon::*;
use crate::eval::eval::*;
use crate::types::types::*;

use std::collections::HashMap;

pub fn moves(p:Pos) -> Vec<Pos> {
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
            if ! v.contains(&np) {
                v.push(np);
            }
        }
    }
    return v;
}

pub const start : Pos = Pos
    {turn:Player::X
    ,board: [Square::Open;9]
    };

fn other(s:Player) -> Player {
    match s {
        // so much boiler plate I want to cry
        Player::X => {return Player::O;}
        Player::O => {return Player::X;}
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
    let eval : [bool;6];

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
                eval = combine(evals);
            }
        }
    }
    all.insert(p,eval);
    return eval;
}

}
