use raylib::prelude::*;

use crate::{client::GameContext, ui::Button};

use super::{GameState, StateTransition};

pub struct ErrorMessage {
    error: String,
    button_exit: Button,
}

impl ErrorMessage {
    pub fn new(rl: &RaylibHandle, error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            button_exit: Button::new(rl, 20, 660, "<- Back", 30),
        } 
    }
}

impl GameState for ErrorMessage {
    fn draw(&mut self, d: &mut RaylibDrawHandle, _ctx: &mut GameContext) {
        d.clear_background(Color::BLACK);
        d.draw_text(&self.error, 100, 100, 50, Color::RED);
        self.button_exit.draw(d);
    } 

    fn update(&mut self, rl: &mut RaylibHandle, _ctx: &mut GameContext) -> super::StateTransition {
        if self.button_exit.is_clicked(rl) {
            StateTransition::Pop
        } else {
            StateTransition::None
        }
    }
}
