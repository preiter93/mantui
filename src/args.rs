use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Show the man page of a command.
    #[arg(value_name = "COMMAND", help = "Show the man page of a command.")]
    pub(crate) command: Option<String>,
}
