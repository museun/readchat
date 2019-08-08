pub mod queue;
pub mod string;
pub mod window;
pub mod writer;

/// Timeout between checks for window resizing
pub const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(50);

/// Size of the backlog
pub const QUEUE_SIZE: usize = 128;

/// Max name length before truncation
pub const NICK_MAX: usize = 11;

/// Separator between the left and right columns
pub const SEPARATOR: &str = " | ";

pub use {
    queue::Queue,
    string::{partition, truncate},
    window::Window,
    writer::Writer,
};
