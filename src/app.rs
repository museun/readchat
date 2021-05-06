use std::{
    io::Write,
    sync::mpsc::{Receiver, SyncSender},
    time::Duration,
};

use crossterm::{event::Event, style::Colorize};

use crate::{
    events::{KeyManager, Message},
    window::{RenderState, Status, View},
    Logger, Window,
};

pub struct App {
    pub(crate) window: Window,
    keys: KeyManager,
    logger: Logger,
    sender: SyncSender<Message>,
    receiver: Receiver<Message>,
    channel: String,
}

impl App {
    // TODO pass in key mappings and so we can create the messages/events channel for them
    pub fn create(
        window: Window,
        logger: Logger,
        channel: impl ToString,
    ) -> (Self, SyncSender<Message>) {
        let (tx, rx) = std::sync::mpsc::sync_channel(32);
        let this = Self {
            window,
            keys: KeyManager::new(tx.clone()),
            logger,
            sender: tx.clone(),
            receiver: rx,
            channel: channel.to_string(),
        };
        (this, tx)
    }

    pub fn run_on_buffered_stdout(self) -> anyhow::Result<()> {
        self.run(&mut std::io::BufWriter::new(std::io::stdout()))
    }

    pub fn run(mut self, out: &mut impl Write) -> anyhow::Result<()> {
        use Message::*;
        const PREFIX: &str = "current mode: ";

        loop {
            if crossterm::event::poll(Duration::from_millis(10))? {
                match crossterm::event::read()? {
                    Event::Key(event) => {
                        if !self.keys.handle(event) {
                            return Ok(());
                        }
                    }
                    Event::Resize(_, _) => {
                        let mut state = RenderState::current_size(&mut *out)?;
                        self.window.render_all(&mut state)?
                    }
                    Event::Mouse(_) => {}
                }
            }

            for message in self.receiver.try_iter() {
                let mut state = RenderState::current_size(&mut *out)?;

                match message {
                    Redraw if self.window.is_empty() => continue,
                    Redraw => { /* fallthrough */ }

                    msg @ LinksViewMode | msg @ MessagesViewMode => {
                        let view = match msg {
                            LinksViewMode => View::Links,
                            MessagesViewMode => View::Message,
                            _ => unreachable!(),
                        };

                        if *self.window.view() != view {
                            self.window.set_view(view);
                        }
                        self.window.set_status(view.as_status(PREFIX));
                    }

                    ToggleTimestamps => self.window.toggle_timestamps(),

                    Append(msg) => {
                        self.window.add(msg);
                        self.window.update(&mut state)?;
                    }

                    Connecting => {
                        self.window.set_status(Status(
                            "connecting...".yellow().to_string().into(),
                            "connecting...".len(),
                        ));
                    }
                    Connected => {
                        self.window.set_status(Status(
                            "connected".green().to_string().into(),
                            "connected".len(),
                        ));
                    }

                    Joining => {
                        let s = format!("joining {}", (&*self.channel).cyan());
                        self.window
                            .set_status(Status(s.into(), "joining ".len() + self.channel.len()));
                    }
                    Joined => {
                        let s = format!("joined {}", (&*self.channel).green());
                        self.window
                            .set_status(Status(s.into(), "joined ".len() + self.channel.len()));
                    }

                    _ => {}
                }

                self.window.render_all(&mut state)?;
                state.flush()?;
            }

            if self.window.is_status_stale() {
                let mut state = RenderState::current_size(&mut *out)?;
                self.window.render_all(&mut state)?;
                state.flush()?;
            }
        }
    }
}
