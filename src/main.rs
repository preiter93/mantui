use args::Args;
use clap::Parser;
use ui::App;

mod args;
mod core;
mod ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = Args::parse();

    App::run()?;
    Ok(())
}
