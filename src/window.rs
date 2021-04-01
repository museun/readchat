use crate::App;

use super::{partition, queue::Queue, truncate};

use std::{borrow::Cow, io::Write};

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
const MIN_WINDOW_WIDTH: usize = 30;
// TODO make this configurable
const TS_COLOR: Color = Color::DarkYellow;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum UpdateMode {
    Redraw,
    Append,
    MarkAll,
    Info,
}

pub(crate) struct Window {
    queue: Queue<Message<'static>>,
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
        self.queue.push(Message::new(message));
    }

    pub(crate) fn update(&mut self, app: &mut App, update: UpdateMode) -> anyhow::Result<()> {
        let (width, height) = terminal::size()?;
        let mut stdout = std::io::stdout();

        if !matches!(
            app.view_mode,
            ViewMode::ForcedNormal | ViewMode::ForcedCompact
        ) {
            app.view_mode = if (width as usize) < self.min.unwrap_or(MIN_WINDOW_WIDTH) {
                ViewMode::Compact
            } else {
                ViewMode::Normal
            };
        }

        match update {
            UpdateMode::Redraw if self.queue.is_empty() => return Ok(()),

            UpdateMode::Redraw => {
                crossterm::execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
                for msg in self.queue.iter().rev().take(height as _).rev() {
                    let state = self.state(width, app.args.timestamps);
                    app.view_mode.print_message(&mut stdout, msg, state)?;
                }
            }

            UpdateMode::Append => {
                if let Some(msg) = self.queue.last() {
                    if self.queue.len() == 1 {
                        crossterm::execute!(stdout, MoveTo(0, 0))?;
                    }
                    let state = self.state(width, app.args.timestamps);
                    app.view_mode.print_message(&mut stdout, msg, state)?;
                }
            }

            UpdateMode::MarkAll
                if matches!(app.view_mode, ViewMode::Normal | ViewMode::ForcedNormal) =>
            {
                crossterm::queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

                let iter = self.queue.iter().rev().take((height) as _).rev();
                let mut ch = ALPHA.iter().take(iter.len()).rev();

                for msg in iter {
                    let mut state = self.state(width, app.args.timestamps);
                    // this'll stop printing deletion marks if we've reached the
                    // end of our alphabet
                    state.prefix = ch.next().copied();
                    app.view_mode.print_message(&mut stdout, msg, state)?;
                }
            }
            _ => {}
        }

        stdout.flush()?;
        Ok(())
    }

    pub(crate) fn delete(&mut self, ch: char, app: &mut App) -> anyhow::Result<()> {
        if let Some(p) = ALPHA.iter().position(|&c| c == ch) {
            let index = self.queue.len() - p - 1;
            self.queue.remove(index)
        }
        self.update(app, UpdateMode::Redraw)
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

    fn state(&self, width: u16, show_timestamp: bool) -> State<'_> {
        State {
            prefix: None,
            left: self.left,
            width: width as _,
            pad: &self.pad,
            indent: "",
            show_timestamp,
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
    show_timestamp: bool,
    pad: &'a str,
    indent: &'a str,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum ViewMode {
    Normal,
    Compact,
    ForcedNormal,
    ForcedCompact,
}

impl ViewMode {
    fn print_message(
        &self,
        stdout: &mut impl Write,
        msg: &Message<'_>,
        state: State<'_>,
    ) -> anyhow::Result<()> {
        let print = match self {
            Self::ForcedNormal | Self::Normal => Self::print_normal,
            Self::ForcedCompact | Self::Compact => Self::print_compact,
        };

        let RGB(r, g, b) = msg.pm.color().unwrap_or_default().rgb;
        print(stdout, msg, state, Color::Rgb { r, g, b })
    }

    fn print_normal(
        stdout: &mut impl Write,
        msg: &Message<'_>,
        state: State<'_>,
        color: Color,
    ) -> anyhow::Result<()> {
        let p = state.prefix.map(|_| 4).unwrap_or(0);

        let name = truncate::truncate_or_pad(msg.pm.name(), state.left - p);
        let name = style(name).with(color);

        let partition = partition::partition(
            msg.pm.data(),
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
        stdout: &mut impl Write,
        msg: &Message<'_>,
        state: State<'_>,
        color: Color,
    ) -> anyhow::Result<()> {
        const TS_FORMAT: usize = "HH:MM:SS".len();

        let name = msg.pm.name();
        let middle = state
            .show_timestamp
            .then(|| state.width - name.width() - TS_FORMAT)
            .unwrap_or_default();

        let name = (name.width() > state.width)
            .then(|| truncate::truncate_or_pad(name, state.width))
            .map(Cow::Owned)
            .unwrap_or_else(|| Cow::Borrowed(name));

        crossterm::queue!(
            stdout,
            Print("\n"),
            MoveToColumn(0),
            Print(&style(name).with(color))
        )?;

        if state.show_timestamp {
            let ts = style(msg.ts.format("%X").to_string()).with(TS_COLOR);
            crossterm::queue!(stdout, Print(" ".repeat(middle)), Print(ts))?;
        }

        crossterm::queue!(
            stdout,
            Print("\n"),
            MoveToColumn(0),
            Print(msg.pm.data()),
            Print("\n"),
            MoveToColumn(0)
        )?;

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Message<'msg> {
    pm: Privmsg<'msg>,
    ts: chrono::DateTime<chrono::Local>,
}

impl<'msg> Message<'msg> {
    fn new(pm: Privmsg<'msg>) -> Self {
        Self {
            pm,
            ts: chrono::Local::now(),
        }
    }
}
