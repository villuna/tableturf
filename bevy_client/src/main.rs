mod board;
mod cards;
mod utils;
mod game;
mod ai;

use bevy::{app::AppExit, prelude::*};
use board::{UpdateTiles, create_board, mouse_over_tile, update_tiles_event};
use cards::{create_hover, toggle_hover, place_card};
use game::{setup_game, opponent_turn, MoveMade, rotate};
use utils::cursor::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Player {
    P1,
    P2,
}

fn startup(mut cmd: Commands) {
    cmd.spawn((Camera2dBundle::default(), MainCamera));
}

fn exit_on_esc_system(input: Res<Input<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if input.pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}

fn main() {
    App::new()
        .add_event::<UpdateTiles>()
        .add_event::<MoveMade>()
        .add_plugins((DefaultPlugins, CursorTrackerPlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, exit_on_esc_system)
        .add_systems(Startup, (create_board, create_hover, setup_game))
        .add_systems(Update, (rotate, toggle_hover))
        .add_systems(Update, mouse_over_tile.after(toggle_hover).after(rotate))
        .add_systems(Update, place_card.after(mouse_over_tile))
        .add_systems(Update, opponent_turn.after(place_card))
        .add_systems(Update, update_tiles_event.after(opponent_turn))
        .run()
}
