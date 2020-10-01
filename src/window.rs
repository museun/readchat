use super::{
    args::Args,
    keys::{self, LoopState},
    queue::Queue,
    twitch::TwitchChat,
    util,
};

use std::io::Write;

use twitchchat::messages::Privmsg;
use twitchchat::twitch::color::RGB;

use crossbeam_channel as channel;

use crossterm::{
    cursor::*,
    event::*,
    style::*,
    terminal::{self, *},
};

fn keep_running(ch: &channel::Receiver<()>) -> bool {
    matches!(ch.try_recv(), Err(channel::TryRecvError::Empty))
}

pub fn main_loop(args: Args) -> anyhow::Result<()> {
    let mut window = Window::new(args.nick_max, args.buffer_max);

    let conn = if args.debug {
        let addr = crate::testing::make_interesting_chat(150)?;
        std::net::TcpStream::connect(addr)?
    } else {
        std::net::TcpStream::connect(twitchchat::TWITCH_IRC_ADDRESS)?
    };

    let (messages_tx, messages) = channel::bounded(64);
    let (done_tx, done) = channel::bounded(1);

    let _ = std::thread::spawn(move || {
        let _ = TwitchChat::run_to_completion(args.channel, messages_tx, conn);
        drop(done_tx)
    });

    while keep_running(&done) {
        if crossterm::event::poll(std::time::Duration::from_millis(150))? {
            match crossterm::event::read()? {
                Event::Key(event) => match keys::handle(event) {
                    LoopState::Continue => continue,
                    LoopState::Break => break,
                },
                Event::Resize(_, _) => window.update(UpdateMode::Redraw)?,
                _ => {}
            }
        }

        for msg in messages.try_iter() {
            window.push(msg);
            window.update(UpdateMode::Append)?;
        }
    }

    Ok(())
}

enum UpdateMode {
    Redraw,
    Append,
}

struct Window {
    queue: Queue<Privmsg<'static>>,
    left: usize,
    pad: String,
}

impl Window {
    fn new(left: usize, limit: usize) -> Self {
        Self {
            left,
            pad: " ".repeat(left),
            queue: Queue::with_size(limit),
        }
    }

    fn push(&mut self, message: Privmsg<'static>) {
        self.queue.push(message);
    }

    fn update(&mut self, update: UpdateMode) -> anyhow::Result<()> {
        fn print_message(
            stdout: &mut std::io::Stdout,
            msg: &Privmsg<'_>,
            left: usize,
            width: usize,
            pad: &str,
        ) -> anyhow::Result<()> {
            let RGB(r, g, b) = msg.color().unwrap_or_default().rgb;
            let color = Color::Rgb { r, g, b };
            let name = util::truncate_or_pad(&msg.name(), left);

            let name = style(name).with(color);

            for (i, part) in util::partition(&msg.data(), width - left - 1)
                .into_iter()
                .enumerate()
            {
                crossterm::queue!(stdout, Print("\n"), MoveToColumn(0))?;
                if i == 0 {
                    crossterm::queue!(stdout, Print(&name))?;
                } else {
                    crossterm::queue!(stdout, Print(pad))?;
                }
                crossterm::queue!(stdout, Print(" "), Print(part))?;
            }

            Ok(())
        }

        let (width, _h) = terminal::size()?;
        let mut stdout = std::io::stdout();

        match update {
            UpdateMode::Redraw if self.queue.is_empty() => return Ok(()),
            UpdateMode::Redraw => {
                crossterm::execute!(stdout, Clear(ClearType::All))?;
                for msg in self.queue.iter() {
                    print_message(&mut stdout, msg, self.left, width as _, &self.pad)?;
                }
            }
            UpdateMode::Append => {
                if let Some(msg) = self.queue.last() {
                    print_message(&mut stdout, msg, self.left, width as _, &self.pad)?;
                }
            }
        }

        stdout.flush()?;
        Ok(())
    }
}
