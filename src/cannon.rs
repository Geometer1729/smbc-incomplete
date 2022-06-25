#[path = "types.rs"]
mod types;

pub mod cannon {

use crate::types::types::*;
use std::cmp::min;

fn flip (b:Board) -> Board {
    return
        [ b[2],b[1],b[0]
        , b[5],b[4],b[3]
        , b[8],b[7],b[6]
        ];
}

fn rot(b:Board) -> Board {
    return
        [ b[2],b[5],b[8]
        , b[1],b[4],b[7]
        , b[0],b[3],b[6]
        ];

}

fn swap2(mut b : Board,i:usize,j:usize) {
    let t = b[i];
    b[i] = b[j];
    b[j]=t;
}

fn swap4(mut b : Board,i1:usize,i2:usize,i3:usize,i4:usize) -> () {
    let t = b[i1];
    b[i1] = b[i2];
    b[i2] = b[i3];
    b[i3] = b[i4];
    b[i4] = t;
}

fn syms(b : Board) -> Vec<Board> {
    let mut v:Vec<Board> = vec![b];
    for i in 0..4 { // hopefully this means 0..3
        v.push(rot(v[i]));
    }
    for i in 0..4 { // hopefully this means 0..3
        v.push(flip(v[i]));
    }
    return v;
}

pub fn borad_cannon (mut b:Board) {
    let b = b.iter().min();
}

pub fn cannon (mut p:Pos) {
    borad_cannon(p.board);
}

}
