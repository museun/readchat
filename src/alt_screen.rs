use crossterm::{
    cursor::*,
    terminal::{self, *},
};

use std::io::Write as _;

pub struct AltScreen;

impl AltScreen {
    pub fn enter() -> anyhow::Result<Self> {
        let mut stdout = std::io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;
        crossterm::execute!(stdout, Clear(ClearType::All), Hide)?;
        Ok(Self {})
    }
}

impl Drop for AltScreen {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let mut out = std::io::stdout();
        let _ = crossterm::execute!(out, LeaveAlternateScreen, Show);
    }
}
