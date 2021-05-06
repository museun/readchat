use std::io::Write;

#[macro_use]
pub mod debug;

mod util;
pub use util::*;

mod color;
pub use color::Color;

mod alt_screen;
pub use alt_screen::AltScreen;

mod window;
pub use window::{Entry, Window};

mod logger;
pub use logger::Logger;

mod app;
pub use app::App;

mod events;
pub use events::Message;

mod config;

mod links_view;
mod user;

fn print_padding_and_timestamp<T>(
    writer: &mut impl Write,
    ts: &chrono::DateTime<T>,
    format: (&str, &str),
    color: Color,
    left: usize,
) -> anyhow::Result<()>
where
    T: chrono::TimeZone,
    T::Offset: std::fmt::Display,
{
    use crossterm::style::*;

    let (format, strftime) = format;
    let width = left - format.len();
    let ts = ContentStyle::new()
        .apply(ts.format(strftime))
        .with(color.into());

    crossterm::queue!(
        writer,
        Print(format_args!("{: <width$}", "", width = width)),
        Print(ts),
    )?;
    Ok(())
}

// fn draw_box(out: &mut impl Write, (x, y): (u16, u16), (w, h): (u16, u16)) -> anyhow::Result<()> {
//     use {
//         crossterm::{cursor::*, style::*, terminal::*},
//         std::iter::repeat,
//     };

//     crossterm::queue!(out, Clear(ClearType::All),)?;

//     (x..x + w)
//         .zip(repeat(y))
//         .chain((x..x + w).zip(repeat(y + h)))
//         .zip(repeat('─'))
//         .chain(
//             repeat(x)
//                 .zip(y..y + h)
//                 .chain(repeat(x + w).zip(y..y + h))
//                 .zip(repeat('│')),
//         )
//         .chain(array_iter([
//             ((x, y), '┌'),
//             ((x + w, y), '┐'),
//             ((x + w, y + h), '┘'),
//             ((x, y + h), '└'),
//         ]))
//         .try_for_each(|((x, y), e)| crossterm::queue!(out, MoveTo(x, y), Print(e)))?;

//     crossterm::queue!(out, Print("\n"))?;

//     Ok(())
// }

#[test]
fn asdf() {
    use crate::{user::User, window::Entry};

    let (mut app, _tx) = App::create(Window::new(), Logger::default(), "#testing");

    fn choose_color() -> Color {
        thread_local! { static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0) }
        let max = crate::debug::DEFAULT_COLORS.len();
        let n = N.with(|c| c.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
        crate::debug::DEFAULT_COLORS[(n + max - 1) % max]
    }

    let p = |s| {
        use chrono::offset::TimeZone as _;
        chrono::Local.datetime_from_str(s, "%b %d %y %X").unwrap()
    };

    let s = [
        (
            p("Apr 19 21 12:14:12"),
            "kappatan",
            "something https://github.com/tonsky/FiraCode",
        ),
        (
            p("Apr 19 21 12:14:38"),
            "mrhalzy",
            "https://www.youtube.com/watch?v=oee3Z69sE-8 and some video",
        ),
        (
            p("Apr 19 21 12:16:48"),
            "kappatan",
            "this is the https://github.com/challenger-deep-theme/vim configuration",
        ),
        (
            p("Apr 19 21 12:23:24"),
            "kappatan",
            "https://github.com/tonsky/FiraCode another thing",
        ),
        (
            p("Apr 19 21 12:23:07"),
            "museun",
            "see https://crates.io/crates/regex and https://crates.io/crates/cargo",
        ),
        (
            p("Apr 19 21 12:23:14"),
            "kappatan",
            "https://github.com/halzy/dotfiles/blob/master/nvim/init.vim",
        ),
        (
            p("Apr 19 21 12:23:24"),
            "zenekron",
            "https://crates.io/crates/isahc is something",
        ),
    ];

    let mut color_cache = std::collections::HashMap::new();

    for (ts, k, v) in array_iter(s) {
        let color = *color_cache.entry(k).or_insert_with(choose_color);

        app.window.add(Entry {
            ts,
            user: User {
                name: k.to_string().into(),
                color,
            },
            data: v.to_string().into(),
        })
    }

    app.run(&mut std::io::BufWriter::new(std::io::stdout()))
        .unwrap();
}
