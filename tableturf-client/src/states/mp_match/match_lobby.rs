//! The lobby where players get to choose their deck and vote on a board

use raylib::{color::Color, prelude::{RaylibDraw, RaylibDrawHandle}, RaylibHandle};
use tableturf::protocol::{PublicPlayerInfo, ServerMessage};

use crate::{client::GameContext, states::error_message::ErrorMessage, GameState, StateTransition};

enum State {
    InLobby,
    OpponentDisconnected,
}

pub struct MatchLobby {
    state: State,
    player_info: PublicPlayerInfo,
    opp_info: PublicPlayerInfo,
}

impl MatchLobby {
    pub fn new(player_info: PublicPlayerInfo, opp_info: PublicPlayerInfo) -> Self {
        Self {
            state: State::InLobby,
            player_info,
            opp_info,
        }
    }
}

impl GameState for MatchLobby {
    fn update(&mut self, rl: &mut RaylibHandle, _ctx: &mut GameContext) -> StateTransition {
        match self.state {
            State::InLobby => StateTransition::None,
            State::OpponentDisconnected => StateTransition::Swap(Box::new(ErrorMessage::new(rl, "Communication error: opponent disconnected"))),
        }
    }
    fn draw(&mut self, d: &mut RaylibDrawHandle, _ctx: &mut GameContext) {
        d.clear_background(Color::RAYWHITE);
        d.draw_text(&format!("{} vs {}", self.player_info.name, self.opp_info.name), 100, 100, 50, Color::PURPLE); 
    }

    fn server_msg(&mut self, msg: ServerMessage, _rl: &mut RaylibHandle, _ctx: &mut GameContext) {
        match msg {
            ServerMessage::OpponentDisconnected => {
                self.state = State::OpponentDisconnected;
            }
            _ => {}
        }
    }
}
