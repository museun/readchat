use super::{args::Args, twitch::TwitchChat};
use super::{queue::Queue, *};

use std::io::Write;
use std::sync::Arc;

use twitchchat::connector::async_io::Connector;
use twitchchat::messages::Privmsg;
use twitchchat::twitch::color::RGB;

use crossterm::{
    cursor::*,
    event::*,
    style::*,
    terminal::{self, *},
};

use futures_lite::StreamExt as _;

pub async fn main_loop(
    args: Args,
    ex: Arc<async_executor::Executor<'static>>,
) -> anyhow::Result<()> {
    let mut window = Window::new(args.nick_max, args.buffer_max);
    let mut reader = EventStream::new();

    let (messages_tx, mut messages_rx) = twitchchat::channel::bounded(64);
    let (done_tx, mut done_rx) = twitchchat::channel::bounded(1);

    let fut = {
        let connector = if args.debug {
            let addr = crate::testing::make_interesting_chat(15)?;
            Connector::custom(addr)
        } else {
            Connector::twitch()
        }?;

        async move {
            let res = TwitchChat::run_to_completion(args.channel, messages_tx, connector).await;
            done_tx.send(res).await
        }
    };

    ex.spawn(fut).detach();

    use keys::LoopState;
    use util::Select;

    loop {
        let next_event = reader.next();
        let next_msg = messages_rx.next();
        let done = done_rx.next();

        let select = util::select_3(next_event, next_msg, done).await;
        match select {
            Select::A(Some(Ok(event))) => match event {
                Event::Key(event) => match keys::handle(event) {
                    LoopState::Continue => continue,
                    LoopState::Break => break,
                },
                Event::Resize(_, _) => window.update(UpdateMode::Redraw)?,
                _ => {}
            },

            Select::B(Some(msg)) => {
                window.push(msg);
                window.update(UpdateMode::Append)?;
            }

            Select::C(_done) => break,
            _ => break,
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

            crossterm::queue!(
                stdout,
                // Print("\n"),
                MoveToColumn(0),
                Print(style(name).with(color))
            )?;

            for (i, part) in util::partition(&msg.data(), width - left - 1)
                .into_iter()
                .enumerate()
            {
                if i > 0 {
                    // if cfg!(target_os = "windows") {
                    // crossterm::queue!(stdout, Print("\n"), Print(pad))?;
                    // } else {
                    crossterm::queue!(stdout, MoveToNextLine(1), Print(pad))?;
                    // }
                }
                crossterm::queue!(stdout, Print(" "), Print(style(part).with(color)))?;
            }

            crossterm::queue!(stdout, MoveToNextLine(1))?;

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
