mod board;
mod cards;
mod utils;
mod game;
mod ai;

use bevy::{app::AppExit, prelude::*};
use board::{UpdateTiles, create_board, mouse_over_tile, update_tiles_event};
use cards::{create_hover, toggle_hover};
use game::{setup_game, execute_turn, MoveMade, rotate, make_move};
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
        .add_systems(Update, make_move.after(mouse_over_tile))
        .add_systems(Update, execute_turn.after(make_move))
        .add_systems(Update, update_tiles_event.after(execute_turn))
        .run()
}
