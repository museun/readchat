use super::{partition, queue::Queue, truncate};

use std::io::Write as _;

use crossterm::{
    cursor::*,
    style::*,
    terminal::{self, *},
};
use twitchchat::{messages::Privmsg, twitch::color::RGB};

pub enum UpdateMode {
    Redraw,
    Append,
    MarkAll,
}

pub(crate) struct Window {
    queue: Queue<Privmsg<'static>>,
    left: usize,
    pad: String,
}

impl Window {
    pub(crate) fn new(left: usize, limit: usize) -> Self {
        Self {
            left,
            pad: " ".repeat(left),
            queue: Queue::with_size(limit),
        }
    }

    pub(crate) fn push(&mut self, message: Privmsg<'static>) {
        self.queue.push(message);
    }

    pub(crate) fn update(&mut self, update: UpdateMode) -> anyhow::Result<()> {
        let (width, height) = terminal::size()?;
        let mut stdout = std::io::stdout();

        match update {
            UpdateMode::Redraw if self.queue.is_empty() => return Ok(()),
            UpdateMode::Redraw => {
                crossterm::execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
                for msg in self.queue.iter().rev().take(height as _).rev() {
                    let state = self.state(width);
                    print_message(&mut stdout, msg, state)?;
                }
            }
            UpdateMode::Append => {
                if let Some(msg) = self.queue.last() {
                    if self.queue.len() == 1 {
                        crossterm::execute!(stdout, MoveTo(0, 0))?;
                    }
                    let state = self.state(width);
                    print_message(&mut stdout, msg, state)?;
                }
            }
            UpdateMode::MarkAll => {
                crossterm::queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

                let iter = self.queue.iter().rev().take((height) as _).rev();
                let mut ch = ALPHA.iter().take(iter.len()).rev();

                for msg in iter {
                    let mut state = self.state(width);
                    state.prefix.replace(*ch.next().unwrap());
                    print_message(&mut stdout, msg, state)?;
                }
            }
        }

        stdout.flush()?;
        Ok(())
    }

    pub(crate) fn delete(&mut self, ch: char) -> anyhow::Result<()> {
        if let Some(p) = ALPHA.iter().position(|&c| c == ch) {
            let index = self.queue.len() - p - 1;
            self.queue.remove(index)
        }
        self.update(UpdateMode::Redraw)
    }

    pub(crate) fn grow_nick_column(&mut self) -> anyhow::Result<bool> {
        if self.left == 25 {
            return Ok(false);
        }

        self.left += 1;
        self.pad = " ".repeat(self.left);
        Ok(true)
    }

    pub(crate) fn shrink_nick_column(&mut self) -> anyhow::Result<bool> {
        if self.left == 5 {
            return Ok(false);
        }

        self.left -= 1;
        self.pad = " ".repeat(self.left);
        Ok(true)
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

const ALPHA: &[char] = &[
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b',
    'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
    'v', 'w', 'x', 'y', 'z',
];

struct State<'a> {
    prefix: Option<char>,
    left: usize,
    width: usize,
    pad: &'a str,
    indent: &'a str,
}

fn print_message(
    stdout: &mut std::io::Stdout,
    msg: &Privmsg<'_>,
    state: State<'_>,
) -> anyhow::Result<()> {
    let RGB(r, g, b) = msg.color().unwrap_or_default().rgb;
    let color = Color::Rgb { r, g, b };

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
