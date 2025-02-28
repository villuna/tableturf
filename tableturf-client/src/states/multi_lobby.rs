use raylib::prelude::*;
use tableturf::protocol::{ClientMessage, ServerMessage};

use crate::{client::GameContext, ui::Button, GameState, StateTransition};

#[derive(Copy, Clone, PartialEq, Eq)]
enum State {
    InLobby,
    WaitingForServer,
    Matchmaking,
}

pub struct MultiplayerLobby {
    back_button: Button,
    find_game_button: Button,

    state: State,
}

impl MultiplayerLobby {
    pub fn new(rl: &RaylibHandle) -> Self {
        Self {
            back_button: Button::new(rl, 20, 660, "<- Back", 30),
            find_game_button: Button::new(rl, 100, 100, "Find game", 50),
            state: State::InLobby,
        }
    }
}

impl GameState for MultiplayerLobby {
    fn update(&mut self, rl: &mut RaylibHandle, ctx: &mut GameContext) -> StateTransition {
        if matches!(self.state, State::InLobby) && self.find_game_button.is_clicked(rl) {
            if ctx.send(&ClientMessage::FindGame).is_err() {
                return StateTransition::Pop;
            }

            self.state = State::WaitingForServer;
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

        match self.state {
            State::WaitingForServer => d.draw_text("please wait...", 100, 100, 50, Color::RED),
            State::Matchmaking => d.draw_text("Matchmaking...", 100, 100, 50, Color::DARKBLUE),
            State::InLobby => self.find_game_button.draw(d),
        }
    }

    fn server_msg(&mut self, msg: ServerMessage, _rl: &mut RaylibHandle, _ctx: &mut GameContext) {
        match msg {
            ServerMessage::WaitForOpponent if matches!(self.state, State::WaitingForServer) => {
                self.state = State::Matchmaking;
            }
            ServerMessage::MatchFound { opp_info, .. }
                if matches!(self.state, State::Matchmaking) =>
            {
                println!("Found game with player: {opp_info:?}")
            }
            _ => panic!("Recieved unexpected message"),
        }
    }
}
