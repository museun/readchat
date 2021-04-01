use std::{
    net::TcpStream,
    time::{Duration, Instant},
};

use crate::{
    args::Args,
    keys::{self, Message},
    twitch,
    window::{UpdateMode, ViewMode, Window},
    Logger,
};

use crossterm::event::*;
use flume as channel;

pub struct App {
    pub(crate) view_mode: ViewMode,
    pub(crate) waiting: bool,
    pub(crate) showing_info: Option<Instant>,
    pub(crate) window: Option<Window>,
    pub(crate) args: Args,
}

impl App {
    pub fn run(args: Args, mut logger: Logger) -> anyhow::Result<()> {
        logger.transcribe(&format!("*** session start: {}", crate::timestamp()))?;

        let addr = if args.debug {
            use crate::testing::*;
            make_interesting_chat(TestingOpts::load())?
        } else {
            twitchchat::TWITCH_IRC_ADDRESS.parse()?
        };
        let conn = TcpStream::connect(addr)?;

        let (sender, messages) = channel::bounded(64);
        let _ = std::thread::spawn({
            let channel = args.channel.clone();
            move || {
                let _ = twitch::run_to_completion(channel, sender, conn);
            }
        });
        let (events_tx, events_rx) = channel::bounded(32);

        let mut this = Self {
            view_mode: args
                .min_width
                .map(|_| ViewMode::Compact)
                .unwrap_or(ViewMode::Normal),

            window: Some(Window::new(args.nick_max, args.buffer_max, args.min_width)),

            waiting: false,
            showing_info: None,

            args,
        };

        'outer: while keep_running(&messages) {
            if crossterm::event::poll(Duration::from_millis(150))? {
                match crossterm::event::read()? {
                    Event::Key(event) => keys::handle(event, &events_tx),
                    Event::Resize(_, _) => {
                        this.update(UpdateMode::Redraw)?;
                    }
                    _ => {}
                }
            }

            for event in events_rx.try_iter() {
                if !this.dispatch(event)? {
                    break 'outer;
                }
            }

            if this.waiting {
                continue 'outer;
            }

            for msg in messages.try_iter() {
                logger.transcribe(&format!(
                    "{} {}: {}",
                    crate::timestamp(),
                    msg.name(),
                    msg.data()
                ))?;

                this.update_with_window(
                    move |window| {
                        window.push(msg);
                        Ok(())
                    },
                    UpdateMode::Append,
                )?;
            }
        }

        todo!();
    }

    fn dispatch(&mut self, event: Message) -> anyhow::Result<bool> {
        use {Message as M, ViewMode as V};

        let update_mode = match (self.waiting, self.showing_info) {
            (.., Some(..)) => UpdateMode::Info,
            (false, None) => UpdateMode::Redraw,
            (true, None) => UpdateMode::MarkAll,
        };

        match (event, self.view_mode) {
            (M::Quit, ..) => return Ok(false),

            (M::Redraw, ..) => self.update(UpdateMode::Redraw)?,

            (M::Delete, V::Normal) | (M::Delete, V::ForcedNormal) if !self.waiting => {
                self.waiting = !self.waiting;
                self.update(UpdateMode::MarkAll)?;
            }

            (M::Delete, V::Normal) | (M::Delete, V::ForcedNormal) if self.waiting => {
                self.waiting = !self.waiting;
                self.update(UpdateMode::Redraw)?
            }

            (M::Char(ch), V::Normal) | (M::Char(ch), V::ForcedNormal) if self.waiting => {
                return self
                    .with_window(|window, this| {
                        window.delete(ch, this)?;
                        this.waiting = false;
                        Ok(())
                    })
                    .map(|_| true)
            }

            (M::NameColumnGrow, V::Normal) | (M::NameColumnGrow, V::ForcedNormal) => self
                .with_window(|window, this| {
                    if Window::grow_nick_column(window) {
                        return window.update(this, update_mode);
                    }
                    Ok(())
                })?,

            (M::NameColumnShrink, V::Normal) | (M::NameColumnShrink, V::ForcedNormal) => self
                .with_window(|window, this| {
                    if Window::shrink_nick_column(window) {
                        return window.update(this, update_mode);
                    }
                    Ok(())
                })?,

            (M::ToggleTimestamps, V::Compact) | (M::ToggleTimestamps, V::ForcedCompact) => {
                self.args.timestamps = !self.args.timestamps;
                self.update(UpdateMode::Redraw)?;
            }

            _ => {}
        }

        Ok(true)
    }

    #[track_caller]
    fn with_window(
        &mut self,
        func: impl FnOnce(&mut Window, &mut Self) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let mut window = self
            .window
            .take()
            .expect("exclusive ownership of the window");

        func(&mut window, self)?;

        assert!(
            self.window.replace(window).is_none(),
            "single ownership of window"
        );

        Ok(())
    }

    fn update(&mut self, mode: UpdateMode) -> anyhow::Result<()> {
        self.update_with_window(|_| Ok(()), mode)
    }

    fn update_with_window(
        &mut self,
        func: impl FnOnce(&mut Window) -> anyhow::Result<()>,
        mode: UpdateMode,
    ) -> anyhow::Result<()> {
        self.with_window(|window, this| {
            func(window)?;
            window.update(this, mode)
        })
    }
}

fn keep_running<T>(ch: &channel::Receiver<T>) -> bool {
    !matches!(ch.try_recv(), Err(channel::TryRecvError::Disconnected))
}
