use crate::types::{*,Player::*,Square::*};
use std::fmt;

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            X => write!(f,"x"),
            O => write!(f,"o")
        }
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Taken{by:p} => write!(f,"{}",p),
            Open => write!(f," ")
        }
    }
}

impl fmt::Display for EvalShowable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}{}{}-{}{}{}"
              ,if self.0[0] {'X'}else{'x'}
              ,if self.0[1] {'D'}else{'d'}
              ,if self.0[2] {'O'}else{'o'}
              ,if self.0[3] {'X'}else{'x'}
              ,if self.0[4] {'D'}else{'d'}
              ,if self.0[5] {'O'}else{'o'}
        )
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!
          (f
          ,"#---#\n|{}{}{}|\n|{}{}{}|\n|{}{}{}|\n#---#\n{}'s turn "
          ,self.board[0]
          ,self.board[1]
          ,self.board[2]
          ,self.board[3]
          ,self.board[4]
          ,self.board[5]
          ,self.board[6]
          ,self.board[7]
          ,self.board[8]
          ,self.turn
          )
    }
}
