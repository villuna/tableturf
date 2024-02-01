mod board;
mod cursor;
mod cards;

use bevy::{prelude::*, app::AppExit};
use board::BoardPlugin;
use cursor::*;

pub enum Player { P1, P2 }

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
