use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[rustfmt::skip]
macro_rules! key {
    (@char $char:expr; $modifier:ident) => { KeyEvent { code: KeyCode::Char($char), modifiers: KeyModifiers::$modifier } };
    (char $char:expr) => { key!(@char $char; NONE) };
    (shift $char:expr) => { key!(@char $char; SHIFT) };
    (ctrl $char:expr) => { key!(@char $char; CONTROL) };
    ($code:ident) => { KeyEvent { code: KeyCode::$code, modifiers: KeyModifiers::NONE } };
}

// TODO add keybindings
pub fn handle(event: KeyEvent) -> LoopState {
    match event {
        key!(ctrl 'c') => LoopState::Break,
        _ => LoopState::Continue,
    }
}

pub enum LoopState {
    Continue,
    Break,
}
