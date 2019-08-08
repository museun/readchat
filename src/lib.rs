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

pub fn get_terminal_size() -> (u16, u16) {
    use terminal_size::{terminal_size, Height, Width};
    terminal_size()
        .map(|(Width(w), Height(h))| (h, w))
        .unwrap_or_else(|| (24, 80))
}

// TODO once cell with a dtor to unset these
#[cfg(target_os = "windows")]
pub fn enable_ansi() {
    let handle = winapi_util::HandleRef::stdout();
    let original = winapi_util::console::mode(&handle).unwrap();
    winapi_util::console::set_mode(handle, original | 0x0002 | 0x0004 | 0x0008).unwrap()
}

#[cfg(not(target_os = "windows"))]
pub fn enable_ansi() {
    assert!(clicolors_control::configure_terminal());
}
