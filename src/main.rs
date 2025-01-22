use args::Args;
use clap::Parser;
use ui::{App, Theme, THEME};

mod args;
mod core;
mod ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    THEME.get_or_init(|| Theme::init(&args));

    App::run(args.command)?;
    Ok(())
}
