use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
pub struct Settings {
    #[structopt(short, long)]
    pub debug: bool,
    #[structopt(long)]
    pub file: Option<String>,
    #[structopt(long, default_value = "16.0")]
    pub updates_per_second: f32,
}

impl Settings {
    pub fn millis_per_update(&self) -> u64 {
        (1.0 / self.updates_per_second * 1_000.0) as u64
    }
}
