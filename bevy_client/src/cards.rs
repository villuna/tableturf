use crate::{
    board::{Board, Coord, CursorCoord, TileData, UpdateTiles},
    Player, game::MoveMade,
};
use bevy::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Rotation {
    Up,
    Right,
    Down,
    Left,
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

#[derive(Resource)]
pub struct SelectedCard(pub Option<CardData>);

pub fn create_hover(mut cmd: Commands) {
    cmd.insert_resource(CursorCoord(None));
    cmd.insert_resource(SelectedCard(None));
}

pub fn toggle_hover(
    input: Res<Input<MouseButton>>,
    mut ew: EventWriter<UpdateTiles>,
    mut card: ResMut<SelectedCard>,
) {
    if input.just_pressed(MouseButton::Right) {
        match card.0 {
            None => card.0 = Some(HERO_SHOT),
            Some(_) => card.0 = None,
        }

        ew.send(UpdateTiles);
    }
}

fn is_placeable(
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
            .is_some_and(|t| t.is_none() || special && t.as_ref().is_some_and(|td| !td.special))
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
                    .is_some_and(|t| t.as_ref().is_some_and(|t| t.player == player))
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

pub fn place_card(
    input: Res<Input<MouseButton>>,
    cursor_coord: Res<CursorCoord>,
    mut selected_card: ResMut<SelectedCard>,
    mut board: ResMut<Board>,
    mut update_tiles: EventWriter<UpdateTiles>,
    mut play_move: EventWriter<MoveMade>,
) {
    if let Some((card, coord)) = selected_card.0.as_mut().zip(cursor_coord.0) {
        if input.just_pressed(MouseButton::Left) {
            if is_placeable(&board, Player::P1, card, Rotation::Up, coord, false) {
                for (tile_pos, special) in card.tiles {
                    *board
                        .board
                        .get_mut(&Coord(tile_pos.0 + coord.0, tile_pos.1 + coord.1))
                        .unwrap() = Some(TileData {
                        special: *special,
                        player: Player::P1,
                    });
                }

                selected_card.0 = None;
                update_tiles.send(UpdateTiles);
                play_move.send(MoveMade);
            }
        }
    }
}
