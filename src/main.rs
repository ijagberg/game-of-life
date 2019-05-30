mod grid;

use crate::grid::Cell;
use ggez::event;
use ggez::event::KeyMods;
use ggez::graphics;
use ggez::input::keyboard::KeyCode;
use ggez::input::mouse::MouseButton;
use ggez::{Context, GameResult};
use grid::Grid;
use std::env;
use std::path;
use std::time::{Duration, Instant};

const UPDATES_PER_SECOND: f32 = 8.0;
const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;

pub struct CameraPosition {
    x: f32,
    y: f32,
}

enum MouseMode {
    MovingCanvas,
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
    fn new() -> Self {
        MainState {
            grid: {
                let mut g = Grid::new();
                g.set_alive((0, 0));
                g.set_alive((1, 0));
                g.set_alive((2, 0));
                g.set_alive((2, -1));
                g.set_alive((1, -2));
                g
            },
            last_update: Instant::now(),
            zoom_level: 1.,
            camera_pos: CameraPosition::new(),
            is_paused: false,
            mouse_mode: MouseMode::None,
        }
    }

    pub fn get_cell_coords(&self, x: f32, y: f32) -> (isize, isize) {
        let cell_x =
            (x / ((30. + self.camera_pos.x) * self.zoom_level + self.camera_pos.x)) as isize;
        let cell_y = (y / ((30. + self.camera_pos.y) * self.zoom_level)) as isize;
        (cell_x, cell_y)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        match self.mouse_mode {
            MouseMode::MovingCanvas => {
                let d = ggez::input::mouse::delta(ctx);
                self.camera_pos.x += d.x;
                self.camera_pos.y += d.y;
            }
            MouseMode::Spawning => {
                let mouse_pos = ggez::input::mouse::position(ctx);
                let (target_cell_x, target_cell_y) = self.get_cell_coords(mouse_pos.x, mouse_pos.y);
                self.grid.set_alive((target_cell_x, target_cell_y));
            }
            MouseMode::Killing => {
                let mouse_pos = ggez::input::mouse::position(ctx);
                let (target_cell_x, target_cell_y) = self.get_cell_coords(mouse_pos.x, mouse_pos.y);
                self.grid.set_dead((target_cell_x, target_cell_y));
            }
            _ => (),
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

        graphics::present(ctx)
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        match button {
            MouseButton::Left if ggez::input::keyboard::is_key_pressed(ctx, KeyCode::LShift) => {
                dbg!(self.mouse_mode = MouseMode::MovingCanvas)
            }
            MouseButton::Left => {
                let (target_cell_x, target_cell_y) = self.get_cell_coords(x, y);
                dbg!(&(target_cell_x, target_cell_y));
                match self
                    .grid
                    .cells
                    .get(&(target_cell_x, target_cell_y))
                    .unwrap_or(&Cell::Dead)
                    {
                        Cell::Alive => dbg!(self.mouse_mode = MouseMode::Killing),
                        Cell::Dead => dbg!(self.mouse_mode = MouseMode::Spawning),
                    }
            }
            _ => (),
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
        match button {
            MouseButton::Left => dbg!(self.mouse_mode = MouseMode::None),
            _ => (),
        }
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
        // TODO: make this look nicer
        if y > 0. {
            self.zoom_level += 0.1;
            if self.zoom_level > 2. {
                self.zoom_level = 2.;
            }
        } else {
            self.zoom_level -= 0.1;
            if self.zoom_level < 0.5 {
                self.zoom_level = 0.5;
            }
        }
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Space => self.is_paused = !self.is_paused,
            KeyCode::LShift if ggez::input::mouse::button_pressed(ctx, MouseButton::Left) => {
                dbg!(self.mouse_mode = MouseMode::MovingCanvas)
            }
            _ => (),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::LShift => dbg!(self.mouse_mode = MouseMode::None),
            _ => (),
        }
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("Spacewalk", "ijagberg").add_resource_path(resource_dir);
    let (ctx, event_loop) = &mut cb.build()?;

    let state = &mut MainState::new();
    event::run(ctx, event_loop, state)
}
