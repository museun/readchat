#![allow(dead_code)]
use std::borrow::Cow;
use std::collections::VecDeque;
use std::io::Write;

use crossbeam_channel as channel;
use twitchchat::{commands, Event, Message as TwitchMessage};
use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;
use yansi::{Color, Paint};

const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(50);
const QUEUE_SIZE: usize = 128;

/// Split a string into parts of 'max' size
fn partition(input: &str, max: usize) -> Vec<String> {
    let mut vec = vec![];

    let mut budget = max;
    let mut temp = String::with_capacity(max);

    for mut word in input.split_word_bounds() {
        if temp.is_empty() && word.chars().all(char::is_whitespace) {
            continue;
        }

        let width = word.width();
        if width < budget {
            budget -= width;
            temp.push_str(word);
            continue;
        }

        if !temp.is_empty() {
            vec.push(std::mem::replace(&mut temp, String::with_capacity(max)));
            budget = max;
        }

        loop {
            if word.width() <= budget {
                if !temp.is_empty() || !word.chars().all(char::is_whitespace) {
                    temp.push_str(word);
                }
                budget -= word.width();
                break;
            }

            let mut target = budget;
            let (left, right) = loop {
                if word.is_char_boundary(target) {
                    break word.split_at(target);
                }
                target -= 1; // this should never underflow
            };

            temp.push_str(left);
            vec.push(std::mem::replace(&mut temp, String::with_capacity(max)));
            budget = max;

            word = right;
        }
    }

    if !temp.is_empty() {
        vec.push(temp)
    }
    vec
}

/// Truncate a string to a size of 'max', appending an ellipsis
fn truncate<'a>(input: &'a str, max: usize) -> Cow<'a, str> {
    if input.width() > max {
        let mut input = input.graphemes(true).take(max - 1).collect::<String>();
        input.push('…');
        input.into()
    } else {
        (*input).into()
    }
}

/// Bounded queue
struct Queue<T> {
    buf: VecDeque<T>,
    size: usize,
}

impl<T> Queue<T> {
    /// Make a new bounded queue with a max size
    fn with_size(size: usize) -> Self {
        Self {
            buf: VecDeque::with_capacity(size),
            size,
        }
    }
    /// Push an element onto the back of the queue (removing any overflow from the front)
    fn push(&mut self, item: T) {
        while self.buf.len() >= self.size {
            self.buf.pop_front();
        }
        self.buf.push_back(item);
    }
    /// Returns the length of the queue
    fn len(&self) -> usize {
        self.buf.len()
    }
    /// Gets the last 'size' elements from the queue
    fn view_last<'a>(&'a self, n: usize) -> impl Iterator<Item = &'a T> {
        let max = self.len();
        let delta = max.saturating_sub(n);
        self.buf.iter().skip(delta)
    }
}

/// Crossterm has poor writing, this is faster
struct BufferedWriter(std::io::Stdout);

impl BufferedWriter {
    /// Make a new BufferedWriter from stdout
    fn stdout() -> Self {
        BufferedWriter(std::io::stdout())
    }
    /// Clear the entire screen
    fn clear_screen(&mut self) {
        self.goto(0, 0);
        self.write_all(b"\x1b[0J").unwrap();
    }
    /// Hide the curosr
    fn hide_cursor(&mut self) {
        self.write_all(b"\x1b[25l").unwrap();
    }
    /// Show the cursor
    fn show_cursor(&mut self) {
        self.write_all(b"\x1b[25h").unwrap();
    }
    /// Scroll up by 'n' lines
    fn scroll(&mut self, n: usize) {
        self.write_all(&["\x1b[", &n.to_string(), "S"].concat().as_bytes())
            .unwrap()
    }
    /// Goto 'row' and 'col'
    fn goto(&mut self, row: usize, col: usize) {
        let (row, col) = (row + 1, col + 1);
        self.write_all(
            &["\x1b[", &row.to_string(), ";", &col.to_string(), "H"]
                .concat()
                .as_bytes(),
        )
        .unwrap();
    }
}

impl Write for BufferedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write_all(buf).map(|_| buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// stdout doesn't flush when its dropped
impl Drop for BufferedWriter {
    fn drop(&mut self) {
        let _ = self.0.flush();
    }
}

/// The display window
struct Window {
    term: crossterm::Terminal,
    queue: Queue<commands::PrivMsg>,
    width: u16,
    height: u16,
    left_pad: Paint<Cow<'static, str>>,
}

impl Window {
    /// Max width of names before they are truncated (sets up width of the 'left' column)
    const NICK_MAX: usize = 11;

    /// Create a new window with a buffer of 'buf_max' messages
    fn new(buf_max: usize) -> Self {
        let term = crossterm::terminal();
        let (width, height) = term.terminal_size();
        let left_pad = Paint::new(" ".repeat(Self::NICK_MAX).into()).fg(Color::Unset);

        Self {
            term,
            queue: Queue::with_size(buf_max),
            width,
            height,
            left_pad,
        }
    }

    /// Check the window size, updating if its different and forcing a full redraw
    fn check_size(&mut self) {
        let (width, height) = self.term.terminal_size();
        if self.width == width && self.height == height {
            return;
        }

        self.width = width;
        self.height = height;

        let mut writer = BufferedWriter::stdout();
        writer.clear_screen();

        let right = (self.width as usize) - Self::NICK_MAX - 3; // for ' | '
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
    fn add(&mut self, msg: commands::PrivMsg) {
        let mut writer = BufferedWriter::stdout();

        let right = (self.width as usize) - Self::NICK_MAX - 3; // for ' | '
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
        let nick = Paint::new(truncate(msg.user(), Self::NICK_MAX)).fg(Color::RGB(r, g, b));

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
            writeln!(writer, "{: >max$} | {}", left, right, max = Self::NICK_MAX).unwrap();
            *budget -= 1;
        }
    }
}

fn main() {
    let mut args = std::env::args();
    let program = args.next().unwrap();
    let channel = args.next().unwrap_or_else(|| {
        eprintln!("usage: {} <channel>", program);
        std::process::exit(1);
    });

    let (tx, rx) = channel::unbounded();
    let handle = std::thread::spawn(move || {
        let mut window = Window::new(QUEUE_SIZE);
        BufferedWriter::stdout().clear_screen();
        loop {
            channel::select! {
                recv(rx) -> event => {
                    match event {
                        Ok(event) => window.add(event),
                        _ => return,
                    }
                },
                default(TIMEOUT) => { window.check_size() },
            }
        }
    });

    let (name, pass) = twitchchat::ANONYMOUS_LOGIN;
    let client = twitchchat::connect_easy(name, pass).unwrap_or_else(|err| {
        eprintln!("cannot connect: {}", err);
        std::process::exit(1);
    });

    let writer = client.writer();
    for event in client.filter::<commands::PrivMsg>() {
        match event {
            Event::IrcReady(..) => {
                writer.join(&channel).unwrap();
            }
            Event::Message(TwitchMessage::PrivMsg(msg)) => {
                let _ = tx.send(msg.into());
            }
            Event::Error(err) => {
                eprintln!("error from twitch: {}", err);
                break;
            }
            _ => {}
        }
    }

    // TODO wait for ctrl-c here
    drop(tx);

    let _ = handle.join().unwrap();
}
