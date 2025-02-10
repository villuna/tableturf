use client::GameContext;
use color_eyre::eyre::OptionExt;
use raylib::prelude::*;
use states::MainMenu;
use tableturf::protocol::ServerMessage;

mod client;
mod states;
mod ui;

enum StateTransition {
    None,
    Pop,
    Push(Box<dyn GameState>),
    Swap(Box<dyn GameState>),
    Exit,
}

trait GameState {
    fn update(&mut self, _rl: &mut RaylibHandle, _ctx: &mut GameContext) -> StateTransition {
        StateTransition::None
    }
    
    fn draw(&mut self, d: &mut RaylibDrawHandle, _ctx: &mut GameContext) {
        d.clear_background(Color::RAYWHITE);
    }

    fn server_msg(&mut self, _msg: ServerMessage, _rl: &mut RaylibHandle, _ctx: &mut GameContext) {}
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let (mut rl, thread) = raylib::init().size(1280, 720).title("Tableturf").build();

    rl.set_target_fps(60);

    let mut states: Vec<Box<dyn GameState>> = vec![Box::new(MainMenu::new(&rl))];
    let mut ctx = GameContext::new();

    while !rl.window_should_close() {
        let state = states.last_mut().ok_or_eyre("No states on the stack")?;
        let mut d = rl.begin_drawing(&thread);

        state.draw(&mut d, &mut ctx);

        d.draw_fps(10, 10);
        drop(d);

        while let Some(msg) = ctx.recv() {
            state.server_msg(msg, &mut rl, &mut ctx);
        }

        let transition = state.update(&mut rl, &mut ctx);

        match transition {
            StateTransition::Exit => {
                break;
            }

            StateTransition::Pop => {
                if states.len() == 1 {
                    break;
                } else {
                    states.pop();
                }
            }

            StateTransition::Push(s) => states.push(s),

            StateTransition::Swap(s) => {
                states.pop();
                states.push(s);
            }

            StateTransition::None => {}
        }
    }

    Ok(())
}
