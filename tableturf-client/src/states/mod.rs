mod menu;
mod join_multi;
mod multi_lobby;
mod mp_match;
mod error_message;

pub use menu::MainMenu;
use raylib::prelude::*;
use tableturf::protocol::ServerMessage;

use crate::client::GameContext;

pub enum StateTransition {
    None,
    Pop,
    Push(Box<dyn GameState>),
    Swap(Box<dyn GameState>),
    Exit,
}

pub trait GameState {
    fn update(&mut self, _rl: &mut RaylibHandle, _ctx: &mut GameContext) -> StateTransition {
        StateTransition::None
    }
    
    fn draw(&mut self, d: &mut RaylibDrawHandle, _ctx: &mut GameContext) {
        d.clear_background(Color::RAYWHITE);
    }

    fn server_msg(&mut self, _msg: ServerMessage, _rl: &mut RaylibHandle, _ctx: &mut GameContext) {}
}
