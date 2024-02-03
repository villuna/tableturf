use crate::{board::{Coord, Board, TileData, UpdateTiles}, cards::{Rotation, CardData, HERO_SHOT, rotate_card}, ai::{Opponent, RandomMove}, Player};
use bevy::{prelude::*, input::keyboard::KeyboardInput};

static TEST_DECK: &[CardData] = &[HERO_SHOT; 15];

#[derive(Resource)]
pub(crate) struct PlayerRotation(pub(crate) Rotation);

#[derive(Event)]
pub(crate) struct MoveMade;

#[derive(Copy, Clone, Debug)]
pub(crate) enum Move {
    Pass(usize),
    Play {
        card_id: usize,
        pos: Coord,
        rot: Rotation,
        #[allow(unused)]
        special: bool
    }
}

pub(crate) fn setup_game(mut cmd: Commands) {
    cmd.insert_resource(Opponent::new(RandomMove, TEST_DECK));
    cmd.insert_resource(PlayerRotation(Rotation::Up));
}

pub(crate) fn opponent_turn(
    mut er: EventReader<MoveMade>,
    mut board: ResMut<Board>,
    mut opponent: ResMut<Opponent>,
) {
    if !er.is_empty() {
        let next_move = opponent.make_move(&board);

        match next_move {
            Move::Pass(_) => {},

            Move::Play { card_id, pos, rot, .. } => {
                let cd = rotate_card(&opponent.deck()[card_id], rot);

                for (tile_pos, special) in cd {
                    board.board.insert(pos + tile_pos, Some(TileData { special, player: Player::P2 }));
                }
            }
        }
        
        er.clear();
    }
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
