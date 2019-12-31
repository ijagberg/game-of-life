use crate::grid::{CellState, Grid};

use crate::grid::Coord;
use crate::Settings;
use ggez::event;
use ggez::event::KeyMods;
use ggez::graphics::{self, Rect};
use ggez::input::keyboard::{self, KeyCode};
use ggez::input::mouse::{self, MouseButton};
use ggez::{Context, GameResult};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::{Duration, Instant};

enum MouseMode {
    MovingCanvas(Coord<f32>),
    Spawning,
    Killing,
    None,
}

pub struct State {
    grid: Grid,
    last_update: Instant,
    zoom_level: f32,
    camera: Coord<f32>,
    is_paused: bool,
    mouse_mode: MouseMode,
    settings: Settings,
}

impl State {
    pub fn new(settings: Settings) -> Self {
        let grid = if let Some(file_name) = &settings.file {
            Grid::from_file(file_name.clone())
        } else {
            Grid::new()
        };
        Self {
            grid: grid,
            last_update: Instant::now(),
            zoom_level: 1.,
            camera: Coord::default(),
            is_paused: false,
            mouse_mode: MouseMode::None,
            settings,
        }
    }

    pub fn get_cell_coords(&self, coord: Coord<f32>) -> Coord<isize> {
        let (x, y) = (coord.x, coord.y);
        let negative_x_offset = if x + self.camera.x < 0. { -1 } else { 0 };
        let negative_y_offset = if y + self.camera.y < 0. { -1 } else { 0 };
        let cell_x = ((x + self.camera.x) / (30. * self.zoom_level)) as isize + negative_x_offset;
        let cell_y = ((y + self.camera.y) / (30. * self.zoom_level)) as isize + negative_y_offset;
        Coord::new(cell_x, cell_y)
    }

    fn update_grid(&mut self, _ctx: &mut Context) -> GameResult {
        let mut next_grid = Grid::new();
        for (&coord, state) in self.grid.cells() {
            let living_neighbors_count = self.grid.living_neighbors(coord).len();
            match state {
                CellState::Alive if living_neighbors_count == 2 || living_neighbors_count == 3 => {
                    // Set the cell to alive in the next state, and add all its neighbors
                    next_grid.set_alive(coord);
                }
                CellState::Dead if living_neighbors_count == 3 => {
                    // Check if this cell is born in the next state
                    next_grid.set_alive(coord);
                }
                _ => (),
            }
        }
        self.grid = next_grid;
        Ok(())
    }

    pub fn draw_grid(&mut self, ctx: &mut Context) -> GameResult {
        for (coord, state) in self.grid.cells() {
            if let CellState::Alive = state {
                let rectangle = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    Rect {
                        x: self.zoom_level * (coord.x * 30) as f32 - self.camera.x,
                        y: self.zoom_level * (coord.y * 30) as f32 - self.camera.y,
                        w: 30.0 * self.zoom_level,
                        h: 30.0 * self.zoom_level,
                    },
                    [0.3, 0.3, 0.0, 1.0].into(),
                )?;
                graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0., y: 0. },))?;
            }
        }

        Ok(())
    }
}

impl event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if Instant::now() - self.last_update >= Duration::from_millis(crate::MILLIS_PER_UPDATE) {
            self.update_grid(ctx)?;

            let mouse_pos = mouse::position(ctx);
            match self.mouse_mode {
                MouseMode::MovingCanvas(grab) => {
                    let (delta_x, delta_y) = (mouse_pos.x - grab.x, mouse_pos.y - grab.y);
                    self.camera.x -= delta_x;
                    self.camera.y -= delta_y;
                    self.mouse_mode = MouseMode::MovingCanvas(Coord::new(mouse_pos.x, mouse_pos.y));
                }
                MouseMode::Spawning => {
                    let target_coord = self.get_cell_coords(Coord::new(mouse_pos.x, mouse_pos.y));
                    self.grid
                        .set_alive(Coord::new(target_coord.x, target_coord.y));

                    if self.settings.debug {
                        println!("spawning at {:?}", target_coord);
                    }
                }
                MouseMode::Killing => {
                    let target_cell = self.get_cell_coords(Coord::new(mouse_pos.x, mouse_pos.y));
                    self.grid.set_dead(Coord::new(target_cell.x, target_cell.y));
                }
                MouseMode::None => (),
            }
            if self.is_paused && !keyboard::is_key_pressed(ctx, KeyCode::D) {
                return Ok(());
            }
            self.last_update = Instant::now();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        self.draw_grid(ctx)?;

        graphics::present(ctx)
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        match button {
            MouseButton::Left if keyboard::is_key_pressed(ctx, KeyCode::LShift) => {
                self.mouse_mode = MouseMode::MovingCanvas(Coord::new(x, y))
            }
            MouseButton::Left => {
                let target_cell = self.get_cell_coords(Coord::new(x, y));
                match self
                    .grid
                    .get(Coord::new(target_cell.x, target_cell.y))
                    .unwrap_or(&CellState::Dead)
                {
                    CellState::Alive => self.mouse_mode = MouseMode::Killing,
                    CellState::Dead => self.mouse_mode = MouseMode::Spawning,
                }
            }
            _ => (),
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
        if let MouseButton::Left = button {
            self.mouse_mode = MouseMode::None
        }
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
        self.zoom_level += if y > 0. { 0.05 } else { -0.05 };
        self.zoom_level = if self.zoom_level > 2. {
            2.
        } else if self.zoom_level < 0.05 {
            0.05
        } else {
            self.zoom_level
        };
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        if let KeyCode::Space = keycode {
            self.is_paused = !self.is_paused;

            if self.settings.debug {
                println!("paused = {}", self.is_paused);
            }
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        if let KeyCode::LShift = keycode {
            self.mouse_mode = MouseMode::None
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new(Settings {
            debug: false,
            file: None,
        })
    }
}
