// #![allow(dead_code, unused)]

use args::Args;
use clap::Parser;
use ui::App;

mod args;
mod core;
mod ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.command.is_some() {
        return Ok(());
    }

    App::run()?;
    Ok(())
}
