use std::{
    borrow::Cow,
    io::Write,
    sync::Arc,
    time::{Duration, Instant},
};

use unicode_width::UnicodeWidthStr;

use crate::{
    links_view::{LinkEntry, LinkSort},
    print_padding_and_timestamp,
    user::User,
    util::*,
    Color,
};

#[derive(Default)]
pub struct Window {
    limit: usize,
    show_timestamps: bool,
    entries: Vec<Entry>,
    links: Vec<LinkEntry>,
    view: View,
    status_mode: StatusMode,
}

impl Window {
    pub fn new() -> Self {
        Self::with_limit(50)
    }

    pub fn with_limit(limit: usize) -> Self {
        Self {
            limit,
            ..<_>::default()
        }
    }

    pub fn toggle_timestamps(&mut self) {
        self.show_timestamps = !self.show_timestamps
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn add(&mut self, entry: Entry) {
        while self.entries.len() >= self.limit {
            self.entries.rotate_left(1);
            self.entries.pop();
        }

        // XXX: should we remove 'stale' links when the queue is rotated?
        let link_entries = entry.scry_links().map(|link| LinkEntry::new(link, &entry));
        self.links.extend(link_entries);

        self.entries.push(entry);
        self.entries.sort_unstable_by_key(|e| e.ts);
    }

    pub const fn view(&self) -> &View {
        &self.view
    }

    pub const fn status_mode(&self) -> &StatusMode {
        &self.status_mode
    }

    pub fn set_view(&mut self, view: View) {
        self.view = view;
    }

    pub fn is_status_stale(&self) -> bool {
        matches!(self.status_mode, StatusMode::Show(_, dt) if dt.elapsed() >= Duration::from_secs(5))
    }

    pub fn hide_status(&mut self) {
        self.status_mode = StatusMode::Hidden
    }

    pub fn set_status(&mut self, status: Status<'static>) {
        match &mut self.status_mode {
            StatusMode::Show(s, dt) => {
                *s = status;
                *dt = Instant::now();
            }
            s @ StatusMode::Hidden => *s = StatusMode::Show(status, Instant::now()),
        }
    }

    pub fn touch_status(&mut self) {
        if let StatusMode::Show(.., dt) = &mut self.status_mode {
            *dt = Instant::now();
        }
    }

    pub fn update(&mut self, state: &mut RenderState<&mut impl Write>) -> anyhow::Result<()> {
        use crossterm::cursor::*;
        if matches!(self.view, View::Message) {
            let entries = std::mem::take(&mut self.entries);

            if let Some(msg) = entries.last() {
                if entries.len() == 1 {
                    crossterm::execute!(state, MoveTo(0, 0))?;
                }
                self.write_single(state, msg)?;
            }

            assert!(
                std::mem::replace(&mut self.entries, entries).is_empty(),
                "entries should have not been written to"
            );
        }
        Ok(())
    }

    fn render_status(&mut self, state: &mut RenderState<&mut impl Write>) -> anyhow::Result<()> {
        const TIMEOUT: Duration = Duration::from_secs(5);
        use crossterm::{cursor::*, style::*};

        match &self.status_mode {
            &StatusMode::Show(.., old)
                if state
                    .dt
                    .checked_duration_since(old)
                    .filter(|&dt| dt >= TIMEOUT)
                    .is_some() =>
            {
                self.hide_status();
            }

            StatusMode::Show(text, ..) => {
                use crossterm::queue;

                let left = text.len();
                let x = (state.width - 1) / 2 - left as u16 / 2;

                queue!(
                    state,
                    SavePosition,
                    MoveTo(x, 0),
                    Print(text),
                    RestorePosition
                )?;
            }

            StatusMode::Hidden => {}
        }

        Ok(())
    }

    pub fn render_all(&mut self, state: &mut RenderState<&mut impl Write>) -> anyhow::Result<()> {
        use crossterm::{cursor::*, terminal::*};

        crossterm::queue!(state, Clear(ClearType::All), MoveTo(0, 0))?;
        self.render_status(state)?;

        match self.view {
            View::Message => {
                let entries = std::mem::take(&mut self.entries);

                for entry in &entries[(entries.len()).saturating_sub(state.height as usize)..] {
                    self.write_single(state, entry)?;
                }

                assert!(
                    std::mem::replace(&mut self.entries, entries).is_empty(),
                    "no writing should have happened to the entries"
                );
            }
            View::Links => self.show_links(state)?,
            _ => unimplemented!(),
        }

        state.flush()?;
        Ok(())
    }

    fn write_single(
        &mut self,
        state: &mut RenderState<&mut impl Write>,
        entry: &Entry,
    ) -> anyhow::Result<()> {
        use crossterm::{cursor::*, style::*};

        self.render_status(state)?;

        crossterm::queue!(
            state,
            Print("\n"),
            MoveToColumn(0),
            Print(style(&entry.user.name).with((entry.user.color).into())),
        )?;

        if self.show_timestamps {
            print_padding_and_timestamp(
                state,
                &entry.ts,
                ("HH:MM:SS", "%X"),
                Color(170, 85, 0),
                (state.width as usize) - entry.user.name.width(),
            )?;
        }

        for part in whitespace_partition(&entry.data, state.width as _, is_probably_a_uri) {
            crossterm::queue!(state, Print("\n"), MoveToColumn(0), Print(part))?;
        }

        crossterm::queue!(state, Print("\n"), MoveToColumn(0))?;

        Ok(())
    }

    // fn mark_deletes(&self, _state: State, _out: &mut impl Write) -> anyhow::Result<()> {
    //     todo!()
    // }

    fn show_links(&self, state: &mut RenderState<&mut impl Write>) -> anyhow::Result<()> {
        use crossterm::{cursor::*, style::*};

        let view = LinkSort(&*self.links);
        let list = view.sorted_by_ts();

        let mut last = 0usize;

        for (n, link) in list {
            if last == n {
                // just append the link
                crossterm::queue!(
                    state,
                    MoveToColumn(0),
                    Print(&link.link),
                    Print("\n"),
                    MoveToColumn(0),
                )?;

                continue;
            } else {
                crossterm::queue!(state, Print("\n"), MoveToColumn(0),)?;
            }

            last = n;

            crossterm::queue!(
                state,
                Print(style(&link.user.name).with(link.user.color.into())),
            )?;

            if self.show_timestamps {
                print_padding_and_timestamp(
                    state,
                    &link.ts,
                    ("HH:MM:SS", "%X"),
                    Color(170, 85, 0),
                    (state.width as usize) - link.user.name.width(),
                )?;
            }

            crossterm::queue!(
                state,
                Print("\n"),
                MoveToColumn(0),
                Print(&link.link),
                Print("\n"),
                MoveToColumn(0), // this is annoying
            )?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum View {
    Message,
    Delete,
    Links,
}

impl View {
    pub fn as_status(&self, prefix: &str) -> Status<'static> {
        use crossterm::style::Colorize as _;
        let (s, l) = match self {
            Self::Message => ("message".cyan(), "message".len()),
            Self::Delete => ("delete".red(), "delete".len()),
            Self::Links => ("links".yellow(), "links".len()),
        };
        Status::new(format!("{}{}", prefix, s), prefix.len() + l)
    }
}

impl Default for View {
    fn default() -> Self {
        Self::Message
    }
}

pub enum StatusMode {
    Show(Status<'static>, Instant),
    Hidden,
}

impl Default for StatusMode {
    fn default() -> Self {
        Self::Hidden
    }
}

#[derive(Clone, PartialEq)]
pub struct Status<'a>(pub Cow<'a, str>, pub usize);

impl<'a> Status<'a> {
    pub fn new(data: impl Into<Cow<'a, str>>, len: usize) -> Self {
        Self(data.into(), len)
    }
}

impl<'a> Status<'a> {
    const fn len(&self) -> usize {
        let &Self(.., l) = self;
        l
    }

    const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> std::fmt::Display for Status<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(s, ..) = self;
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Entry {
    pub ts: chrono::DateTime<chrono::Local>,
    pub user: User,
    pub data: Arc<str>,
}

impl Entry {
    fn scry_links(&self) -> impl Iterator<Item = String> + '_ {
        self.data
            .split_whitespace()
            .flat_map(url::Url::parse)
            .map(url::Url::into_string)
    }
}

impl<'a> From<twitchchat::messages::Privmsg<'a>> for Entry {
    fn from(msg: twitchchat::messages::Privmsg<'a>) -> Self {
        let msg = twitchchat::IntoOwned::into_owned(msg);

        Self {
            ts: chrono::Local::now(),
            user: User {
                color: msg.color().unwrap_or_default().rgb.into(),
                name: msg.name().to_string().into(),
            },
            data: msg.data().to_string().into(),
        }
    }
}

pub struct RenderState<O: Sized> {
    height: u16,
    width: u16,
    dt: Instant,
    out: O,
}

impl<O: Sized> RenderState<O> {
    pub fn current_size(out: O) -> anyhow::Result<Self> {
        crossterm::terminal::size()
            .map(|(width, height)| Self {
                height,
                width,
                out,
                dt: Instant::now(),
            })
            .map_err(<_>::into)
    }
}

impl<O: Sized + Write> Write for RenderState<O> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.out.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.out.flush()
    }
}
