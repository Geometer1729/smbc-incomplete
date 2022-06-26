use crate::types::*;
use std::collections::HashSet;

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

fn syms(b : Board) -> Vec<Board> {
    let mut v:Vec<Board> = vec![b];
    for i in 0..4 {
        v.push(rot(v[i]));
    }
    let v2 : Vec<Board> =v.iter().map(|&b|flip(b)).collect();
    v.extend(v2);
    v
}

pub fn cannon (p: &mut Pos) {
    p.board = *syms(p.board).iter().min().unwrap();
}

pub fn cannon_vec (vec: &mut Vec<Pos>) -> Vec<Pos> {
    let mut h : HashSet<Pos> = HashSet::with_capacity(9);
    for pos in vec {
        cannon(pos);
        h.insert(*pos);
    }
    h.into_iter().collect()
}
