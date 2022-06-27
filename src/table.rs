use crate::cannon::*;
use crate::eval::*;
use crate::moves::*;
use crate::types::*;
use rayon::prelude::*;
pub fn gen_table() -> Table {
    let all: Table = Default::default();
    evaluate(all.clone(), START);
    all
}

pub fn cannon_lookup(table: &Table, pos: Pos) -> Eval {
    let mut cannon_pos = pos.clone();
    cannon(&mut cannon_pos);
    *table.get(&cannon_pos).unwrap()
}

fn evaluate(all: Table, p: Pos) -> Eval {
    let eval: Eval = match eval_pos(p) {
        Some(Player::X) => X_WON,
        Some(Player::O) => O_WON,
        None => {
            let mut raw_moves = moves(p);
            let moves = cannon_vec(&mut raw_moves);
            if moves.len() == 0 {
                DRAW
            } else {
                combine(
                    moves
                        .par_iter()
                        .map(|new_position| {
                            let all = all.clone();
                            let lookup = all.get(new_position).map(|x| *x);
                            lookup.unwrap_or_else(|| evaluate(all, *new_position))
                        })
                        .collect(),
                )
            }
        }
    };
    all.insert(p, eval);
    eval
}

pub fn check_table(table: &Table, p:Pos) {
    let eval_lkp : Eval = *table.get(&cannon_pos(p)).unwrap();
    let mut evals : Vec<Eval> = Vec::new();
    for m in moves(p) {
        match (*table).get(&cannon_pos(m)) {
            Some(eval) => evals.push(*eval),
            None => panic!("pos not in table:\n{}",m)
        }
    }
    let eval_calc : Eval = match eval_pos(p) {
        Some(Player::X) => X_WON,
        Some(Player::O) => O_WON,
        None => if evals.len() == 0 { DRAW } else {combine(evals.clone())}
    };
    println!("works:{}\neval lkp:{}\neval cacl:{}\nevals:"
        ,eval_lkp == eval_calc
        ,EvalShowable(eval_lkp)
        ,EvalShowable(eval_calc)
        );
    for e in evals {
        println!("{}",EvalShowable(e));
    }
}
