use super::{
    args::Args,
    twitch::TwitchChat,
    window::{UpdateMode, Window},
};

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures_lite::StreamExt as _;

use std::sync::Arc;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

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
        } => LoopState::Break,

        _ => LoopState::Continue,
    }
}

pub async fn main_loop(
    Args {
        nick_max,
        buffer_max,
        channel,
    }: Args,
    ex: Arc<async_executor::Executor<'static>>,
) -> anyhow::Result<()> {
    use crossterm::style::{style, Color, Print};
    use std::io::Write as _;

    crossterm::execute!(
        std::io::stdout(),
        Print(style("press Ctrl-C to exit").with(Color::Red)),
        Print("\n")
    )?;

    let mut window = Window::new(nick_max, buffer_max);
    let mut reader = EventStream::new();

    let (messages_tx, mut messages_rx) = twitchchat::channel::bounded(64);
    let (done_tx, mut done_rx) = twitchchat::channel::bounded(1);

    ex.spawn(async move {
        let res = TwitchChat::run_to_completion(channel, messages_tx).await;
        let _ = done_tx.send(res).await;
    })
    .detach();

    loop {
        let next_event = reader.next();
        let next_msg = messages_rx.next();
        let done = done_rx.next();

        let select = select_3(next_event, next_msg, done).await;
        match select {
            E3::A(Some(Ok(event))) => {
                match event {
                    Event::Key(event) => match handle_key(event) {
                        LoopState::Continue => continue,
                        LoopState::Break => break,
                    },
                    Event::Resize(_, _) => {
                        // TODO debounce this
                        // would just delay the task and check to see if we had another resize during that period
                        window.update(UpdateMode::Redraw)?;
                    }
                    _ => {}
                }
            }

            E3::B(Some(msg)) => {
                window.push(msg);
                window.update(UpdateMode::Append)?;
            }

            E3::C(_done) => break,
            _ => break,
        }
    }

    Ok(())
}

enum E3<A, B, C> {
    A(A),
    B(B),
    C(C),
}

pin_project_lite::pin_project! {
     struct S3<A, B, C>{
        #[pin] a: A,
        #[pin] b: B,
        #[pin] c: C,
    }
}

impl<A, B, C> Future for S3<A, B, C>
where
    A: Future + Send + Sync,
    A::Output: Send + Sync,

    B: Future + Send + Sync,
    B::Output: Send + Sync,

    C: Future + Send + Sync,
    C::Output: Send + Sync,
{
    type Output = E3<A::Output, B::Output, C::Output>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        if let Poll::Ready(t) = this.a.poll(ctx) {
            return Poll::Ready(E3::A(t));
        }

        if let Poll::Ready(t) = this.b.poll(ctx) {
            return Poll::Ready(E3::B(t));
        }

        if let Poll::Ready(t) = this.c.poll(ctx) {
            return Poll::Ready(E3::C(t));
        }

        Poll::Pending
    }
}

fn select_3<A, B, C>(a: A, b: B, c: C) -> S3<A, B, C> {
    S3 { a, b, c }
}
