use crate::cards::*;
use crate::game::GameState;
use crate::utils::cursor::*;
use crate::Player;
use bevy::prelude::*;
use bevy::utils::HashMap;

const TILE_SIZE: f32 = 32.;
const BOARD_X_OFFSET: f32 = 200.;

// Wrapper for an unsigned integer 2d coordinate that I can use for the board
#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub struct Coord(pub i32, pub i32);

impl std::ops::Add for Coord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Coord(self.0 + rhs.0, self.1 + rhs.1)
    }
}

// Converts from a position in world space to the tile on the board that overlaps this position,
// if it is on any of the tiles.
fn world_to_board(dimensions: (i32, i32), position: Vec2) -> Option<Coord> {
    let w = TILE_SIZE + 1.;
    let hw = w / 2.0;

    let centred_x = ((position.x - 2.0 * BOARD_X_OFFSET + hw) / w).floor() as i32;
    let centred_y = ((position.y + hw) / w).floor() as i32;

    let offset_x = (dimensions.0 as f32 / 2.).floor() as i32;
    let offset_y = (dimensions.1 as f32 / 2.).floor() as i32;

    let x = centred_x + offset_x;
    let y = centred_y + offset_y;

    (x >= 0 && x < dimensions.0 && y >= 0 && y <= dimensions.1).then_some(Coord(x, y))
}

// Converts from the coordinate of a tile on the board to the position in screen space where that
// tile should be placed (i.e., the centre of the tile)
fn board_to_world(dimensions: (i32, i32), position: Coord) -> Vec2 {
    let base_x = -((dimensions.0 as f32 - 1.) * (TILE_SIZE + 1.)) / 2.;
    let base_y = -((dimensions.1 as f32 - 1.) * (TILE_SIZE + 1.)) / 2.;

    Vec2::new(
        base_x + BOARD_X_OFFSET + position.0 as f32 * (TILE_SIZE + 1.),
        base_y + position.1 as f32 * (TILE_SIZE + 1.),
    )
}

#[derive(Resource)]
pub struct CursorCoord(pub Option<Coord>);

pub fn mouse_over_tile(
    // for calculating board coordinates
    gs: Res<GameState>,
    cursor_pos: Res<CursorPosition>,
    // for checking if the cursor was moved
    cursor_moved: EventReader<CursorMoved>,
    // for writing the new cursor position if it was
    mut cursor_coord: ResMut<CursorCoord>,
    mut ew: EventWriter<UpdateTiles>,
) {
    if cursor_moved.is_empty() {
        return;
    }

    // At this point we know the cursor has moved and there is a hovering card
    let coord = world_to_board(gs.board.dimensions, cursor_pos.0);

    if coord != cursor_coord.0 {
        cursor_coord.0 = coord;
        ew.send(UpdateTiles);
    }
}

#[derive(Event)]
pub struct UpdateTiles;

pub fn update_tiles_event(
    mut er: EventReader<UpdateTiles>,
    mut tiles: Query<(&mut Sprite, &Tile)>,
    cursor_coord: Res<CursorCoord>,
    gs: ResMut<GameState>,
) {
    let card = gs.selected_card.as_ref();
    let coord = cursor_coord.0;

    if !er.is_empty() {
        // If there is a card selected and the cursor is on the board, we want to draw the overlay
        // If there is no card selected, we clear everything
        // If there *is* a card selected but the cursor is not on the board, we do nothing so that
        // any highlighted squares that *were* there stay there.
        if let Some((card, coord)) = card.zip(coord) {
            let card = rotate_card(&gs.player_state.deck[card.card], card.rotation);
            for (mut sprite, tile) in tiles.iter_mut() {
                if let Some(special) = card.iter().find_map(|(ctile, special)| {
                    (Coord(ctile.0 + coord.0, ctile.1 + coord.1) == tile.coord).then_some(*special)
                }) {
                    let colour = TileData::PlayerSquare {
                        player: Player::P1,
                        special,
                    }
                    .colour();
                    let base_colour = TileData::Empty.colour();
                    let blend = 0.4;

                    let blended = colour * [blend; 4] + base_colour * [(1. - blend); 4];
                    sprite.color = blended;
                } else {
                    let tile = gs.board.board.get(&tile.coord).unwrap();
                    sprite.color = tile.colour();
                }
            }
        } else if card == None {
            // Clear everything
            for (mut sprite, tile) in tiles.iter_mut() {
                let tile = gs.board.board.get(&tile.coord).unwrap();
                sprite.color = tile.colour();
            }
        }
        er.clear();
    }
}

#[derive(Component)]
pub struct Tile {
    coord: Coord,
}

pub fn create_board(cmd: &mut Commands, board: &Board) {
    for (&coord, tile) in board.board.iter() {
        let color = tile.colour();
        let pos =
            (board_to_world(board.dimensions, coord) + Vec2::new(BOARD_X_OFFSET, 0.)).extend(0.);

        cmd.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(TILE_SIZE + 2., TILE_SIZE + 2.)),
                ..default()
            },
            transform: Transform::from_translation(pos),
            ..default()
        });

        cmd.spawn(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(pos),
            ..default()
        })
        .insert(Tile { coord });
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TileData {
    Empty,
    Wall,
    PlayerSquare { player: Player, special: bool },
}

pub const EMPTY_COLOUR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const WALL_COLOUR: Color = Color::rgb(0.7, 0.7, 0.7);
pub const P1_COLOUR: Color = Color::rgb(0.7, 0.8, 0.2);
pub const P1_SPECIAL_COLOUR: Color = Color::rgb(0.8, 0.5, 0.2);
pub const P2_COLOUR: Color = Color::rgb(0.22, 0.29, 0.93);
pub const P2_SPECIAL_COLOUR: Color = Color::rgb(0.2, 0.9, 0.93);

impl TileData {
    pub fn colour(&self) -> Color {
        match self {
            TileData::Empty => EMPTY_COLOUR,

            TileData::Wall => WALL_COLOUR,

            TileData::PlayerSquare {
                player: Player::P1,
                special: false,
            } => P1_COLOUR,
            TileData::PlayerSquare {
                player: Player::P1,
                special: true,
            } => P1_SPECIAL_COLOUR,

            TileData::PlayerSquare {
                player: Player::P2,
                special: false,
            } => P2_COLOUR,
            TileData::PlayerSquare {
                player: Player::P2,
                special: true,
            } => P2_SPECIAL_COLOUR,
        }
    }
}

pub struct Board {
    pub board: HashMap<Coord, TileData>,
    pub dimensions: (i32, i32),
}

impl Board {
    pub(crate) fn new() -> Self {
        let mut board = HashMap::new();

        let dimensions = (17, 23);

        for x in 0..dimensions.0 {
            for y in 0..dimensions.1 {
                let tile = if x == 8 && y == 5 {
                    TileData::PlayerSquare {
                        special: true,
                        player: Player::P1,
                    }
                } else if x == 8 && y == dimensions.1 - 4 {
                    TileData::PlayerSquare {
                        special: true,
                        player: Player::P2,
                    }
                } else {
                    TileData::Empty
                };

                board.insert(Coord(x, y), tile);
            }
        }

        Self { board, dimensions }
    }
}
