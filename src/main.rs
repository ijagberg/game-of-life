mod grid;

use ggez::event;
use ggez::graphics;
use ggez::{Context, GameResult};
use grid::Grid;
use std::env;
use std::path;
use std::time::{Duration, Instant};

const UPDATES_PER_SECOND: f32 = 8.0;
const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;

struct MainState {
    grid: Grid,
    last_update: Instant,
    zoom_level: f32,
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
        }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE) {
            self.grid.update(ctx)?;
            self.last_update = Instant::now();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        self.grid.draw(ctx, self.zoom_level)?;

        graphics::present(ctx)
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
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
