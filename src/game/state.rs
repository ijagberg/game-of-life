use super::{CELL_HEIGHT, CELL_WIDTH, DEAD_CELL_COLOR, LIVING_CELL_COLOR};
use crate::grid::Coord;
use crate::grid::{CellState, Grid};
use crate::Settings;
use ggez::event;
use ggez::event::KeyMods;
use ggez::graphics::{self, Rect};
use ggez::input::keyboard::{self, KeyCode};
use ggez::input::mouse::{self, MouseButton};
use ggez::{Context, GameResult};

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
        let cell_x =
            ((x + self.camera.x) / (CELL_WIDTH * self.zoom_level)) as isize + negative_x_offset;
        let cell_y =
            ((y + self.camera.y) / (CELL_HEIGHT * self.zoom_level)) as isize + negative_y_offset;
        Coord::new(cell_x, cell_y)
    }

    fn update_grid(&mut self, _ctx: &mut Context) -> GameResult {
        if self.settings.debug {
            println!("updating grid...");
        }

        let mut next_grid = Grid::new();
        for (&coord, state) in self.grid.cells() {
            let living_neighbors_count = self.grid.living_neighbors(coord).len();

            if self.settings.debug {
                println!(
                    "cell at {:?} has {} living neighbors",
                    coord, living_neighbors_count
                );
            }

            match state {
                CellState::Alive if living_neighbors_count == 2 || living_neighbors_count == 3 => {
                    next_grid.set_alive(coord);
                }
                CellState::Dead if living_neighbors_count == 3 => {
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
                        x: self.zoom_level * (coord.x as f32 * CELL_WIDTH) - self.camera.x,
                        y: self.zoom_level * (coord.y as f32 * CELL_HEIGHT) - self.camera.y,
                        w: CELL_WIDTH * self.zoom_level,
                        h: CELL_HEIGHT * self.zoom_level,
                    },
                    LIVING_CELL_COLOR.into(),
                )?;
                graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0., y: 0. },))?;
            }
        }

        Ok(())
    }
}

impl event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if !self.is_paused
            && Instant::now() - self.last_update >= Duration::from_millis(crate::MILLIS_PER_UPDATE)
        {
            self.update_grid(ctx)?;
            self.last_update = Instant::now();
        }

        // read input even if game is paused
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
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, DEAD_CELL_COLOR.into());
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
        let absolute_position_before = {
            let mouse_pos = mouse::position(ctx);
            Coord::new(mouse_pos.x, mouse_pos.y)
        };
        let relative_mouse_position_before =
            relative_mouse_position(self, absolute_position_before);

        self.zoom_level += if y > 0. { 0.05 } else { -0.05 };
        if self.zoom_level > 2. {
            self.zoom_level = 2.;
        } else if self.zoom_level < 0.05 {
            self.zoom_level = 0.05;
        }

        self.camera.x =
            relative_mouse_position_before.x * self.zoom_level - absolute_position_before.x;
        self.camera.y =
            relative_mouse_position_before.y * self.zoom_level - absolute_position_before.y;
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

fn relative_mouse_position(state: &State, absolute_position: Coord<f32>) -> Coord<f32> {
    Coord::new(
        (state.camera.x + absolute_position.x) / state.zoom_level,
        (state.camera.y + absolute_position.y) / state.zoom_level,
    )
}
