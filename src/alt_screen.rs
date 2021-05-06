use crossterm::{cursor::*, execute, terminal::*};

pub struct AltScreen;

impl AltScreen {
    pub fn enter() -> anyhow::Result<Self> {
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        enable_raw_mode()?;
        execute!(stdout, Clear(ClearType::All), Hide)?;
        Ok(Self {})
    }
}

impl Drop for AltScreen {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut out = std::io::stdout();
        let _ = execute!(out, LeaveAlternateScreen, Show);
    }
}
