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
    size: Box<Fn() -> (u16, u16)>,
}

impl<'a, W: Write> Window<'a, W> {
    pub fn new<F>(buf_max: usize, size: F, term: &'a mut W) -> Self
    where
        F: Fn() -> (u16, u16) + 'static,
    {
        let mut this = Self {
            queue: Queue::with_size(buf_max),
            width: 0,
            height: 0,
            term,
            size: Box::new(size),
        };
        this.update_size();
        this
    }

    fn update_size(&mut self) -> bool {
        let (width, height) = {
            let (rows, cols) = (self.size)();
            (cols as _, rows as _)
        };
        if self.width == width && self.height == height {
            return false;
        }
        self.width = width;
        self.height = height;
        true
    }

    // TODO just redraw a delta
    pub fn check_size(&mut self) {
        if !self.update_size() {
            return;
        }

        // TODO cache this
        let left_pad = Paint::new(" ".repeat(NICK_MAX).into()).fg(Color::Unset);

        let height = self.height as usize;
        let right = (self.width as usize) - NICK_MAX - SEPARATOR.len();

        let mut region = 0;
        let mut buf = Vec::with_capacity(height);
        for element in &self.queue {
            let start = buf.len();
            buf.push((element, partition(element.message(), right)));
            region += buf.len() - start;
        }

        let diff = height.saturating_sub(std::cmp::max(region, height));

        let mut writer = Writer::new(&mut self.term);
        writer.clear_screen();

        for (msg, data) in &buf[diff..] {
            write_message(&mut writer, msg, &data, &left_pad);
        }
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
    }
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
        let _ = writeln!(writer, "{}{}{}", left, SEPARATOR, right);
    }
}
