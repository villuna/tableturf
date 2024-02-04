use crate::{
    board::{Board, Coord, TileData},
    Player,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Rotation {
    Up,
    Right,
    Down,
    Left,
}

impl Rotation {
    pub(crate) fn rotate_right(&self) -> Rotation {
        match self {
            Rotation::Up => Rotation::Right,
            Rotation::Right => Rotation::Down,
            Rotation::Down => Rotation::Left,
            Rotation::Left => Rotation::Up,
        }
    }
}

const ROTATIONS: [Rotation; 4] = [
    Rotation::Up,
    Rotation::Right,
    Rotation::Down,
    Rotation::Left,
];

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CardData {
    pub tiles: &'static [(Coord, bool)],
    pub special_cost: usize,
}

pub(crate) fn rotate_card(card: &CardData, rotation: Rotation) -> Vec<(Coord, bool)> {
    let transformation: fn(Coord) -> Coord = match rotation {
        Rotation::Up => |coord| coord,
        Rotation::Right => |Coord(x, y)| Coord(y, -x),
        Rotation::Down => |Coord(x, y)| Coord(-x, -y),
        Rotation::Left => |Coord(x, y)| Coord(-y, x),
    };

    card.tiles
        .iter()
        .cloned()
        .map(|(c, s)| (transformation(c), s))
        .collect::<Vec<_>>()
}

#[rustfmt::skip]
pub static HERO_SHOT: CardData = CardData {
    tiles: &[
        (Coord(-2, 1), false), (Coord(-1, 1), false), (Coord( 0, 1), false), (Coord( 1, 1), false), (Coord( 2, 1), false),
        (Coord(-2, 0), false), (Coord(-1, 0), false), (Coord( 0, 0), false), (Coord( 1, 0),  true), (Coord( 2, 0), false),
                          (Coord(-1,-1), false),
        (Coord(-2,-2), false),
    ],
    special_cost: 5,
};

pub(crate) fn is_placeable(
    board: &Board,
    player: Player,
    card: &CardData,
    rotation: Rotation,
    position: Coord,
    special: bool,
) -> bool {
    let card = rotate_card(card, rotation);

    let no_obstructions = card.iter().all(|(tile_pos, _)| {
        board
            .board
            .get(&(*tile_pos + position))
            .is_some_and(|t| *t == TileData::Empty || special && matches!(t, TileData::PlayerSquare { special: false, .. }))
    });

    let any_adjacent = card.iter().any(|(tile_pos, _)| {
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                if board
                    .board
                    .get(&(*tile_pos + position + Coord(dx, dy)))
                    .is_some_and(|t| matches!(t, TileData::PlayerSquare { player: p, .. } if player == *p))
                {
                    return true;
                }
            }
        }

        return false;
    });

    no_obstructions && any_adjacent
}

pub fn possible_card_placements(
    board: &Board,
    player: Player,
    card: &CardData,
    special_points: usize,
) -> Vec<(Coord, Rotation, bool)> {
    let mut moves = vec![];
    let can_special = card.special_cost <= special_points;

    for (pos, _) in board.board.iter() {
        for rotation in ROTATIONS {
            if can_special {
                if is_placeable(board, player, card, rotation, *pos, true) {
                    moves.push((*pos, rotation, true));
                } else {
                    // If it's not placeable as a special, it's not placeable as a normal
                    continue;
                }
            }

            if is_placeable(board, player, card, rotation, *pos, false) {
                moves.push((*pos, rotation, false));
            }
        }
    }

    moves
}

