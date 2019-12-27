use crate::game::grid::{Cell, Grid};
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

#[derive(Default)]
pub struct CameraPosition {
    pub x: f32,
    pub y: f32,
}

enum MouseMode {
    MovingCanvas(f32, f32),
    Spawning,
    Killing,
    None,
}

impl CameraPosition {
    pub fn new() -> Self {
        Self { x: 0., y: 0. }
    }
}

pub struct State {
    grid: Grid,
    last_update: Instant,
    zoom_level: f32,
    camera_pos: CameraPosition,
    is_paused: bool,
    mouse_mode: MouseMode,
}

impl State {
    pub fn new() -> Self {
        Self {
            grid: Grid::new(),
            last_update: Instant::now(),
            zoom_level: 1.,
            camera_pos: CameraPosition::new(),
            is_paused: false,
            mouse_mode: MouseMode::None,
        }
    }

    pub fn from(file_name: &Path) -> Self {
        match file_name.extension().and_then(OsStr::to_str) {
            Some("txt") => Self::from_txt(file_name).unwrap_or_default(),
            Some("rle") => Self::from_rle(file_name).unwrap_or_default(),
            _ => Self::new(),
        }
    }

    fn from_txt(file_name: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut grid = Grid::new();
        let f = File::open(file_name)?;
        let f = BufReader::new(f);
        f.lines()
            .filter_map(|l| l.ok())
            .enumerate()
            .for_each(|(row, line)| {
                line.chars().enumerate().for_each(|(col, c)| {
                    if let 'O' = c {
                        grid.set_alive((col as isize, row as isize))
                    }
                })
            });
        Ok(State {
            grid,
            last_update: Instant::now(),
            zoom_level: 1.,
            camera_pos: CameraPosition::new(),
            is_paused: false,
            mouse_mode: MouseMode::None,
        })
    }

    fn from_rle(file_name: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        use regex::Regex;

        let mut grid = Grid::new();
        let f = File::open(file_name)?;
        let f = BufReader::new(f);

        let mut lines = f
            .lines()
            .filter_map(|l| l.ok())
            .skip_while(|line| line.starts_with('#'));

        let size_line = lines.next().ok_or("invalid file format")?;
        let (_x, _y): (usize, usize) = {
            let re = Regex::new(r"x\s*=\s*(?P<x>\d+)\s*,\s*y\s*=\s*(?P<y>\d+)")?;
            let captures = re.captures(&size_line).ok_or("could not capture")?;
            (
                captures
                    .get(1)
                    .ok_or("did not capture x")?
                    .as_str()
                    .parse()?,
                captures
                    .get(2)
                    .ok_or("did not capture y")?
                    .as_str()
                    .parse()?,
            )
        };

        // Concatenate remaining lines into one long string
        let content = lines.fold("".into(), |prev, line| format!("{}{}", prev, line));

        let content_lines = content.split('$').collect::<Vec<_>>();

        let tag_regex = Regex::new(r"(\d*)([b|o])")?;
        for (row_idx, content_line) in content_lines.iter().enumerate() {
            let mut col = 0;
            for capture in tag_regex.captures_iter(content_line) {
                let (repetitions, cell_state) = {
                    (
                        capture[1].parse::<usize>().unwrap_or(1),
                        match capture[2].into() {
                            "b" => Cell::Dead,
                            "o" => Cell::Alive,
                            _ => {
                                return Err(regex::Error::Syntax(
                                    "could not capture cell state".into(),
                                )
                                .into())
                            }
                        },
                    )
                };
                if let Cell::Alive = cell_state {
                    for col_idx in col..col + repetitions {
                        grid.set_alive((col_idx as isize, row_idx as isize));
                    }
                }
                col += repetitions;
            }
        }

        Ok(State {
            grid,
            last_update: Instant::now(),
            zoom_level: 1.,
            camera_pos: CameraPosition::new(),
            is_paused: false,
            mouse_mode: MouseMode::None,
        })
    }

    pub fn get_cell_coords(&self, x: f32, y: f32) -> (isize, isize) {
        let negative_x_offset = if x + self.camera_pos.x < 0. { -1 } else { 0 };
        let negative_y_offset = if y + self.camera_pos.y < 0. { -1 } else { 0 };
        let cell_x =
            ((x + self.camera_pos.x) / (30. * self.zoom_level)) as isize + negative_x_offset;
        let cell_y =
            ((y + self.camera_pos.y) / (30. * self.zoom_level)) as isize + negative_y_offset;
        //dbg!(&(self.camera_pos.x, self.camera_pos.y, x, y, cell_x, cell_y));
        (cell_x, cell_y)
    }
}

impl event::EventHandler for State {
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
        if self.is_paused && !keyboard::is_key_pressed(ctx, KeyCode::D) {
            return Ok(());
        }
        if Instant::now() - self.last_update >= Duration::from_millis(crate::MILLIS_PER_UPDATE) {
            self.grid.update(ctx)?;
            self.last_update = Instant::now();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        self.grid.draw(ctx, self.zoom_level, &self.camera_pos)?;

        // Debug drawing
        // {
        // // Draw highlighted cell
        // let mouse_pos = mouse::position(ctx);
        // let (x, y) = self.get_cell_coords(mouse_pos.x, mouse_pos.y);
        // let highlight = graphics::Mesh::new_rectangle(
        // ctx,
        // graphics::DrawMode::stroke(5. * self.zoom_level),
        // Rect {
        // x: self.zoom_level * (x * 30) as f32 - self.camera_pos.x,
        // y: self.zoom_level * (y * 30) as f32 - self.camera_pos.y,
        // w: 30.0 * self.zoom_level,
        // h: 30.0 * self.zoom_level,
        // },
        // graphics::BLACK,
        // )?;
        // graphics::draw(ctx, &highlight, (ggez::mint::Point2 { x: 0., y: 0. },))?;

        // if let Ok(origo_to_camera) = graphics::Mesh::new_line(
        // ctx,
        // &vec![
        // ggez::mint::Point2 {
        // x: 0. - self.camera_pos.x,
        // y: 0. - self.camera_pos.y,
        // },
        // ggez::mint::Point2 { x: 0., y: 0. },
        // ],
        // 10.,
        // graphics::BLACK,
        // ) {
        // graphics::draw(
        // ctx,
        // &origo_to_camera,
        // (ggez::mint::Point2 { x: 0., y: 0. },),
        // )?;
        // }
        // let mouse_pos = ggez::input::mouse::position(ctx);
        // if let Ok(origo_to_mouse) = graphics::Mesh::new_line(
        // ctx,
        // &vec![
        // ggez::mint::Point2 {
        // x: 0. - self.camera_pos.x,
        // y: 0. - self.camera_pos.y,
        // },
        // ggez::mint::Point2 {
        // x: mouse_pos.x,
        // y: mouse_pos.y,
        // },
        // ],
        // 10.,
        // graphics::BLACK,
        // ) {
        // graphics::draw(ctx, &origo_to_mouse, (ggez::mint::Point2 { x: 0., y: 0. },))?;
        // }
        // }

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

        let mouse_pos_before = {
            let pos = ggez::input::mouse::position(ctx);
            (pos.x - self.camera_pos.x, pos.y - self.camera_pos.y)
        };
        let camera_mouse_vector = (
            self.camera_pos.x - mouse_pos_before.0,
            self.camera_pos.y - mouse_pos_before.1,
        );
        let mouse_pos_after = (
            mouse_pos_before.0 * self.zoom_level,
            mouse_pos_before.1 * self.zoom_level,
        );
        let camera_pos_after = CameraPosition {
            x: mouse_pos_after.0 + camera_mouse_vector.0,
            y: mouse_pos_after.1 + camera_mouse_vector.1,
        };
        self.camera_pos = camera_pos_after;
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        if let KeyCode::Space = keycode {
            self.is_paused = !self.is_paused
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
        Self::new()
    }
}
