use raylib::prelude::RaylibDrawHandle;
use raylib::{prelude::RaylibDraw, RaylibHandle};
use raylib::color::Color;

use crate::client::GameContext;
use crate::ui::colours::DARKGRAY;
use crate::ui::Button;
use crate::{GameState, StateTransition};

use super::join_multi::JoinMultiplayer;

static TITLE: &str = "Tableturf.rs";
const TITLE_FONT_SIZE: i32 = 70;

pub struct MainMenu {
    button_single: Button,
    button_multi: Button,
    button_settings: Button,
    button_info: Button,
    button_exit: Button,
    title_x: i32,
}

impl MainMenu {
    pub fn new(rl: &RaylibHandle) -> Self {
        let offset = rl.measure_text(TITLE, TITLE_FONT_SIZE) / 2;
        let title_x = 1280 / 2 - offset;

        Self {
            button_single: Button::new(rl, title_x, 250, "Singleplayer", 50),
            button_multi: Button::new(rl, title_x, 330, "Multiplayer", 50),
            button_settings: Button::new(rl, title_x, 410, "Settings", 50),
            button_info: Button::new(rl, title_x, 490, "Info", 50),
            button_exit: Button::new(rl, title_x, 570, "Exit", 50),
            title_x,
        }
    }
}

impl GameState for MainMenu {
    fn update(&mut self, rl: &mut RaylibHandle, ctx: &mut GameContext) -> StateTransition {
        if self.button_exit.is_clicked(rl) {
            StateTransition::Exit
        } else if self.button_multi.is_clicked(rl) {
            StateTransition::Push(Box::new(JoinMultiplayer::new(rl, ctx)))
        } else {
            StateTransition::None
        }
    }

    fn draw(&mut self, d: &mut RaylibDrawHandle, _ctx: &mut GameContext) {
        d.clear_background(Color::RAYWHITE);
        d.draw_text(TITLE, self.title_x, 100, TITLE_FONT_SIZE, DARKGRAY);

        self.button_single.draw(d);
        self.button_multi.draw(d);
        self.button_settings.draw(d);
        self.button_info.draw(d);
        self.button_exit.draw(d);
    }
}
