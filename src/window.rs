use crossterm::terminal;
use std::io::Write;

use super::{queue::Queue, *};
use twitchchat::messages::Privmsg;

pub enum UpdateMode {
    Redraw,
    Append,
}

pub struct Window {
    queue: Queue<twitch::Message>,
    left: usize,
    pad: String,
}

impl Window {
    pub fn new(left: usize, limit: usize) -> Self {
        Self {
            left,
            pad: " ".repeat(left),
            queue: Queue::with_size(limit),
        }
    }

    pub fn push(&mut self, message: Privmsg<'static>) {
        let message = twitch::Message {
            color: util::normalize_color(message.color().unwrap_or_default().rgb, 35.0),
            nick: message.name,
            data: message.data,
        };

        self.queue.push(message);
    }

    pub fn update(&mut self, update: UpdateMode) -> anyhow::Result<()> {
        use crossterm::style::{style, Color, Print};

        fn print_message(
            stdout: &mut std::io::Stdout,
            msg: &twitch::Message,
            left: usize,
            w: usize,
            pad: &str,
        ) -> anyhow::Result<()> {
            let twitchchat::color::RGB(r, g, b) = msg.color;
            let style = style(util::truncate_or_pad(&msg.nick, left)).with(Color::Rgb { r, g, b });
            crossterm::queue!(stdout, Print(style))?;

            for (i, part) in util::partition(&msg.data, w - left - 1)
                .into_iter()
                .enumerate()
            {
                if i > 0 {
                    crossterm::queue!(stdout, Print(pad))?;
                }
                crossterm::queue!(stdout, Print(" "))?;
                crossterm::queue!(stdout, Print(part))?;
                crossterm::queue!(stdout, Print("\n"))?;
            }
            crossterm::queue!(stdout, crossterm::cursor::Hide)?;
            Ok(())
        }

        let (w, _h) = terminal::size()?;
        let mut stdout = std::io::stdout();

        match update {
            UpdateMode::Redraw => {
                if self.queue.is_empty() {
                    return Ok(());
                }

                crossterm::execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
                for msg in self.queue.iter() {
                    print_message(&mut stdout, msg, self.left, w as _, &self.pad)?;
                }
            }
            UpdateMode::Append => {
                let msg = self.queue.iter().rev().next().unwrap();
                print_message(&mut stdout, msg, self.left, w as _, &self.pad)?;
            }
        }

        stdout.flush().map_err(Into::into)
    }
}

pub struct AltScreen {}

impl AltScreen {
    pub fn new() -> anyhow::Result<Self> {
        let mut stdout = std::io::stdout();
        crossterm::execute!(stdout, terminal::EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;
        crossterm::execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
        crossterm::execute!(stdout, crossterm::cursor::Hide)?;
        Ok(Self {})
    }
}

impl Drop for AltScreen {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = crossterm::execute!(std::io::stdout(), terminal::LeaveAlternateScreen);
    }
}
