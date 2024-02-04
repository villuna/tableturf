use std::sync::Arc;

use crate::{
    ai::{Opponent, RandomMove},
    board::{Board, Coord, TileData, UpdateTiles, CursorCoord},
    cards::{rotate_card, CardData, Rotation, HERO_SHOT, is_placeable},
    Player, AssetCache,
};
use bevy::{prelude::*, utils::HashMap};
use rand::seq::IteratorRandom;

static TEST_DECK: &[CardData] = &[HERO_SHOT; 15];

pub(crate) struct ActorState {
    pub(crate) deck: Arc<[CardData]>,
    pub(crate) discard: Vec<usize>,
    pub(crate) hand: [usize; 4],
    pub(crate) special_points: usize,
}

impl ActorState {
    pub(crate) fn new<D: Into<Arc<[CardData]>>>(deck: D) -> Self {
        let mut rng = rand::thread_rng();
        let deck: Arc<[CardData]> = deck.into();
        assert_eq!(deck.len(), 15);
        let hand = (0..15).choose_multiple(&mut rng, 4);
        let hand = [hand[0], hand[1], hand[2], hand[3]];

        Self {
            deck,
            discard: Vec::new(),
            hand,
            special_points: 0,
        }
    }

    pub(crate) fn make_move(&mut self, move_made: &Move) {
        let mut rng = rand::thread_rng();

        let removed = match move_made {
            Move::Pass(card_id) => {
                self.special_points += 1;
                *card_id
            }

            Move::Play { card_id, special, .. } => {
                if *special {
                    let cost = self.deck[self.hand[*card_id]].special_cost;

                    if self.special_points >= cost {
                        self.special_points -= cost;
                    } else {
                        panic!("cant play special - not enough points");
                    }
                }
                *card_id
            }
        };

        self.discard.push(self.hand[removed]);

        let drawn = (0..15)
            .filter(|i| !self.discard.contains(i) && !self.hand.contains(i))
            .choose(&mut rng)
            .unwrap();

        self.hand[removed] = drawn;
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SelectedCard {
    pub card: usize,
    pub rotation: Rotation,
}

#[derive(Resource)]
pub(crate) struct PlayerState {
    pub(crate) state: ActorState,
    pub(crate) selected_card: Option<SelectedCard>,
}

pub fn toggle_selected_card(
    input: Res<Input<KeyCode>>,
    mut player_state: ResMut<PlayerState>,
    mut ew: EventWriter<UpdateTiles>,
) {
    let pressed = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4]
        .into_iter()
        .enumerate()
        .find_map(|(i, kc)| input.just_pressed(kc).then_some(i + 1));

    if let Some(num) = pressed {
        match player_state.selected_card {
            None => player_state.selected_card = Some(SelectedCard { 
                card: num,
                rotation: Rotation::Up,
            }),
            Some(_) => player_state.selected_card = None,
        }

        ew.send(UpdateTiles);
    }
}


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

pub(crate) fn setup_game(
    mut cmd: Commands,
    assets: Res<AssetServer>,
) {
    let player = PlayerState {
        state: ActorState::new(TEST_DECK),
        selected_card: None,
    };
    cmd.insert_resource(Opponent::new(RandomMove, TEST_DECK));

    let card_texture = assets.load("tableturf_card.png");
    let number_font = assets.load("FiraSans-Black.ttf");
    let number_style = TextStyle {
        font: number_font.clone(),
        font_size: 50.,
        color: Color::WHITE,
    };

    // Create card previews
    for (i, cid) in player.state.hand.iter().cloned().enumerate() {
        let card = player.state.deck[cid];

        let pos_y = if i <= 1 {
            130.
        } else {
            -130.
        };

        let pos_x = -600. + (i % 2) as f32 * 190.;

        // This shit sucks so bad
        // i stg if i was using godot id be unstoppable
        // ...
        // why am i doing this again?
        cmd.spawn(SpriteBundle {
            texture: card_texture.clone(),
            transform: Transform::from_translation(Vec3::new(pos_x, pos_y, 1.)).with_scale(Vec3::new(0.9, 0.9, 1.0)),
            ..default()
        }).with_children(|cb| {
            let tile_size = 15.;

            for i in -4..=5 {
                for j in -4..=5 {
                    let colour = if let Some((_, special)) = card.tiles.iter().find(|(pos, _)| *pos == Coord(i, j)) {
                        TileData::PlayerSquare { player: Player::P1, special: *special }.colour()
                    } else {
                        Color::rgba(0.1, 0.1, 0.1, 0.8)
                    };

                    let y_offset = 30.;
                    let x_offset = 0.;

                    let transform = Transform::from_translation(Vec3::new(
                        (i as f32 - 0.5) * (tile_size + 1.0) + x_offset,
                        j as f32 * (tile_size + 1.0) + y_offset,
                        1.5
                    ));
                    
                    cb.spawn(SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(tile_size, tile_size)),
                            color: colour,
                            ..default()
                        },
                        transform,
                        ..default()
                    });
                }
            }

            cb.spawn(Text2dBundle {
                text: Text::from_section(format!("{:?}", card.tiles.len()), number_style.clone()),
                transform: Transform::from_translation(Vec3::new(
                    -60.,
                    -90.,
                    2.,
                )),
                ..default()
            });

            let special_tile_size = 12.;
            for i in 0..card.special_cost {
                cb.spawn(SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(special_tile_size, special_tile_size)),
                        color: TileData::PlayerSquare { player: Player::P1, special: true }.colour(),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        -20. + (special_tile_size + 1.) * i as f32,
                        -90.,
                        2.,
                    )),
                    ..default()
                });
            }
        });
    }

    cmd.insert_resource(AssetCache { 
        card_sprite: card_texture,
        number_font,
    });
    cmd.insert_resource(player);
}

pub(crate) fn rotate(
    input: Res<Input<KeyCode>>,
    mut player_state: ResMut<PlayerState>,
    mut update_tiles: EventWriter<UpdateTiles>,
) {
    if input.just_pressed(KeyCode::R) {
        if let Some(rotation) = player_state.selected_card.as_mut().map(|s| &mut s.rotation) {
            *rotation = rotation.rotate_right();
            update_tiles.send(UpdateTiles);
        }
    }
}

pub(crate) fn make_move(
    mouse: Res<Input<MouseButton>>,
    kb: Res<Input<KeyCode>>,
    cursor_coord: Res<CursorCoord>,
    mut player_state: ResMut<PlayerState>,
    board: Res<Board>,
    mut play_move: EventWriter<MoveMade>,
) {
    if kb.just_pressed(KeyCode::P) {
        player_state.selected_card = None;
        play_move.send(MoveMade(Move::Pass(0)));
    } else if let Some((card, coord)) = player_state.selected_card.as_ref().zip(cursor_coord.0) {
        if mouse.just_pressed(MouseButton::Left) {
            let rotation = card.rotation;
            if is_placeable(&board, Player::P1, &player_state.state.deck[card.card], rotation, coord, false) {
                player_state.selected_card = None;
                play_move.send(MoveMade(Move::Play {
                    card_id: 0,
                    pos: coord,
                    rot: rotation,
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
    mut player: ResMut<PlayerState>,
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
            Move::Play { card_id, pos, rot, .. } => {
                let card = rotate_card(&player.state.deck[card_id], rot);
                player_card_power = Some(card.len());

                for (tile_pos, special) in card {
                    temporary_board.insert(tile_pos + pos, TileData::PlayerSquare { player: Player::P1, special });
                }
            }
        }

        player.state.make_move(&player_move);

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
