mod args;
mod queue;
mod term;
mod twitch;
mod util;
mod window;

fn main() -> anyhow::Result<()> {
    let args = args::Args::parse()?;
    let _screen = window::AltScreen::new();
    let fut = term::main_loop(args);
    async_executor::Executor::new().run(fut)
}
