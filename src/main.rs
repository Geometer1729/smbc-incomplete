#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use smbc_incomplete::Outcome::*;
use smbc_incomplete::Player::*;
use smbc_incomplete::*;

use std::io::*;

fn main() {
    let table = gen_table();
    println!("{}",Pref::Winable > Pref::Drawable);
    println!("{}",Pref::Drawable > Pref::Lost);
    println!("{}",Pref::Winable > Pref::Lost);
    //check_table(&table,start);
    explore(table,START);
}

fn explore(table:Table,pos:Pos) {
    println!("{}",pos);
    println!("{}",EvalShowable(cannon_lookup(&table,pos)));
    let ms : Vec<Pos>  = moves(pos);
    let mut display_moves : Vec<String> = Vec::new();

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
    for (i,m) in moves(pos).iter().enumerate() {
        if next == *m {
            println!("choice was:{}",i)
        }
    }

    println!("====================");
    if moves(next).len() == 0 || eval_pos(next).is_some() {
        let res =
            match eval_pos(next) {
                Some(X) => "X won",
                Some(O) => "O won",
                None => "Draw"
            };
        println!("Game ended : {}",res);
        return;
    }
    //check_table(&table,next);
    explore(table,next);
}
