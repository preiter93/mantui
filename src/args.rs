use clap::{Parser, arg};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The command whose man page to view.
    #[arg(short, long)]
    pub command: Option<String>,
}
