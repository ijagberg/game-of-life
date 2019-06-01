mod grid;

use crate::grid::Cell;
use clap::{App, Arg};
use ggez::event;
use ggez::event::KeyMods;
use ggez::graphics;
use ggez::graphics::Rect;
use ggez::input::keyboard::{self, KeyCode};
use ggez::input::mouse::{self, MouseButton};
use ggez::{Context, GameResult};
use grid::Grid;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::{Duration, Instant};

const UPDATES_PER_SECOND: f32 = 16.0;
const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;

#[derive(Default)]
pub struct CameraPosition {
    x: f32,
    y: f32,
}

enum MouseMode {
    MovingCanvas(f32, f32),
    Spawning,
    Killing,
    None,
}

impl CameraPosition {
    pub fn new() -> Self {
        CameraPosition { x: 0., y: 0. }
    }
}

struct MainState {
    grid: Grid,
    last_update: Instant,
    zoom_level: f32,
    camera_pos: CameraPosition,
    is_paused: bool,
    mouse_mode: MouseMode,
}

impl MainState {
    pub fn new() -> Self {
        MainState {
            grid: Grid::new(),
            last_update: Instant::now(),
            zoom_level: 1.,
            camera_pos: CameraPosition::new(),
            is_paused: false,
            mouse_mode: MouseMode::None,
        }
    }

    pub fn from(file_name: &str) -> Self {
        let mut grid = Grid::new();
        if let Ok(f) = File::open(file_name) {
            let f = BufReader::new(f);

            f.lines()
                .map(std::result::Result::unwrap)
                .enumerate()
                .for_each(|(row, line)| {
                    line.chars().enumerate().for_each(|(col, c)| match c {
                        'O' => grid.set_alive((col as isize, row as isize)),
                        _ => (),
                    })
                });

            MainState {
                grid,
                last_update: Instant::now(),
                zoom_level: 1.,
                camera_pos: CameraPosition::new(),
                is_paused: false,
                mouse_mode: MouseMode::None,
            }
        } else {
            MainState::new()
        }
    }

    pub fn get_cell_coords(&self, x: f32, y: f32) -> (isize, isize) {
        let cell_x = ((x + self.camera_pos.x) / (30. * self.zoom_level)) as isize;
        let cell_y = ((y + self.camera_pos.y) / (30. * self.zoom_level)) as isize;
        //dbg!(&(self.camera_pos.x, self.camera_pos.y, x, y, cell_x, cell_y));
        (cell_x, cell_y)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        match self.mouse_mode {
            MouseMode::MovingCanvas(grab_x, grab_y) => {
                let mouse_pos = mouse::position(ctx);
                let (delta_x, delta_y) = (mouse_pos.x - grab_x, mouse_pos.y - grab_y);
                self.camera_pos.x -= delta_x;
                self.camera_pos.y -= delta_y;
                self.mouse_mode = MouseMode::MovingCanvas(mouse_pos.x, mouse_pos.y);
            }
            MouseMode::Spawning => {
                let mouse_pos = mouse::position(ctx);
                let (target_cell_x, target_cell_y) = self.get_cell_coords(mouse_pos.x, mouse_pos.y);
                self.grid.set_alive((target_cell_x, target_cell_y));
            }
            MouseMode::Killing => {
                let mouse_pos = mouse::position(ctx);
                let (target_cell_x, target_cell_y) = self.get_cell_coords(mouse_pos.x, mouse_pos.y);
                self.grid.set_dead((target_cell_x, target_cell_y));
            }
            MouseMode::None => (),
        }
        if self.is_paused {
            return Ok(());
        }
        if Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE) {
            self.grid.update(ctx)?;
            self.last_update = Instant::now();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        self.grid.draw(ctx, self.zoom_level, &self.camera_pos)?;

        // Draw highlighted cell
        let mouse_pos = mouse::position(ctx);
        let (x, y) = self.get_cell_coords(mouse_pos.x, mouse_pos.y);
        let highlight = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(5. * self.zoom_level),
            Rect {
                x: self.zoom_level * (x * 30) as f32 - self.camera_pos.x,
                y: self.zoom_level * (y * 30) as f32 - self.camera_pos.y,
                w: 30.0 * self.zoom_level,
                h: 30.0 * self.zoom_level,
            },
            graphics::BLACK,
        )?;
        graphics::draw(ctx, &highlight, (ggez::mint::Point2 { x: 0., y: 0. },))?;

        graphics::present(ctx)
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        match button {
            MouseButton::Left if keyboard::is_key_pressed(ctx, KeyCode::LShift) => {
                self.mouse_mode = MouseMode::MovingCanvas(x, y)
            }
            MouseButton::Left => {
                let (target_cell_x, target_cell_y) = self.get_cell_coords(x, y);
                match self
                    .grid
                    .cells
                    .get(&(target_cell_x, target_cell_y))
                    .unwrap_or(&Cell::Dead)
                {
                    Cell::Alive => self.mouse_mode = MouseMode::Killing,
                    Cell::Dead => self.mouse_mode = MouseMode::Spawning,
                }
            }
            _ => (),
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
        match button {
            MouseButton::Left => self.mouse_mode = MouseMode::None,
            _ => (),
        }
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
        // TODO: make this look nicer
        if y > 0. {
            self.zoom_level += 0.05;
            if self.zoom_level > 2. {
                self.zoom_level = 2.;
            }
        } else {
            self.zoom_level -= 0.05;
            if self.zoom_level < 0.05 {
                self.zoom_level = 0.05;
            }
        }
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Space => self.is_paused = !self.is_paused,
            _ => (),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::LShift => self.mouse_mode = MouseMode::None,
            _ => (),
        }
    }
}

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

    dbg!(&matches);

    let initial_state_file = matches
        .value_of("initial state")
        .unwrap_or("resources/default.txt");

    let cb = ggez::ContextBuilder::new("Game of Life", "ijagberg");
    let (ctx, event_loop) = &mut cb.build()?;

    let state = &mut MainState::from(initial_state_file);
    event::run(ctx, event_loop, state)
}
