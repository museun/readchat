use crate::Color;
use std::sync::Arc;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct User {
    pub color: Color,
    pub name: Arc<str>,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = crossterm::style::style(&*self.name).with(self.color.into());
        write!(f, "{}", s)
    }
}
