#[path = "types.rs"]
mod types;
#[path = "cannon.rs"]
mod cannon;
#[path = "moves.rs"]
mod moves;

use crate::moves::moves::*;

fn main() {
    let t = genTable();
    println!("{}",t.len());
}


