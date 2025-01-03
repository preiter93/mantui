use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Show the man page of a command.
    #[arg(short, long)]
    pub(crate) command: Option<String>,
}
