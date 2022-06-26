#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use smbc_incomplete::Outcome::*;
use smbc_incomplete::Player::*;
use smbc_incomplete::*;

use std::io::*;

fn main() {
    let table = genTable();
    //println!("{}",table.len());
    explore(table, start);
}

fn explore(table: Table, pos: Pos) {
    println!("{}", pos);
    println!("{}", EvalShowable(*table.get(&start).unwrap()));
    let ms: Vec<Pos> = moves(pos);
    let mut display_moves: Vec<String> = Vec::new();

    for (i, m) in ms.iter().enumerate() {
        let eval = cannon_lookup(&table, *m);
        display_moves.push(format!("{}:\n{}\n{}", i, m, EvalShowable(eval)));
    }
    println!("{}", join_gridy(display_moves));

    let next = match pos.turn {
        Player::X => {
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            let index: usize = input.trim().parse().unwrap();
            ms[index]
        }
        Player::O => simple(&table, [Win { with: O }, Win { with: X }, Draw], pos),
    };

    println!("====================");
    if moves(next).len() == 0 || eval_pos(next).is_some() {
        println!("Game ended");
        return;
    }
    explore(table, next);
}
