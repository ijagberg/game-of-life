mod grid;

use ggez::event;
use ggez::graphics;
use ggez::{Context, GameResult};
use grid::Grid;
use std::env;
use std::path;
use std::time::Duration;

struct MainState {
    grid: Grid,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState {
            grid: {
                let mut g = Grid::new();
                g.set_alive((0, 0));
                g.set_alive((1, 0));
                g.set_alive((2, 0));
                g.set_alive((2, -1));
                g.set_alive((1, -2));
                g
            },
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        println!("{:?}", self.grid);
        self.grid.update(ctx)?;

        ggez::timer::sleep(Duration::from_secs(1));
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        self.grid.draw(ctx)?;

        Ok(())
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

    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
