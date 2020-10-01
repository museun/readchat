use std::io::Write;

use twitchchat::{
    channel::Sender,
    connector::Connector,
    messages::{Commands, Privmsg},
    Status,
};

use crossterm::{cursor::*, style::*, terminal::*};
use futures_lite::{AsyncRead, AsyncWrite};

pub struct TwitchChat;

impl TwitchChat {
    pub async fn run_to_completion<C>(
        channel: String,
        messages: Sender<Privmsg<'static>>,
        connector: C,
    ) -> anyhow::Result<()>
    where
        C: Connector,
        for<'a> &'a C::Output: AsyncRead + AsyncWrite + Send + Sync + Unpin,
    {
        crossterm::execute!(
            std::io::stdout(),
            MoveTo(0, 0),
            Print(style("connecting..").with(Color::Cyan)),
            MoveToNextLine(1),
        )?;

        let user_config = twitchchat::UserConfig::builder()
            .anonymous()
            .enable_all_capabilities()
            .build()?;
        let mut runner = twitchchat::AsyncRunner::connect(connector, &user_config).await?;

        let mut out = std::io::stdout();

        replace_line(
            &mut out,
            format!(
                "{}{}",
                style("joining ").with(Color::Cyan),
                style(&channel).with(Color::Green),
            ),
        )?;

        runner.join(&channel).await?;

        replace_line(
            &mut out,
            format!(
                "{}{}",
                style("joined ").with(Color::Cyan),
                style(&channel).with(Color::Green),
            ),
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

fn replace_line(w: &mut impl Write, line: impl ToString) -> anyhow::Result<()> {
    crossterm::execute!(
        w,
        MoveToPreviousLine(1),
        Clear(ClearType::CurrentLine),
        Print(line.to_string()),
        MoveToNextLine(1),
    )?;
    Ok(())
}
