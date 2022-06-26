use std::collections::HashMap;

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub enum Player {X,O}

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub enum Square {
    Open,
    Taken{by : Player},
}

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub enum Outcome {
    Draw,
    Win{with:Player}
}


#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub struct Pos{
    pub turn:Player,
    pub board:Board,
}


pub struct EvalShowable(pub Eval);

pub struct CharGrid{
    pub height : usize,
    pub width : usize,
    pub strs : Vec<String>
}

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub enum Pref {
    Winable,
    Drawable,
    Lost
}

pub type Board = [Square;9];
pub type Eval = [bool;6];
pub type Table = HashMap<Pos,Eval>;
pub type Objective = [Outcome;3];
