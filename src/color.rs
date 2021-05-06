pub const DEFAULT_COLOR: Color = Color(0xC0, 0xC0, 0xC0);

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Color(pub u8, pub u8, pub u8);

impl From<Color> for crossterm::style::Color {
    fn from(Color(r, g, b): Color) -> Self {
        Self::Rgb { r, g, b }
    }
}

impl From<twitchchat::twitch::color::RGB> for Color {
    fn from(twitchchat::twitch::color::RGB(r, g, b): twitchchat::twitch::color::RGB) -> Self {
        Self(r, g, b)
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(r, g, b) = self;
        write!(f, "#{:02X}{:02X}{:02X}", r, g, b)
    }
}

impl Default for Color {
    fn default() -> Self {
        DEFAULT_COLOR
    }
}

impl std::str::FromStr for Color {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        use anyhow::Context as _;

        let s = input.trim();
        let s = match s.len() {
            7 if s.starts_with('#') => &s[1..],
            6 if s.chars().all(|c| c.is_ascii_hexdigit()) => s,
            _ => anyhow::bail!("invalid hex string"),
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
