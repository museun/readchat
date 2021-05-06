use std::sync::mpsc::SyncSender;

use crate::window::Entry;
use crossterm::event::*;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Message {
    Connecting,
    Connected,
    Disconnected,
    Joining,
    Joined,
    ClearChat,
    ClearMsg,
    //
    Redraw,
    LinksViewMode,
    MessagesViewMode,
    ToggleTimestamps,
    Append(Entry),
}

#[rustfmt::skip]
macro_rules! key {
    (@char $char:expr; $modifier:ident) => { KeyEvent { code: KeyCode::Char($char), modifiers: KeyModifiers::$modifier } };
    (char $char:expr) => { key!(@char $char; NONE) };
    (shift $char:expr) => { key!(@char $char; SHIFT) };
    (ctrl $char:expr) => { key!(@char $char; CONTROL) };
    ($code:ident) => { KeyEvent { code: KeyCode::$code, modifiers: KeyModifiers::NONE } };
}

pub struct KeyManager {
    events: SyncSender<Message>,
}

impl KeyManager {
    pub const fn new(events: SyncSender<Message>) -> Self {
        Self { events }
    }

    pub fn handle(&self, event: KeyEvent) -> bool {
        let send = |msg| self.events.send(msg);

        let _ = match event {
            key!(ctrl 'c') => return false,
            key!(ctrl 'r') => send(Message::Redraw),

            key!(char 'l') => send(Message::LinksViewMode),
            key!(char 'm') => send(Message::MessagesViewMode),

            key!(char 't') => send(Message::ToggleTimestamps),

            _ => Ok(()),
        };

        true
    }
}
