use super::{Color, UserDefinedColor, DEFAULT_COLORS};
use crate::array_iter;

#[derive(Debug)]
pub struct UniqueishColors<const N: usize> {
    colors: [UserDefinedColor; N],
    pos: usize,
}

impl Default for UniqueishColors<{ DEFAULT_COLORS.len() }> {
    fn default() -> Self {
        Self::from_predetermined_colors(
            array_iter(DEFAULT_COLORS)
                .map(UserDefinedColor::Color)
                .enumerate()
                .fold(
                    [UserDefinedColor::Color(Color::default()); { DEFAULT_COLORS.len() }],
                    |mut colors, (i, color)| {
                        colors[i] = color;
                        colors
                    },
                ),
        )
    }
}

impl<const N: usize> UniqueishColors<N> {
    pub const fn from_predetermined_colors(colors: [UserDefinedColor; N]) -> Self {
        Self { colors, pos: 0 }
    }

    pub fn select(&self, input: &str) -> UserDefinedColor {
        self.lookup(simple_hash(0, input.as_bytes()) as usize)
    }

    pub fn next(&mut self) -> UserDefinedColor {
        let pos = self.pos;
        self.pos += 1;
        self.lookup(pos)
    }

    const fn lookup(&self, index: usize) -> UserDefinedColor {
        let max = self.colors.len();
        self.colors[(index as usize + max - 1) % max]
    }
}

#[inline]
fn simple_hash(seed: u32, data: &[u8]) -> u32 {
    const PRIME: u32 = 5;
    data.iter()
        .fold(seed, |a, c| ((PRIME << 5).wrapping_add(a)) + *c as u32)
}
