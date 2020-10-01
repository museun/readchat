use std::sync::Arc;

mod args;
mod queue;
mod term;
mod twitch;
mod util;
mod window;

fn main() -> anyhow::Result<()> {
    let args = args::Args::parse()?;
    let _screen = window::AltScreen::new();
    let ex = Arc::new(async_executor::Executor::new());

    let fut = term::main_loop(args, ex.clone());
    futures_lite::future::block_on(ex.run(fut))
}
