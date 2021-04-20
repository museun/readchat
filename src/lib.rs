mod alt_screen;
mod app;
mod args;
mod logger;

pub mod colors;
pub mod window;

pub use alt_screen::AltScreen;
pub use app::App;
pub use args::Args;
pub use logger::Logger;

mod debug;
mod queue;
mod twitch;

mod partition;
mod truncate;

mod keys;

fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as _
}

fn array_iter<T, const N: usize>(
    array: [T; N],
) -> impl Iterator<Item = T> + ExactSizeIterator + Clone
where
    T: Clone,
{
    std::array::IntoIter::new(array)
}
