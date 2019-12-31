use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
pub struct Settings {
    #[structopt(short, long)]
    pub debug: bool,
    #[structopt(long)]
    pub file: Option<String>,
}
