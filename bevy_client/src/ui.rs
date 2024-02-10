use crate::{board::TileData, game::GameState};
use bevy::prelude::*;

use crate::{AssetCache, Player};

#[derive(Component)]
pub(crate) struct TurnCounterText;

#[derive(Component)]
pub(crate) struct TileCounterText(pub Player);

#[derive(Component)]
pub(crate) struct GameOverText;

pub(crate) fn create_ui(mut cmd: Commands, assets: Res<AssetServer>) {
    let font = assets.load("FiraSans-Black.ttf");
    let style = TextStyle {
        font: font.clone(),
        font_size: 60.,
        color: Color::WHITE,
    };

    let small_style = TextStyle {
        font: font.clone(),
        font_size: 20.,
        color: Color::WHITE,
    };

    cmd.spawn((
        TurnCounterText,
        Text2dBundle {
            text: Text::from_section("0", style),
            transform: Transform::from_translation(Vec3::new(-250., 200., 0.)),
            ..default()
        },
    ))
    .with_children(|parent| {
        parent.spawn(Text2dBundle {
            text: Text::from_section("Turns Left", small_style),
            transform: Transform::from_translation(Vec3::new(0., 40., 0.)),
            ..default()
        });
    });

    cmd.spawn((
        TileCounterText(Player::P1),
        Text2dBundle {
            text: Text::from_section(
                "0",
                TextStyle {
                    font: font.clone(),
                    font_size: 60.,
                    color: TileData::PlayerSquare {
                        player: Player::P1,
                        special: false,
                    }
                    .colour(),
                },
            ),
            transform: Transform::from_translation(Vec3::new(-250., 0., 0.)),
            ..default()
        },
    ));

    cmd.spawn((
        TileCounterText(Player::P2),
        Text2dBundle {
            text: Text::from_section(
                "0",
                TextStyle {
                    font: font.clone(),
                    font_size: 60.,
                    color: TileData::PlayerSquare {
                        player: Player::P2,
                        special: false,
                    }
                    .colour(),
                },
            ),
            transform: Transform::from_translation(Vec3::new(-250., -80., 0.)),
            ..default()
        },
    ));

    cmd.spawn((
        GameOverText,
        Text2dBundle {
            text: Text::from_section(
                "GAME OVER!",
                TextStyle {
                    font: font.clone(),
                    font_size: 80.,
                    color: Color::rgb(1.0, 0.4, 0.3),
                }
            ),
            transform: Transform::from_translation(Vec3::new(0., 300., 1.)),
            visibility: Visibility::Hidden,
            ..default()
        }
    )).with_children(|parent| {
        parent.spawn(Text2dBundle {
            text: Text::from_section(
                "Press R to restart",
                TextStyle {
                    font: font.clone(),
                    font_size: 60.,
                    color: Color::rgb(0.9, 0.9, 0.9),
                }
            ),
            transform: Transform::from_translation(Vec3::new(0., -100., 1.)),
            ..default()
        });

        parent.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(0.05, 0.05, 0.05, 0.8),
                custom_size: Some(Vec2::new(500., 300.,)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0., -50., 0.)),
            ..default()
        });
    });
}

pub(crate) fn update_turn_text(
    assets: Res<AssetCache>,
    gs: Res<GameState>,
    mut text: Query<&mut Text, With<TurnCounterText>>,
) {
    let mut text = text.single_mut();
    let style = TextStyle {
        font: assets.number_font.clone(),
        font_size: 60.,
        color: Color::WHITE,
    };

    *text = Text::from_section(format!("{}", gs.turns_left), style);
}

pub(crate) fn update_tile_text(
    assets: Res<AssetCache>,
    gs: Res<GameState>,
    mut text: Query<(&mut Text, &TileCounterText)>,
) {
    for (mut text, &TileCounterText(player)) in text.iter_mut() {
        let count = gs
            .board
            .board
            .values()
            .filter(
                |tile| matches!(tile, TileData::PlayerSquare { player: p, .. } if *p == player),
            )
            .count();

        *text = Text::from_section(
            format!("{count}"),
            TextStyle {
                font: assets.number_font.clone(),
                font_size: 60.,
                color: TileData::PlayerSquare {
                    player,
                    special: false,
                }
                .colour(),
            },
        )
    }
}

pub(crate) fn show_game_over(
    mut text: Query<&mut Visibility, With<GameOverText>>,
    gs: Res<GameState>,
) {
    let mut visibility = text.single_mut();
    if gs.game_over {
        *visibility = Visibility::Visible
    } else {
        *visibility = Visibility::Hidden
    }
}
