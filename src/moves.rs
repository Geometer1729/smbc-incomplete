#[path = "types.rs"]
mod types;
#[path = "cannon.rs"]
mod cannon;

pub mod moves{

use crate::types::types::*;
use crate::cannon::cannon::*;
use std::collections::HashSet;

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
            cannon(np)
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

pub fn genTable() -> HashSet<Pos> {
    let mut all = HashSet::new();
    all.insert(start);
    let mut queue = vec![start];
    while true {
        match queue.pop() {
            None => {break;}
            Some(p) => {
                for np in moves(p) {
                    if all.insert(np) {
                        queue.push(np);
                    }

                }
            }
        }
    }

    return all;
}

}
