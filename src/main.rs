use crate::game::state::State;
use clap::{App, Arg};
use ggez::conf::{FullscreenType, WindowMode};
use ggez::event;
use ggez::event::KeyMods;
use ggez::graphics;
use ggez::input::keyboard::{self, KeyCode};
use ggez::input::mouse::{self, MouseButton};
use ggez::{Context, GameResult};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::{Duration, Instant};

mod game;

pub const UPDATES_PER_SECOND: f32 = 16.0;
pub const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;

pub fn main() -> GameResult {
    let matches = App::new("Game of Life")
        .version("0.1")
        .author("Isak J. <ijagberg@gmail.com>")
        .arg(
            Arg::with_name("initial state")
                .short("i")
                .long("initial-state")
                .value_name("FILE")
                .help("Sets up the initial state of the world")
                .takes_value(true),
        )
        .get_matches();

    //dbg!(&matches);

    let initial_state_file = matches
        .value_of("initial state")
        .unwrap_or("resources/default.txt");

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

    let mut state = State::from(Path::new(initial_state_file));
    event::run(ctx, event_loop, &mut state)
}
