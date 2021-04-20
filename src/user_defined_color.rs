use crate::{color::Color, pair::Pair};
use crossterm::style::StyledContent;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserDefinedColor {
    Color(Color),
    Pair(Pair),
}

impl std::fmt::Debug for UserDefinedColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Color(color) => color.fmt(f),
            Self::Pair(pair) => pair.fmt(f),
        }
    }
}

impl UserDefinedColor {
    pub fn as_display_triplet(&self) -> String {
        match self {
            Self::Color(color) => color.as_display_triplet(),
            Self::Pair(Pair { fg, bg }) => {
                format!("{} on {}", fg.as_display_triplet(), bg.as_display_triplet())
            }
        }
    }

    pub fn styled<'a, D>(&self, input: D) -> StyledContent<D>
    where
        D: 'a + Display + Clone,
    {
        match self {
            Self::Color(color) => color.styled(input),
            Self::Pair(pair) => pair.styled(input),
        }
    }
}
