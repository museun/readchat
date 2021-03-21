use super::{partition, queue::Queue, truncate};

use std::{borrow::Cow, io::Write as _};

use crossterm::{
    cursor::*,
    style::*,
    terminal::{self, *},
};
use twitchchat::{messages::Privmsg, twitch::color::RGB};
use unicode_width::UnicodeWidthStr;

// TODO make this configurable
const MAX_COLUMN_WIDTH: usize = 25;
// TODO make this configurable
const MIN_COLUMN_WIDTH: usize = 5;
// TODO make this configurable
const MIN_WIDTH: usize = 30;

pub enum UpdateMode {
    Redraw,
    Append,
    MarkAll,
}

pub(crate) struct Window {
    queue: Queue<Privmsg<'static>>,
    left: usize,
    pad: String,
    min: Option<usize>,
}

impl Window {
    pub(crate) fn new(left: usize, limit: usize, min: Option<usize>) -> Self {
        Self {
            left,
            pad: " ".repeat(left),
            queue: Queue::with_size(limit),
            min,
        }
    }

    pub(crate) fn push(&mut self, message: Privmsg<'static>) {
        self.queue.push(message);
    }

    pub(crate) fn update(
        &mut self,
        update: UpdateMode,
        view_mode: &mut ViewMode,
    ) -> anyhow::Result<()> {
        let (width, height) = terminal::size()?;
        let mut stdout = std::io::stdout();

        *view_mode = if (width as usize) < self.min.unwrap_or(MIN_WIDTH) {
            ViewMode::Compact
        } else {
            ViewMode::Normal
        };

        match update {
            UpdateMode::Redraw if self.queue.is_empty() => return Ok(()),
            UpdateMode::Redraw => {
                crossterm::execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
                for msg in self.queue.iter().rev().take(height as _).rev() {
                    let state = self.state(width);
                    view_mode.print_message(&mut stdout, msg, state)?;
                }
            }
            UpdateMode::Append => {
                if let Some(msg) = self.queue.last() {
                    if self.queue.len() == 1 {
                        crossterm::execute!(stdout, MoveTo(0, 0))?;
                    }
                    let state = self.state(width);
                    view_mode.print_message(&mut stdout, msg, state)?;
                }
            }
            UpdateMode::MarkAll if matches!(view_mode, ViewMode::Normal) => {
                crossterm::queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

                let iter = self.queue.iter().rev().take((height) as _).rev();
                let mut ch = ALPHA.iter().take(iter.len()).rev();

                for msg in iter {
                    let mut state = self.state(width);
                    // this'll stop printing deletion marks if we've reached the
                    // end of our alphabet
                    state.prefix = ch.next().copied();
                    view_mode.print_message(&mut stdout, msg, state)?;
                }
            }
            _ => {}
        }

        stdout.flush()?;
        Ok(())
    }

    pub(crate) fn delete(&mut self, ch: char, view_mode: &mut ViewMode) -> anyhow::Result<()> {
        if let Some(p) = ALPHA.iter().position(|&c| c == ch) {
            let index = self.queue.len() - p - 1;
            self.queue.remove(index)
        }
        self.update(UpdateMode::Redraw, view_mode)
    }

    pub(crate) fn grow_nick_column(&mut self) -> bool {
        if self.left == MAX_COLUMN_WIDTH {
            return false;
        }

        self.left += 1;
        // TODO this could just truncate or append spaces instead of using an entirely new allocation
        self.pad = " ".repeat(self.left);
        true
    }

    pub(crate) fn shrink_nick_column(&mut self) -> bool {
        if self.left == MIN_COLUMN_WIDTH {
            return false;
        }

        self.left -= 1;
        // TODO this could just truncate or append spaces instead of using an entirely new allocation
        self.pad = " ".repeat(self.left);
        true
    }

    fn state(&self, width: u16) -> State<'_> {
        State {
            prefix: None,
            left: self.left,
            width: width as _,
            pad: &self.pad,
            indent: "",
        }
    }
}

#[rustfmt::skip]
// XXX: we cannot use a binary search on this because 'a' < 'A'
const ALPHA: &[char] = &[
    // uppercase first
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',

    // then numbers
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',

    // finally lowercase
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

struct State<'a> {
    prefix: Option<char>,
    left: usize,
    width: usize,
    pad: &'a str,
    indent: &'a str,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum ViewMode {
    Normal,
    Compact,
}

impl ViewMode {
    fn print_message(
        &self,
        stdout: &mut std::io::Stdout,
        msg: &Privmsg<'_>,
        state: State<'_>,
    ) -> anyhow::Result<()> {
        let RGB(r, g, b) = msg.color().unwrap_or_default().rgb;
        let color = Color::Rgb { r, g, b };

        match self {
            ViewMode::Normal => Self::print_normal(stdout, msg, state, color),
            ViewMode::Compact => Self::print_compact(stdout, msg, state, color),
        }
    }

    fn print_normal(
        stdout: &mut std::io::Stdout,
        msg: &Privmsg<'_>,
        state: State<'_>,
        color: Color,
    ) -> anyhow::Result<()> {
        let p = state.prefix.map(|_| 4).unwrap_or(0);

        let name = truncate::truncate_or_pad(msg.name(), state.left - p);
        let name = style(name).with(color);

        let partition = partition::partition(
            msg.data(),
            state.width - p - state.left - 1 - state.indent.len(),
        );

        for (i, part) in partition.into_iter().enumerate() {
            let first = i == 0;

            crossterm::queue!(stdout, Print("\n"), MoveToColumn(0))?;

            if let Some(prefix) = state.prefix {
                if first {
                    crossterm::queue!(
                        stdout,
                        Print("["),
                        Print(style(prefix).with(Color::Yellow)),
                        Print("] ")
                    )?;
                } else {
                    crossterm::queue!(stdout, Print("    "))?;
                }
            }

            if first {
                crossterm::queue!(stdout, Print(&name))?;
            } else {
                crossterm::queue!(
                    stdout,
                    Print(&state.pad[..state.pad.len() - p]),
                    Print(state.indent)
                )?;
            }
            crossterm::queue!(stdout, Print(" "), Print(part))?;
        }

        Ok(())
    }

    fn print_compact(
        stdout: &mut std::io::Stdout,
        msg: &Privmsg<'_>,
        state: State<'_>,
        color: Color,
    ) -> anyhow::Result<()> {
        let name = msg.name();
        let name = if name.width() > state.width {
            Cow::Owned(truncate::truncate_or_pad(name, state.width))
        } else {
            Cow::Borrowed(name)
        };

        let name = style(name).with(color);
        let partition = partition::partition(msg.data(), state.width);

        crossterm::queue!(stdout, Print("\n"), MoveToColumn(0), Print(&name))?;
        for part in partition {
            crossterm::queue!(stdout, Print("\n"), MoveToColumn(0), Print(part))?;
        }
        crossterm::queue!(stdout, Print("\n"), MoveToColumn(0))?;

        Ok(())
    }
}
