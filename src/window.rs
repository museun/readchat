use std::borrow::Cow;
use std::io::Write;
use twitchchat::commands;
use yansi::{Color, Paint};

use crate::*;

pub struct Window<'a, W> {
    queue: Queue<commands::PrivMsg>,
    width: u16,
    height: u16,
    term: &'a mut W,
}

impl<'a, W: Write> Window<'a, W> {
    pub fn new(buf_max: usize, term: &'a mut W) -> Self {
        let mut this = Self {
            queue: Queue::with_size(buf_max),
            width: 0,
            height: 0,
            term,
        };
        this.update_size();
        this
    }

    // TODO need a special marker for lines at the 'top' that are continuations
    // but that are also single lines
    //
    // f | a
    //   | b
    //   | c
    // --
    // f | c <-- this should be different
    // XXX: how should this be different?
    pub fn check_size(&mut self) {
        if !self.update_size() {
            return;
        }

        // TODO cache this
        let left_pad = Paint::new(" ".repeat(NICK_MAX).into()).fg(Color::Unset);

        let height = self.height as usize;
        let right = (self.width as usize) - NICK_MAX - SEPARATOR.len();

        let mut buf = vec![];
        let mut budget = height as isize;
        for element in self.queue.iter().rev() {
            if budget == 0 {
                break;
            }

            let mut lines = partition(element.message(), right);
            let len = lines.len() as isize;
            if budget >= len {
                budget -= len;
                buf.push((element, lines));
                continue;
            }

            let rem = budget - len;
            let rem = if rem < 0 { len + rem } else { rem };
            lines.split_off(rem as usize);
            buf.push((element, lines));
            break;
        }

        let mut writer = Writer::new(&mut self.term);
        writer.clear_screen();

        for (msg, data) in buf.iter().rev() {
            write_message(&mut writer, msg, &data, &left_pad);
        }

        totally_a_viable_hide_cursor(&mut writer, self.width as _);
    }

    pub fn add(&mut self, msg: commands::PrivMsg) {
        // TODO cache this
        let left_pad = Paint::new(" ".repeat(NICK_MAX).into()).fg(Color::Unset);

        let right = (self.width as usize) - NICK_MAX - SEPARATOR.len();
        let data = partition(msg.message(), right);

        let mut writer = Writer::new(&mut self.term);
        writer.scroll(data.len());
        writer.goto((self.height as usize) - data.len(), 0);

        write_message(&mut writer, &msg, &data, &left_pad);
        self.queue.push(msg);

        totally_a_viable_hide_cursor(&mut writer, self.width as _);
    }

    fn update_size(&mut self) -> bool {
        let (width, height) = {
            let (rows, cols) = crate::get_terminal_size();
            (cols as _, rows as _)
        };
        if self.width == width && self.height == height {
            return false;
        }
        self.width = width;
        self.height = height;
        true
    }
}

// TODO until vscode supports the DEC escape sequence for hiding the cursor
// move it to the top right
fn totally_a_viable_hide_cursor<W: Write>(writer: &mut Writer<W>, width: usize) {
    writer.goto(0, width);
}

fn write_message<W: Write>(
    writer: &mut Writer<W>,
    msg: &twitchchat::commands::PrivMsg,
    data: &[String],
    left_pad: &Paint<Cow<'_, str>>,
) {
    let twitchchat::RGB(r, g, b) = msg.color().unwrap_or_default().rgb;
    let left = Paint::new(truncate(msg.user(), NICK_MAX)).fg(Color::RGB(r, g, b));
    let continuation = data.len() > 1;

    for (i, right) in data.iter().enumerate() {
        let left = if i == 0 || !continuation {
            format!("{: >max$}", left, max = NICK_MAX)
        } else {
            format!("{: >max$}", left_pad, max = NICK_MAX)
        };

        writer.clear_line();
        let _ = write!(writer, "{}{}{}", left, SEPARATOR, right);
        writer.carriage_return();
    }
}
