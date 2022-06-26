use crate::types::*;

fn flip(b: Board) -> Board {
    return [b[2], b[1], b[0], b[5], b[4], b[3], b[8], b[7], b[6]];
}

fn rot(b: Board) -> Board {
    return [b[2], b[5], b[8], b[1], b[4], b[7], b[0], b[3], b[6]];
}

fn syms(b: Board) -> Vec<Board> {
    let mut v: Vec<Board> = vec![b];
    for i in 0..4 {
        let b = rot(v[i]);
        v.push(b);
    }
    for i in 0..4 {
        v.push(flip(v[i]));
    }
    v
}

pub fn cannon(p: &mut Pos) {
    p.board = *syms(p.board).iter().min().unwrap();
}

pub fn cannon_vec(vec: &mut Vec<Pos>) -> Vec<Pos> {
    let mut h: Vec<Pos> = Vec::with_capacity(vec.len());
    for pos in vec {
        cannon(pos);
        h.push(*pos);
    }
    h.sort();
    h.dedup();
    h
}
