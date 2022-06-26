#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub enum Player {X,O}

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub enum Square {
    Open,
    Taken{by : Player},
}

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub struct Pos{
    pub turn:Player,
    pub board:Board,
}

pub type Board = [Square;9];

pub type Eval = [bool;6];

pub struct EvalShowable(pub Eval);

pub struct CharGrid{
    pub height : usize,
    pub width : usize,
    pub strs : Vec<String>
}
