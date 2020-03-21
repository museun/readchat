use super::{
    args::Args,
    twitch::TwitchChat,
    window::{UpdateMode, Window},
};

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use tokio::stream::StreamExt as _;

enum LoopState {
    Continue,
    Break,
}

// TODO add keybindings
fn handle_key(event: KeyEvent) -> LoopState {
    match event {
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
        } => return LoopState::Break,

        _ => LoopState::Continue,
    }
}

pub async fn main_loop(
    Args {
        nick_max,
        buffer_max,
        channel,
    }: Args,
) -> anyhow::Result<()> {
    use crossterm::style::{style, Color, Print};
    use std::io::Write as _;
    crossterm::execute!(
        std::io::stdout(),
        Print(style("press Ctrl-C to exit").with(Color::Red)),
        Print("\n")
    )?;

    let chat = TwitchChat::new();
    let mut window = Window::new(nick_max, buffer_max);

    let mut reader = EventStream::new();
    let mut privmsg = chat.dispatcher.subscribe::<twitchchat::events::Privmsg>();

    tokio::pin! {
        let done = chat.run_to_completion(channel);
    }

    loop {
        tokio::select! {
            Some(Ok(event)) = reader.next() => {
                match event {
                    Event::Key(event) => {
                        match handle_key(event) {
                            LoopState::Continue => continue,
                            LoopState::Break => break,
                        }
                    }
                    Event::Resize(_,_) => {
                        // TODO debounce this
                        // would just delay the task and check to see if we had another resize during that period
                        window.update(UpdateMode::Redraw)?;
                    }
                    _ => {}
                }
            }
            Some(msg) = privmsg.next() => {
                let message = std::sync::Arc::try_unwrap(msg).unwrap_or_else(|data| (&*data).clone());
                window.push(message);
                window.update(UpdateMode::Append)?;
            }
            res = &mut done => { return res }
            else => { break }
        }
    }

    Ok(())
}
