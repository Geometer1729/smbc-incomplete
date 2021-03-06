use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Player {
    X,
    O,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Square {
    Open,
    Taken { by: Player },
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Outcome {
    Draw,
    Win { with: Player },
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Pos {
    pub turn: Player,
    pub board: Board,
}

pub struct EvalShowable(pub Eval);

pub struct CharGrid {
    pub height: usize,
    pub width: usize,
    pub strs: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Pref {
    Lost,
    Drawable,
    Winable,
}

pub type Board = [Square; 9];
pub type Eval = [[bool; 3];2];
pub type Objective = [Outcome; 3];
pub type Table = Arc<dashmap::DashMap<Pos, Eval>>;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum OutMsg{
    Upd(Pos),
    ReqPong(Vec<u8>),
}

#[derive(Clone,  PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum InMsg{
    MoveAt(usize),
    Reset ,
}
