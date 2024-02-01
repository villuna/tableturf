use bevy::prelude::*;

use crate::Player;

#[derive(Resource)]
pub struct CurrentPlayer(pub Player);
