use crate::game::state::State;
use ggez::conf::{FullscreenType, WindowMode};
use ggez::event;
use structopt::StructOpt;

use ggez::GameResult;

pub use settings::Settings;

mod game;
mod grid;
mod settings;

pub fn main() -> GameResult {
    let settings = Settings::from_args();

    let cb = ggez::ContextBuilder::new("Game of Life", "ijagberg");
    let (ctx, event_loop) = &mut cb
        .window_mode(WindowMode {
            width: 1200.,
            height: 600.,
            maximized: false,
            fullscreen_type: FullscreenType::Windowed,
            borderless: false,
            min_width: 100.,
            min_height: 100.,
            max_width: 0.0,
            max_height: 0.0,
            hidpi: false,
            resizable: true,
        })
        .build()?;

    let mut state = State::new(settings);
    event::run(ctx, event_loop, &mut state)
}
