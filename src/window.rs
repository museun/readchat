use std::borrow::Cow;
use std::io::Write;
use twitchchat::commands;
use yansi::{Color, Paint};

use crate::*;

/// The display window
pub struct Window {
    term: crossterm::Terminal,
    queue: Queue<commands::PrivMsg>,
    width: u16,
    height: u16,
    left_pad: Paint<Cow<'static, str>>,
}

impl Window {
    /// Create a new window with a buffer of 'buf_max' messages
    pub fn new(buf_max: usize) -> Self {
        let term = crossterm::terminal();
        let (width, height) = term.terminal_size();
        let left_pad = Paint::new(" ".repeat(NICK_MAX).into()).fg(Color::Unset);

        Self {
            term,
            queue: Queue::with_size(buf_max),
            width,
            height,
            left_pad,
        }
    }

    /// Check the window size, updating if its different and forcing a full redraw
    pub fn check_size(&mut self) {
        let (width, height) = self.term.terminal_size();
        if self.width == width && self.height == height {
            return;
        }

        self.width = width;
        self.height = height;

        let mut writer = BufferedWriter::stdout();
        writer.clear_screen();

        let right = (self.width as usize) - NICK_MAX - SEPARATOR.len();
        let mut budget = self.height;

        for msg in self.queue.view_last(self.height as _) {
            let data = partition(msg.message(), right);
            self.write_message(&mut writer, &msg, &data, &mut budget);
            if budget == 0 {
                break;
            }
        }
    }

    /// Add a message into the window
    pub fn add(&mut self, msg: commands::PrivMsg) {
        let mut writer = BufferedWriter::stdout();

        let right = (self.width as usize) - NICK_MAX - SEPARATOR.len();
        let data = partition(msg.message(), right);

        writer.scroll(data.len());
        writer.goto((self.height as usize) - (data.len() + 1), 0);

        let mut max = self.height; // just to be safe
        self.write_message(&mut writer, &msg, &data, &mut max);
        self.queue.push(msg);
    }

    fn write_message(
        &self,
        writer: &mut BufferedWriter,
        msg: &commands::PrivMsg,
        data: &[String],
        budget: &mut u16,
    ) {
        let twitchchat::RGB(r, g, b) = msg.color().unwrap_or_default().rgb;
        let nick = Paint::new(truncate(msg.user(), NICK_MAX)).fg(Color::RGB(r, g, b));

        let continuation = data.len() > 1;
        for (i, right) in data.into_iter().enumerate() {
            if *budget == 0 {
                return;
            }
            let left = if i == 0 || !continuation {
                &nick
            } else {
                &self.left_pad
            };
            writeln!(
                writer,
                "{: >max$}{}{}",
                left,
                SEPARATOR,
                right,
                max = NICK_MAX
            )
            .unwrap();
            *budget -= 1;
        }
    }
}
