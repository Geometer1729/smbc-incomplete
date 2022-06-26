use crate::eval::*;
use crate::types::*;
use crate::moves::*;
use crate::cannon::*;

use std::collections::HashMap;


pub fn genTable() -> HashMap<Pos,Eval> {
    let mut all = HashMap::new();
    evaluate(&mut all,start);
    all
}

pub fn cannon_lookup(table:& HashMap<Pos,Eval>,pos:Pos) -> Eval {
    let mut cannon_pos = pos.clone();
    cannon(&mut cannon_pos);
    *table.get(&cannon_pos).unwrap()
}

fn evaluate(all:&mut HashMap<Pos,Eval>, p:Pos) -> Eval {
    let eval : Eval =
        match eval_pos(p) {
            Some(Player::X) =>
                { x_won }
            Some(Player::O) =>
                { o_won }
            None =>
                {
                    let mut raw_moves = moves(p);
                    let moves = cannon_vec(&mut raw_moves);
                    if moves.len() == 0 {
                        draw
                    } else {
                        combine(
                            moves.iter().map(|new_position|{
                                let lookup = all.get(new_position).map(|&x|x) ;
                                lookup.unwrap_or_else(||evaluate(all,*new_position))
                            }).collect()
                        )
                    }
                }
        };
    all.insert(p,eval);
    eval
}
