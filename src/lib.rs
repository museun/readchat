pub use args::Args;
pub mod window;

mod alt_screen;
pub use alt_screen::AltScreen;

mod app;
pub use app::App;

mod args;

pub mod colors;

mod debug;
mod queue;
mod twitch;

mod partition;
mod truncate;

mod keys;

mod logger;
pub use logger::Logger;

fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as _
}
