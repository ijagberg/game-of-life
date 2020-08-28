use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
pub struct Settings {
    #[structopt(short, long)]
    pub debug: bool,
    #[structopt(long)]
    pub file: Option<String>,
    #[structopt(long, default_value = "16.0")]
    pub updates_per_second: f32,
    #[structopt(long, default_value = "1.0")]
    pub brush_size: f32,
    #[structopt(long)]
    pub render_active_cells: bool,
}

impl Settings {
    pub fn millis_per_update(&self) -> u64 {
        (1.0 / self.updates_per_second * 1_000.0) as u64
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            debug: false,
            file: None,
            updates_per_second: 16.0,
            brush_size: 1.0,
            render_active_cells: false,
        }
    }
}
