pub mod cannon {

use crate::types::types::*;

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
    for i in 0..4 { // hopefully this means 0..3
        v.push(rot(v[i]));
    }
    for i in 0..4 { // hopefully this means 0..3
        v.push(flip(v[i]));
    }
    return v;
}

fn board_cannon (b:Board) -> Board {
    match syms(b).iter().min() {
        Some(m) => {return *m;}
        None => {println!("syms was empty?");return b;}
    }
}

pub fn cannon (p: &mut Pos) {
    p.board = board_cannon(p.board);
}

}
