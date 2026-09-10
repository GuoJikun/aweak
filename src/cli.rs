use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "aweak")]
#[command(about = "Keep your computer awake without modifying power settings")]
pub struct Cli {
    #[arg(long)]
    pub display_on: bool,

    #[arg(long)]
    pub time_limit: Option<u64>,

    #[arg(long)]
    pub expire_at: Option<String>,

    #[arg(long)]
    pub pid: Option<u32>,

    #[arg(long)]
    pub use_parent_pid: bool,

    #[arg(long)]
    pub use_pt_config: bool,
}