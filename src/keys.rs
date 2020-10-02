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

// TODO add keybindings
pub fn handle(event: KeyEvent, events: &Sender<Message>) {
    match event {
        key!(ctrl 'c') => {
            let _ = events.send(Message::Quit);
        }
        key!(ctrl 'r') => {
            let _ = events.send(Message::Redraw);
        }
        key!(ctrl 'd') => {
            let _ = events.send(Message::Delete);
        }

        KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::NONE,
        } if matches!(ch,  'a'..='z' | '0'..='9') => {
            let _ = events.send(Message::Char(ch));
        }

        KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::SHIFT,
        } if matches!(ch, 'A'..='Z') => {
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
}
