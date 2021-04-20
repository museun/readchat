use std::{borrow::Cow, fmt::Display};

use crossterm::style::StyledContent;

const DEFAULT_COLOR: Color = Color(0xC0, 0xC0, 0xC0);

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Color(pub u8, pub u8, pub u8);

impl Default for Color {
    fn default() -> Self {
        DEFAULT_COLOR
    }
}

impl std::str::FromStr for Color {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use anyhow::Context as _;

        let s = s.trim();
        let s = match s.len() {
            7 if s.starts_with('#') => &s[1..],
            6 if s.chars().all(|c| c.is_ascii_hexdigit()) => s,
            _ => {
                anyhow::bail!("invalid hex string")
            }
        };

        u32::from_str_radix(s, 16)
            .map(|s| {
                Self(
                    ((s >> 16) & 0xFF) as _,
                    ((s >> 8) & 0xFF) as _,
                    (s & 0xFF) as _,
                )
            })
            .with_context(|| "cannot parse hex string")
    }
}

impl std::fmt::Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Color(r, g, b) = self;
        write!(f, "#{:02X}{:02X}{:02X}", r, g, b)
    }
}

impl From<Color> for crossterm::style::Color {
    fn from(Color(r, g, b): Color) -> Self {
        Self::Rgb { r, g, b }
    }
}

impl serde::Serialize for Color {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let Self(r, g, b) = self;
        ser.collect_str(&format_args!("#{:02X}{:02X}{:02X}", r, g, b))
    }
}

impl<'de> serde::Deserialize<'de> for Color {
    fn deserialize<D>(deser: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <Cow<'_, str>>::deserialize(deser)?
            .parse()
            .map_err(|_| serde::de::Error::custom("invalid hex string"))
    }
}

impl Color {
    pub fn as_display_triplet(&self) -> String {
        let Self(r, g, b) = self;
        format!("{},{},{}", r, g, b)
    }

    pub fn styled<'a, D>(&self, input: D) -> StyledContent<D>
    where
        D: 'a + Display + Clone,
    {
        crossterm::style::style(input).with((*self).into())
    }
}
