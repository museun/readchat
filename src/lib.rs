pub use args::Args;
pub mod window;

mod alt_screen;
pub use alt_screen::AltScreen;

mod app;
pub use app::main_loop;

mod args;
mod queue;
mod testing;
mod twitch;

mod partition;
mod truncate;

mod keys;
