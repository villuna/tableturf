use serde::{Deserialize, Serialize};

use crate::board::Tile;

type CardID = usize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    id: CardID,
    name: String,
    tiles: [[Tile; 8]; 8],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Deck {
    cards: [CardID; 15],
}
