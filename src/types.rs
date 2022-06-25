pub mod types {

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub enum Player {
    X,
    O,
}

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub enum Square {
    Open,
    Taken{by : Player},
}

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct Pos{
    pub turn:Player,
    pub board:Board,
}

pub type Board = [Square;9];

pub type Eval = [bool;6];

}
