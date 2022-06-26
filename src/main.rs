#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(bench_black_box)]

pub mod ai;
pub mod cannon;
pub mod eval;
pub mod format;
pub mod moves;
pub mod table;
pub mod types;

use crate::ai::*;
use crate::eval::*;
use crate::format::*;
use crate::moves::*;
use crate::table::*;
use crate::types::*;

use crate::types::EvalShowable;
use crate::types::Outcome::*;
use crate::types::Player::*;

use std::io::*;
use std::time::Duration;
use std::time::Instant;

fn main() {
    if std::env::args().nth(1) == Some("bench".into()) {
        let mut runs = 0;
        let mut total_duration = Duration::ZERO;
        let mut min = Duration::MAX;
        let mut max = Duration::ZERO;

        while runs < 5000 {
            let st = Instant::now();

            std::hint::black_box(genTable());

            let dur = Instant::now().duration_since(st);
            total_duration += dur;
            runs += 1;
            if dur < min {
                min = dur;
            }
            if dur > max {
                max = dur;
            }
        }

        println!(
            "{}us average, {}us-{}us range ({})",
            (total_duration / runs).as_micros(),
            min.as_micros(),
            max.as_micros(),
            (max - min).as_micros()
        );
        return;
    }
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
