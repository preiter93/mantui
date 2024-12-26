#![allow(dead_code, unused)]
use std::error::Error;

use args::Args;
use clap::Parser;
use ui::App;

mod args;
mod core;
mod ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = Args::parse();
    if (args.command.is_some()) {
        core::test();
        return Ok(());
    }

    App::run()?;
    Ok(())
}
