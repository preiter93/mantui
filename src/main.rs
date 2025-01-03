use args::Args;
use clap::Parser;
use ui::App;

mod args;
mod core;
mod ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    App::run(args.command)?;
    Ok(())
}
