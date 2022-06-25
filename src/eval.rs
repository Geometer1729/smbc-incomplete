#[path = "types.rs"]
mod types;

pub mod eval {

use crate::types::types::*;

// Eval meaning:
// can force win for x
// can force win for o
// can force draw
// can stop win for x
// can stop win for o
// can stop draw

pub fn combine (opts:Vec<Eval>) -> Eval {
    let mut eval=[false;6];
    for i in 0..3 {
        eval[i]=!opts.iter().all(|ent|ent[i+3]);
    }
    for i in 3..6 {
        eval[i]=!opts.iter().all(|ent|ent[i-3]);
    }
    return eval;

}

pub fn eval_pos(p:Pos) -> Option<Player> {
    let b = p.board;
    // TODO sorta repeditive
    if win_sets.iter().any(|ws| ws.iter().all(|&s| b[s] == Square::Taken{by:Player::X})) {
        return Some(Player::X);
    }
    if win_sets.iter().any(|ws| ws.iter().all(|&s| b[s] == Square::Taken{by:Player::O})) {
        return Some(Player::O);
    }
    return None;
}

const win_sets : [[usize;3];8]
    = [[0,1,2]
      ,[3,4,5]
      ,[6,7,8]
      ,[0,3,6]
      ,[1,4,7]
      ,[2,5,8]
      ,[0,4,8]
      ,[2,4,6]
      ];

pub const x_won : Eval
  = [true ,false,false
    ,false,true ,true
    ];

pub const draw : Eval
    = [false,true ,false
      ,true ,false,true
      ];

pub const o_won :Eval
    = [false,false,true
      ,true ,true ,false
      ];
}
