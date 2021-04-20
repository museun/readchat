use super::Color;
use crossterm::style::StyledContent;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Deserialize, Serialize, PartialEq, Copy, Clone, Hash, Eq, PartialOrd, Ord)]
pub struct Pair {
    pub fg: Color,
    pub bg: Color,
}

impl std::fmt::Debug for Pair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?} on {:?}", self.fg, self.bg))
    }
}

impl Pair {
    pub fn styled<'a, D>(&self, input: D) -> StyledContent<D>
    where
        D: 'a + Display + Clone,
    {
        let Pair { fg, bg } = *self;
        crossterm::style::style(input).with(fg.into()).on(bg.into())
    }
}
