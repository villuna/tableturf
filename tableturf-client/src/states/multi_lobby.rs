use raylib::prelude::*;
use tableturf::protocol::ClientMessage;

use crate::{client::GameContext, ui::Button, GameState, StateTransition};

pub struct MultiplayerLobby {
    back_button: Button,
    find_game_button: Button,
}

impl MultiplayerLobby {
    pub fn new(rl: &RaylibHandle) -> Self {
        Self {
            back_button: Button::new(rl, 20, 660, "<- Back", 30),
            find_game_button: Button::new(rl, 100, 100, "Find game", 50),
        }
    }
}

impl GameState for MultiplayerLobby {
    fn update(&mut self, rl: &mut RaylibHandle, ctx: &mut GameContext) -> StateTransition {
        if self.find_game_button.is_clicked(rl) {
            if ctx.send(&ClientMessage::FindGame).is_err() {
                return StateTransition::Pop;
            }
        }

        if self.back_button.is_clicked(rl) {
            ctx.disconnect();
            StateTransition::Pop
        } else {
            StateTransition::None
        }
    }

    fn draw(&mut self, d: &mut RaylibDrawHandle, _ctx: &mut GameContext) {
        d.clear_background(Color::RAYWHITE);
        self.back_button.draw(d);
        self.find_game_button.draw(d);
    }
}
