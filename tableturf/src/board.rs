use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Rotation {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Tile {
    Empty,
    Wall,
    Tile { special: bool },
}

#[derive(Debug)]
pub struct Board {
    width: i32,
    height: i32,
    name: String,
    board: HashMap<Coord, Tile>,
}
