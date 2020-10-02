use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use flume::Sender;

#[rustfmt::skip]
macro_rules! key {
    (@char $char:expr; $modifier:ident) => { KeyEvent { code: KeyCode::Char($char), modifiers: KeyModifiers::$modifier } };
    (char $char:expr) => { key!(@char $char; NONE) };
    (shift $char:expr) => { key!(@char $char; SHIFT) };
    (ctrl $char:expr) => { key!(@char $char; CONTROL) };
    ($code:ident) => { KeyEvent { code: KeyCode::$code, modifiers: KeyModifiers::NONE } };
}

fn is_mark(ch: char, mod_: KeyModifiers) -> bool {
    (matches!(ch, 'a'..='z' | '0'..='9') && matches!(mod_, KeyModifiers::NONE))
        || (matches!(ch, 'A'..='Z') && matches!(mod_, KeyModifiers::SHIFT))
}

pub fn handle(event: KeyEvent, events: &Sender<Message>) {
    macro_rules! send {
        ($ev:tt) => {{
            let _ = events.send(Message::$ev);
        }};
    }

    match event {
        key!(ctrl 'c') => send!(Quit),
        key!(ctrl 'r') => send!(Redraw),
        key!(ctrl 'd') => send!(Delete),
        key!(shift '<') => send!(NameColumnShrink),
        key!(shift '>') => send!(NameColumnGrow),

        KeyEvent {
            code: KeyCode::Char(ch),
            modifiers,
        } if is_mark(ch, modifiers) => {
            let _ = events.send(Message::Char(ch));
        }

        _ => {}
    }
}

#[derive(Debug)]
pub enum Message {
    Quit,
    Redraw,
    Delete,
    Char(char),
    NameColumnGrow,
    NameColumnShrink,
}
