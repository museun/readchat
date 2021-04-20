use crossterm::style::{style, Print, StyledContent};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, fmt::Display, io::Write};

pub fn show_off_colors<W>(mut w: W, config: &ColorConfig) -> anyhow::Result<()>
where
    W: Write + Sized,
{
    config.render(&mut w)
}

fn array_iter<T, const N: usize>(
    array: [T; N],
) -> impl Iterator<Item = T> + ExactSizeIterator + Clone
where
    T: Clone,
{
    std::array::IntoIter::new(array)
}

fn draw_table<'a, W, D, const N: usize>(
    mut w: W,
    headers: [&str; N],
    rows: &[[StyledContent<D>; N]],
) -> anyhow::Result<()>
where
    W: Write + Sized,
    D: Display + Clone + Length + AsRef<str>,
{
    fn calculate_hints<D, const N: usize>(
        headers: &[&str; N],
        rows: &[[D; N]],
    ) -> ([usize; N], Vec<usize>)
    where
        D: Length,
    {
        let headers = headers
            .iter()
            .enumerate()
            .fold([0; N], |mut sizes, (i, c)| {
                sizes[i] = sizes[i].max(c.len());
                sizes
            });

        let rows = rows.iter().flat_map(|row| row.iter().enumerate()).fold(
            vec![0; rows.len() + 1], // what
            |mut sizes, (i, cell)| {
                sizes[i] = sizes[i].max(cell.len());
                sizes
            },
        );

        (headers, rows)
    }

    let (header_hint, row_hint) = calculate_hints(&headers, &rows);

    let hints = array_iter(header_hint)
        .zip(row_hint.iter())
        .map(|(l, r)| l.max(*r));

    let min_width = hints.clone().sum::<usize>() + header_hint.len() * 2;

    for (i, (header, hint)) in headers.iter().zip(hints).enumerate() {
        if i > 0 {
            crossterm::queue!(w, Print(" | "))?;
        }
        crossterm::queue!(w, Print(format!("{: <hint$}", header, hint = hint)))?;
    }

    crossterm::queue!(
        w,
        Print("\n"),
        Print(format!("{:->width$}", "", width = min_width)),
        Print("\n")
    )?;

    for row in rows {
        for (i, (cell, hint)) in row.iter().zip(row_hint.iter()).enumerate() {
            if i > 0 {
                crossterm::queue!(w, Print(" | "))?;
            }
            crossterm::queue!(
                w,
                Print(format!(
                    "{: <hint$}",
                    cell,
                    hint = (*hint).max(header_hint[i])
                ))
            )?;
        }
        crossterm::queue!(w, Print("\n"))?;
    }

    Ok(())
}

trait Length {
    fn len(&self) -> usize;
}

impl<D> Length for StyledContent<D>
where
    D: Display + Clone + AsRef<str>,
{
    fn len(&self) -> usize {
        self.content().as_ref().len()
    }
}

impl Length for String {
    fn len(&self) -> usize {
        str::len(self)
    }
}

impl Length for &str {
    fn len(&self) -> usize {
        str::len(self)
    }
}

pub trait Render {
    fn render<W>(&self, w: W) -> anyhow::Result<()>
    where
        W: Write + Sized;
}

impl Render for ColorMapping {
    fn render<W>(&self, w: W) -> anyhow::Result<()>
    where
        W: Write + Sized,
    {
        let headers = ["color name", "hex value", "triplet"];
        let rows: Vec<_> = self
            .map
            .iter()
            .map(|(base_name, color)| {
                [
                    color.styled(base_name.clone()),
                    style(format!("{:?}", color)),
                    style(color.as_display_triplet()),
                ]
            })
            .collect();

        draw_table(w, headers, &*rows)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorConfig {
    pub mapping: ColorMapping,
    pub interactions: Interactions,
}

impl Render for ColorConfig {
    fn render<W>(&self, mut w: W) -> anyhow::Result<()>
    where
        W: Write + Sized,
    {
        crossterm::queue!(
            w,
            Print(style("color mapping").with(crossterm::style::Color::Yellow)),
            Print(":\n")
        )?;
        self.mapping.render(&mut w)?;

        crossterm::queue!(
            w,
            Print("\n"),
            Print(style("interactions").with(crossterm::style::Color::Yellow)),
            Print(":\n")
        )?;
        self.interactions.render(&mut w)
    }
}

#[test]
fn asdf() {
    ColorConfig::default().render(std::io::stdout()).unwrap();
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            mapping: <_>::default(),
            interactions: Interactions {
                highlight: vec![
                    ColoredTerm {
                        term: "rust".to_string(),
                        color: UserDefinedColor::Pair(Pair {
                            fg: Color(183, 65, 14),
                            bg: Color(0, 0, 0),
                        }),
                        case_insensitive: false,
                    },
                    ColoredTerm {
                        term: "error detected".to_string(),
                        color: UserDefinedColor::Color(Color(255, 0, 0)),
                        case_insensitive: false,
                    },
                ],
                mention: vec![
                    ColoredTerm {
                        term: "museun".to_string(),
                        color: UserDefinedColor::Color(Color(255, 0, 0)),
                        case_insensitive: true,
                    },
                    ColoredTerm {
                        term: "shaken_bot".to_string(),
                        color: UserDefinedColor::Pair(Pair {
                            fg: Color(255, 0, 0),
                            bg: Color(255, 255, 255),
                        }),
                        case_insensitive: true,
                    },
                ],
            },
        }
    }
}

impl ColorConfig {
    pub fn load() -> anyhow::Result<Self> {
        let path = crate::Logger::get_dir()?.join("colors.yaml");
        let data = std::fs::read(&path)?;
        serde_yaml::from_slice(&data).map_err(Into::into)
    }
}

// TODO make this table at compile time, instead of being hard-coded
const COLOR_NAMES: [&str; 15] = [
    "blue",
    "blue_violet",
    "cadet_blue",
    "chocolate",
    "coral",
    "dodger_blue",
    "firebrick",
    "golden_rod",
    "green",
    "hot_pink",
    "orange_red",
    "red",
    "sea_green",
    "spring_green",
    "yellow_green",
];

// TODO make this table at compile time, instead of being hard-coded
const DEFAULT_COLORS: [Color; 15] = [
    Color(0, 0, 255),
    Color(138, 43, 226),
    Color(95, 158, 160),
    Color(210, 105, 30),
    Color(255, 127, 80),
    Color(30, 144, 255),
    Color(178, 34, 34),
    Color(218, 165, 32),
    Color(0, 128, 0),
    Color(255, 105, 180),
    Color(255, 69, 0),
    Color(255, 0, 0),
    Color(46, 139, 87),
    Color(0, 255, 127),
    Color(173, 255, 47),
];

fn lookup_default_color_by_name(s: &str) -> Option<Color> {
    // TODO have a snake_case conversion method
    let pos = COLOR_NAMES.iter().position(|&c| c == s)?;
    DEFAULT_COLORS.get(pos).copied()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorMapping {
    #[serde(flatten)]
    map: HashMap<String, UserDefinedColor>,
}

impl ColorMapping {
    fn as_array(&self) -> [UserDefinedColor; 15] {
        array_iter(COLOR_NAMES).enumerate().fold(
            [UserDefinedColor::Color(Color::default()); 15],
            |mut colors, (i, name)| {
                colors[i] = self.map[name];
                colors
            },
        )
    }
}

impl Default for ColorMapping {
    fn default() -> Self {
        let keys = array_iter(COLOR_NAMES).map(ToString::to_string);
        let values = array_iter(DEFAULT_COLORS).map(UserDefinedColor::Color);
        let map = keys.zip(values).collect();
        Self { map }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interactions {
    pub highlight: Vec<ColoredTerm>,
    pub mention: Vec<ColoredTerm>,
}

impl Render for Interactions {
    fn render<W>(&self, mut w: W) -> anyhow::Result<()>
    where
        W: Write + Sized,
    {
        let make_rows = |source: &[_]| -> Vec<_> {
            source
                .iter()
                .map(|term| {
                    let ColoredTerm {
                        term,
                        color,
                        case_insensitive,
                    } = term;

                    [
                        style(term.clone()),
                        color.styled(format!("{:?}", color)),
                        style(case_insensitive.to_string()),
                    ]
                })
                .collect()
        };

        let headers = ["term", "color", "case insensitive"];

        crossterm::queue!(
            w,
            Print(style("highlight").with(crossterm::style::Color::Yellow)),
            Print(":\n")
        )?;
        let rows = make_rows(&self.highlight);
        draw_table(&mut w, headers, &*rows)?;
        crossterm::queue!(w, Print("\n"))?;

        crossterm::queue!(
            w,
            Print(style("mention").with(crossterm::style::Color::Yellow)),
            Print(":\n")
        )?;
        let rows = make_rows(&self.mention);
        draw_table(&mut w, headers, &*rows)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColoredTerm {
    pub term: String,
    pub color: UserDefinedColor,
    // TODO: should there be a partial / fuzzy option?
    pub case_insensitive: bool, // exact?
}

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
                    [UserDefinedColor::Color(Color(0, 0, 0)); { DEFAULT_COLORS.len() }],
                    |mut colors, (i, color)| {
                        colors[i] = color;
                        colors
                    },
                ),
        )
    }
}

impl<const N: usize> UniqueishColors<N> {
    pub fn from_predetermined_colors(colors: [UserDefinedColor; N]) -> Self {
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

    fn lookup(&self, index: usize) -> UserDefinedColor {
        let max = self.colors.len();
        self.colors[(index as usize + max - 1) % max]
    }
}

fn simple_hash(seed: u32, data: &[u8]) -> u32 {
    const PRIME: u32 = 5;
    data.iter()
        .fold(seed, |a, c| ((PRIME << 5).wrapping_add(a)) + *c as u32)
}

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
    fn as_display_triplet(&self) -> String {
        match self {
            Self::Color(color) => color.as_display_triplet(),
            Self::Pair(Pair { fg, bg }) => {
                format!("{} on {}", fg.as_display_triplet(), bg.as_display_triplet())
            }
        }
    }

    fn styled<'a, D: 'a + Display + Clone>(&self, input: D) -> StyledContent<D> {
        match self {
            Self::Color(color) => color.styled(input),
            Self::Pair(pair) => pair.styled(input),
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Copy, Clone)]
pub struct Pair {
    pub fg: Color,
    pub bg: Color,
}

impl Pair {
    fn styled<'a, D>(&self, input: D) -> StyledContent<D>
    where
        D: 'a + Display + Clone,
    {
        let Pair { fg, bg } = *self;
        crossterm::style::style(input).with(fg.into()).on(bg.into())
    }
}

impl std::fmt::Debug for Pair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?} on {:?}", self.fg, self.bg))
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    fn as_display_triplet(&self) -> String {
        let Self(r, g, b) = self;
        format!("{},{},{}", r, g, b)
    }

    fn styled<'a, D>(&self, input: D) -> StyledContent<D>
    where
        D: 'a + Display + Clone,
    {
        crossterm::style::style(input).with((*self).into())
    }
}

impl Default for Color {
    fn default() -> Self {
        Self(0xC0, 0xC0, 0xC0)
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
        crossterm::style::Color::Rgb { r, g, b }
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
