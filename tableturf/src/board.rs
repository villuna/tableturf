use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
