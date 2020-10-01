use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    let args = readchat::Args::parse()?;
    let _screen = readchat::AltScreen::enter();

    let ex = Arc::new(async_executor::Executor::new());
    let fut = readchat::window::main_loop(args, ex.clone());
    futures_lite::future::block_on(ex.run(fut))
}
