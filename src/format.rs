use crate::types::{Player::*, Square::*, *};

use std::cmp::*;
use std::fmt;
use std::iter::*;

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            X => write!(f, "x"),
            O => write!(f, "o"),
        }
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Taken { by: p } => write!(f, "{}", p),
            Open => write!(f, " "),
        }
    }
}

impl fmt::Display for EvalShowable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}-{}{}{}",
            if self.0[0][0] { 'X' } else { 'x' },
            if self.0[0][1] { 'D' } else { 'd' },
            if self.0[0][2] { 'O' } else { 'o' },
            if self.0[1][0] { 'X' } else { 'x' },
            if self.0[1][1] { 'D' } else { 'd' },
            if self.0[1][2] { 'O' } else { 'o' },
        )
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#---#\n|{}{}{}|\n|{}{}{}|\n|{}{}{}|\n#---#\n{}'s turn ",
            self.board[0],
            self.board[1],
            self.board[2],
            self.board[3],
            self.board[4],
            self.board[5],
            self.board[6],
            self.board[7],
            self.board[8],
            self.turn
        )
    }
}

impl fmt::Display for CharGrid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl CharGrid {
    pub fn from_string(input: String) -> CharGrid {
        let mut strs: Vec<String> = input.lines().map(Into::into).collect();
        let height = strs.len();
        let width = strs.iter().map(|c| c.len()).max().unwrap_or(0);
        for ln in &mut strs {
            ln.extend(repeat(' ').take(width - ln.len()));
        }
        CharGrid {
            height,
            width,
            strs,
        }
    }

    fn join(l: CharGrid, r: CharGrid) -> CharGrid {
        let height = max(l.height, r.height);
        let width = l.width + r.width;
        let lstrs = l
            .strs
            .iter()
            .cloned()
            .chain(repeat(repeat(' ').take(l.width).collect()))
            .take(height);
        let rstrs = r
            .strs
            .iter()
            .cloned()
            .chain(repeat(repeat(' ').take(r.width).collect()))
            .take(height);
        let strs = zip(lstrs, rstrs).map(|(a, b)| a + &b).collect();
        CharGrid {
            height,
            width,
            strs,
        }
    }

    pub fn join_vec(vec: Vec<CharGrid>) -> CharGrid {
        vec.into_iter()
            .fold(Self::from_string("".into()), Self::join)
    }

    pub fn to_string(self: CharGrid) -> String {
        self.strs.join("\n")
    }
}

pub fn join_gridy(inputs: Vec<String>) -> String {
    CharGrid::join_vec(inputs.into_iter().map(CharGrid::from_string).collect()).to_string()
}
