use raylib::{color::Color, prelude::{RaylibDraw, RaylibDrawHandle}, RaylibHandle};
use tableturf::protocol::{ClientMessage, PublicPlayerInfo};

use crate::{client::GameContext, ui::Button, GameState, StateTransition};

pub struct JoinMultiplayer {
    button_exit: Button,
    button_hello: Button,
}

impl JoinMultiplayer {
    pub fn new(rl: &RaylibHandle, ctx: &mut GameContext) -> Self {
        ctx.connect("127.0.0.1:2611");

        Self {
            button_exit: Button::new(rl, 20, 660, "<- Back", 30),
            button_hello: Button::new(rl, 100, 300, "Say hello", 40),
        }
    }
}

impl GameState for JoinMultiplayer {
    fn update(&mut self, rl: &mut RaylibHandle, ctx: &mut GameContext) -> StateTransition {
        if self.button_hello.is_clicked(rl) {
            ctx.send(&ClientMessage::HelloServer { info: PublicPlayerInfo { name: "villuna".into() } }).unwrap();
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
        } else {
            d.draw_text("Not connected!!", 100, 100, 40, Color::RED);
        }

        self.button_exit.draw(d);
        self.button_hello.draw(d);
    }
}
