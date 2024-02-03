use crate::{
    ai::{Opponent, RandomMove},
    board::{Board, Coord, TileData, UpdateTiles, CursorCoord},
    cards::{rotate_card, CardData, Rotation, HERO_SHOT, SelectedCard, is_placeable},
    Player,
};
use bevy::{prelude::*, utils::HashMap};

static TEST_DECK: &[CardData] = &[HERO_SHOT; 15];

#[derive(Resource)]
pub(crate) struct PlayerRotation(pub(crate) Rotation);

#[derive(Event, Clone, Copy, Debug)]
pub(crate) struct MoveMade(pub Move);

#[derive(Copy, Clone, Debug)]
pub(crate) enum Move {
    Pass(usize),
    Play {
        card_id: usize,
        pos: Coord,
        rot: Rotation,
        special: bool,
    },
}

pub(crate) fn setup_game(mut cmd: Commands) {
    cmd.insert_resource(Opponent::new(RandomMove, TEST_DECK));
    cmd.insert_resource(PlayerRotation(Rotation::Up));
}

pub(crate) fn rotate(
    input: Res<Input<KeyCode>>,
    mut rotation: ResMut<PlayerRotation>,
    mut update_tiles: EventWriter<UpdateTiles>,
) {
    if input.just_pressed(KeyCode::R) {
        rotation.0 = rotation.0.rotate_right();
        update_tiles.send(UpdateTiles);
    }
}

pub(crate) fn make_move(
    mouse: Res<Input<MouseButton>>,
    kb: Res<Input<KeyCode>>,
    cursor_coord: Res<CursorCoord>,
    rotation: Res<PlayerRotation>,
    mut selected_card: ResMut<SelectedCard>,
    board: Res<Board>,
    mut play_move: EventWriter<MoveMade>,
) {
    if kb.just_pressed(KeyCode::P) {
        selected_card.0 = None;
        play_move.send(MoveMade(Move::Pass(0)));
    } else if let Some((card, coord)) = selected_card.0.as_mut().zip(cursor_coord.0) {
        if mouse.just_pressed(MouseButton::Left) {
            if is_placeable(&board, Player::P1, card, rotation.0, coord, false) {
                selected_card.0 = None;
                play_move.send(MoveMade(Move::Play {
                    card_id: 0,
                    pos: coord,
                    rot: rotation.0,
                    special: false,
                }));
            }
        }
    }
}

pub(crate) fn execute_turn(
    mut player_move_event: EventReader<MoveMade>,
    mut board: ResMut<Board>,
    mut opponent: ResMut<Opponent>,
    mut update_tiles: EventWriter<UpdateTiles>,
) {
    if let Some(MoveMade(player_move)) = player_move_event.read().cloned().next() {
        let opponent_move = opponent.make_move(&board);
        let mut temporary_board = HashMap::new();
        // Keep track of the power of the player's card *if they played one*
        // for use when calculating who has priority
        let mut player_card_power = None;

        match player_move {
            Move::Pass(_) => {},
            Move::Play { /*card_id,*/ pos, rot, .. } => {
                let card = rotate_card(&HERO_SHOT, rot);
                player_card_power = Some(card.len());

                for (tile_pos, special) in card {
                    temporary_board.insert(tile_pos + pos, TileData::PlayerSquare { player: Player::P1, special });
                }
            }
        }

        match opponent_move {
            Move::Pass(_) => {},
            Move::Play { card_id, pos, rot, .. } => {
                let card = rotate_card(&opponent.deck()[card_id], rot);

                let priority = if let Some(power) = player_card_power {
                    if power > card.len() {
                        Some(Player::P2)
                    } else if power < card.len() {
                        Some(Player::P1)
                    } else {
                        None
                    }
                } else {
                    Some(Player::P2)
                };

                for (tile_pos, special) in card {
                    let new_tile = if temporary_board.contains_key(&(tile_pos + pos)) {
                        match priority {
                            None => Some(TileData::Wall),
                            Some(Player::P2) => Some(TileData::PlayerSquare { player: Player::P2, special }),
                            _ => None,
                        }
                    } else {
                        Some(TileData::PlayerSquare { player: Player::P2, special })
                    };

                    temporary_board.extend(new_tile.map(|nt| (tile_pos + pos, nt)));
                }
            }
        }

        board.board.extend(temporary_board);
        update_tiles.send(UpdateTiles);
    }
}
