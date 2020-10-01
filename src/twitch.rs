use std::io::Write as _;

use twitchchat::{
    channel::Sender,
    messages::{Commands, Privmsg},
    Status,
};

pub struct TwitchChat;

impl TwitchChat {
    pub async fn run_to_completion(
        channel: String,
        messages: Sender<Privmsg<'static>>,
    ) -> anyhow::Result<()> {
        use crossterm::style::{style, Color, Print};

        crossterm::execute!(
            std::io::stdout(),
            Print(style("connecting..").with(Color::Cyan)),
            crossterm::cursor::MoveToNextLine(1),
        )?;

        let user_config = twitchchat::UserConfig::builder()
            .anonymous()
            .enable_all_capabilities()
            .build()?;

        let connector = twitchchat::connector::AsyncIoConnector::twitch()?;

        let mut runner = twitchchat::AsyncRunner::connect(connector, &user_config).await?;

        crossterm::execute!(
            std::io::stdout(),
            crossterm::cursor::MoveToPreviousLine(1),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
            Print(style("joining ").with(Color::Cyan)),
            Print(style(&channel).with(Color::Green)),
            crossterm::cursor::MoveToNextLine(1),
        )?;

        runner.join(&channel).await?;
        crossterm::execute!(
            std::io::stdout(),
            crossterm::cursor::MoveToPreviousLine(1),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
            Print(style("joined ").with(Color::Cyan)),
            Print(style(channel).with(Color::Green)),
            crossterm::cursor::MoveToNextLine(1),
        )?;

        loop {
            match runner.next_message().await? {
                Status::Message(Commands::Privmsg(msg)) => {
                    if messages.send(msg).await.is_err() {
                        break Ok(());
                    }
                }
                Status::Quit | Status::Eof => break Ok(()),
                _ => continue,
            }
        }
    }
}
