use raylib::{color::Color, ffi::KeyboardKey, prelude::{RaylibDraw, RaylibDrawHandle}, RaylibHandle};
use tableturf::protocol::{ClientMessage, PublicPlayerInfo};

use crate::{client::GameContext, ui::{Button, TextBox}, GameState, StateTransition};

use super::multi_lobby::MultiplayerLobby;

pub struct JoinMultiplayer {
    button_exit: Button,
    button_join: Button,
    name_box: TextBox,
}

impl JoinMultiplayer {
    pub fn new(rl: &RaylibHandle, ctx: &mut GameContext) -> Self {
        ctx.connect("127.0.0.1:2611");
        let name_box = TextBox::new(rl, 100 + rl.measure_text("Username: ", 50), 200, 10, 50);

        Self {
            button_exit: Button::new(rl, 20, 660, "<- Back", 30),
            button_join: Button::new(rl, (name_box.bounds().width + name_box.bounds().x + 30.) as _, 200, "Join", 30),
            name_box,
        }
    }
}

impl GameState for JoinMultiplayer {
    fn update(&mut self, rl: &mut RaylibHandle, ctx: &mut GameContext) -> StateTransition {
        self.name_box.update(rl);

        if self.button_join.is_clicked(rl) || rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            let name = self.name_box.take();

            if name.len() > 0 {
                ctx.send(&ClientMessage::HelloServer { info: PublicPlayerInfo { name } }).unwrap();
                return StateTransition::Swap(Box::new(MultiplayerLobby::new(rl)));
            }
        }

        if self.button_exit.is_clicked(rl) {
            ctx.disconnect();
            StateTransition::Pop
        } else {
            StateTransition::None
        }
    }

    fn draw(&mut self, d: &mut RaylibDrawHandle, ctx: &mut GameContext) {
        d.clear_background(Color::RAYWHITE);

        if ctx.connected() {
            d.draw_text("Connected!", 100, 100, 40, Color::BLUEVIOLET);
            d.draw_text("Username: ", 100, 200, 50, Color::BLACK);
            self.name_box.draw(d);
            self.button_join.draw(d);
        } else {
            d.draw_text("Not connected!!", 100, 100, 40, Color::RED);
        }

        self.button_exit.draw(d);
    }
}
