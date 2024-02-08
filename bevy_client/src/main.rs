mod ai;
mod board;
mod cards;
mod game;
mod ui;
mod utils;

use bevy::{app::AppExit, prelude::*};
use board::{create_board, mouse_over_tile, update_tiles_event, UpdateTiles};
use game::{
    execute_turn, make_move, recreate_previews, rotate, setup_game, toggle_selected_card, MoveMade,
    RecreatePreviewsEvent,
};
use ui::{create_ui, update_tile_text, update_turn_text};
use utils::cursor::*;

// Is there a better way to do this?
#[derive(Resource)]
pub(crate) struct AssetCache {
    card_sprite: Handle<Image>,
    number_font: Handle<Font>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Player {
    P1,
    P2,
}

fn startup(mut cmd: Commands, mut window: Query<&mut Window>) {
    cmd.spawn((Camera2dBundle::default(), MainCamera));
    window.single_mut().set_maximized(true);
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
        .add_event::<RecreatePreviewsEvent>()
        .add_plugins((DefaultPlugins, CursorTrackerPlugin))
        .add_systems(Update, exit_on_esc_system)
        .add_systems(Startup, startup)
        .add_systems(Startup, (create_board, setup_game, create_ui))
        .add_systems(Update, (rotate, toggle_selected_card))
        .add_systems(
            Update,
            mouse_over_tile.after(toggle_selected_card).after(rotate),
        )
        .add_systems(Update, make_move.after(mouse_over_tile))
        .add_systems(Update, execute_turn.after(make_move))
        .add_systems(
            Update,
            (
                update_tiles_event,
                recreate_previews,
                update_turn_text,
                update_tile_text,
            )
                .after(execute_turn),
        )
        .run()
}
