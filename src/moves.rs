use crate::cannon::*;
use crate::types::*;
use crate::types::Player::*;
use crate::types::Square::*;
use std::collections::HashSet;

pub fn moves(p:Pos) -> Vec<Pos> {
    let mut moves : HashSet<Pos> = HashSet::with_capacity(9);
    for i in 0..9 {
        if p.board[i] == Open {
            let Pos{turn,board} = p;
            let mut board2 = board.clone();
            board2[i] = Taken{by:turn};
            let mut new_position = Pos{turn:other(turn),board:board2};
            cannon(&mut new_position);
            moves.insert(new_position);
        }
    }
    moves.into_iter().collect()
}

pub const start : Pos = Pos
    {turn:X
    ,board: [Open;9]
    };

fn other(s:Player) -> Player {
    match s {
        X => O,
        O => X,
    }
}
