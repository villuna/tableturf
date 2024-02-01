mod board;
mod cards;
mod utils;
mod game;

use bevy::{app::AppExit, prelude::*};
use board::BoardPlugin;
use utils::cursor::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Player {
    P1,
    P2,
}

impl Player {
    fn other_player(&self) -> Self {
        match self {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        }
    }
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
        .add_plugins((DefaultPlugins, CursorTrackerPlugin, BoardPlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, exit_on_esc_system)
        .run()
}
