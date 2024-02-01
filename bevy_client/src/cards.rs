use crate::{
    board::{Board, Coord, CursorCoord, TileData, UpdateTiles},
    Player, game::CurrentPlayer,
};
use bevy::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CardData {
    pub tiles: &'static [(Coord, bool)],
}

#[rustfmt::skip]
pub static HERO_SHOT: CardData = CardData {
    tiles: &[
        ((-2, 1), false), ((-1, 1), false), (( 0, 1), false), (( 1, 1), false), (( 2, 1), false),
        ((-2, 0), false), ((-1, 0), false), (( 0, 0), false), (( 1, 0),  true), (( 2, 0), false),
                          ((-1,-1), false),
        ((-2,-2), false),
    ],
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

fn is_placeable(player: Player, card: &CardData, position: Coord, board: &Board) -> bool {
    let no_obstructions = card.tiles.iter().all(|(tile_pos, _)| {
        board
            .board
            .get(&(tile_pos.0 + position.0, tile_pos.1 + position.1))
            .is_some_and(|t| t.is_none())
    });

    let any_adjacent = card.tiles.iter().any(|(tile_pos, _)| {
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                if board.board.get(&(tile_pos.0 + position.0 + dx, tile_pos.1 + position.1 + dy))
                    .is_some_and(|t| t.as_ref().is_some_and(|t| t.player == player)) {
                    return true;
                }
            }
        }

        return false;
    });

    no_obstructions && any_adjacent
}

pub fn place_card(
    input: Res<Input<MouseButton>>,
    cursor_coord: Res<CursorCoord>,
    mut current_player: ResMut<CurrentPlayer>,
    mut selected_card: ResMut<SelectedCard>,
    mut board: ResMut<Board>,
    mut ew: EventWriter<UpdateTiles>,
) {
    if let Some((card, coord)) = selected_card.0.as_mut().zip(cursor_coord.0) {
        if input.just_pressed(MouseButton::Left) {
            if is_placeable(current_player.0, card, coord, &board) {
                for (tile_pos, special) in card.tiles {
                    *board
                        .board
                        .get_mut(&(tile_pos.0 + coord.0, tile_pos.1 + coord.1))
                        .unwrap() = Some(TileData {
                        special: *special,
                        player: current_player.0,
                    });
                }

                selected_card.0 = None;
                current_player.0 = current_player.0.other_player();
                ew.send(UpdateTiles);
            }
        }
    }
}
