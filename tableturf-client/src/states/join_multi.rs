use raylib::{color::Color, ffi::KeyboardKey, prelude::{RaylibDraw, RaylibDrawHandle}, RaylibHandle};
use tableturf::protocol::{ClientMessage, PublicPlayerInfo, ServerMessage};

use crate::{client::GameContext, ui::{Button, TextBox}, GameState, StateTransition};

use super::multi_lobby::MultiplayerLobby;

enum State {
    InLobby,
    WaitingForServer,
    Joined,
}

pub struct JoinMultiplayer {
    button_exit: Button,
    button_join: Button,
    name_box: TextBox,

    state: State,
}

impl JoinMultiplayer {
    pub fn new(rl: &RaylibHandle, ctx: &mut GameContext) -> Self {
        ctx.connect("127.0.0.1:2611");
        let name_box = TextBox::new(rl, 100 + rl.measure_text("Username: ", 50), 200, 10, 50);

        Self {
            button_exit: Button::new(rl, 20, 660, "<- Back", 30),
            button_join: Button::new(rl, (name_box.bounds().width + name_box.bounds().x + 30.) as _, 200, "Join", 30),
            name_box,
            state: State::InLobby,
        }
    }
}

impl GameState for JoinMultiplayer {
    fn update(&mut self, rl: &mut RaylibHandle, ctx: &mut GameContext) -> StateTransition {
        self.name_box.update(rl);

        match self.state {
            State::InLobby => {
                if self.button_join.is_clicked(rl) || rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    let name = self.name_box.take();

                    if name.len() > 0 {
                        ctx.send(&ClientMessage::HelloServer { info: PublicPlayerInfo { name } }).unwrap();
                        self.state = State::WaitingForServer;
                    }
                }
            }

            State::Joined => {
                return StateTransition::Swap(Box::new(MultiplayerLobby::new(rl)));
            }

            State::WaitingForServer => {}
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

        if !ctx.connected() {
            d.draw_text("Not connected!!", 100, 100, 40, Color::RED);
        } else {
            if matches!(self.state, State::WaitingForServer) {
                d.draw_text("Please wait...", 100, 100, 40, Color::DARKBLUE);
            } else {
                d.draw_text("Connected!", 100, 100, 40, Color::BLUEVIOLET);
                d.draw_text("Username: ", 100, 200, 50, Color::BLACK);
                self.name_box.draw(d);
                self.button_join.draw(d);
            }
        }

        self.button_exit.draw(d);
    }

    fn server_msg(&mut self, msg: ServerMessage, _rl: &mut RaylibHandle, _ctx: &mut GameContext) {
        match msg {
            ServerMessage::HelloClient if matches!(self.state, State::WaitingForServer) => {
                self.state = State::Joined;
            }
            _ => panic!("Unexpected server message"),
        }
    }
}
