use std::borrow::Cow;
use std::io::Write as _;

pub struct Message {
    pub color: twitchchat::color::RGB,
    pub nick: Cow<'static, str>,
    pub data: Cow<'static, str>,
}

pub struct TwitchChat {
    pub dispatcher: twitchchat::Dispatcher,
    runner: twitchchat::Runner,
    control: twitchchat::Control,
}

impl TwitchChat {
    pub fn new() -> Self {
        let dispatcher = twitchchat::Dispatcher::new();
        let (runner, control) = twitchchat::Runner::new(dispatcher.clone(), Default::default());

        Self {
            dispatcher,
            runner,
            control,
        }
    }

    pub async fn run_to_completion(mut self, channel: String) -> anyhow::Result<()> {
        let (nick, pass) = twitchchat::ANONYMOUS_LOGIN;
        use crossterm::style::{style, Color, Print};

        crossterm::execute!(
            std::io::stdout(),
            Print(style("connecting..").with(Color::Cyan)),
            crossterm::cursor::MoveToNextLine(1),
        )?;

        let conn = twitchchat::connect_easy(nick, pass).await?;
        crossterm::execute!(
            std::io::stdout(),
            crossterm::cursor::MoveToPreviousLine(1),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
            Print(style("joining ").with(Color::Cyan)),
            Print(style(&channel).with(Color::Green)),
            crossterm::cursor::MoveToNextLine(1),
        )?;

        self.control.writer().join(&channel).await?;

        use futures::prelude::*;

        let wait_for = self
            .dispatcher
            .wait_for::<twitchchat::events::RoomState>()
            .inspect(move |_| {
                let _ = crossterm::execute!(
                    std::io::stdout(),
                    crossterm::cursor::MoveToPreviousLine(1),
                    crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
                    Print(style("joined ").with(Color::Cyan)),
                    Print(style(channel).with(Color::Green)),
                    crossterm::cursor::MoveToNextLine(1),
                );
            });

        let (left, _) = tokio::join! {
            self.runner.run(conn),
            wait_for
        };

        match left? {
            twitchchat::Status::Canceled => {}
            twitchchat::Status::Eof => {}
        }

        Ok(())
    }
}
