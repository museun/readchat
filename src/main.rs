mod args;
mod queue;
mod term;
mod twitch;
mod util;
mod window;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = args::Args::parse()?;

    let _screen = window::AltScreen::new();
    term::main_loop(args).await?;

    Ok(())
}
