use crate::{
    color::Color,
    color_mapping::ColorMapping,
    interactions::{ColoredTerm, Interactions},
    pair::Pair,
    user_defined_color::UserDefinedColor,
};

use crossterm::style::{style, Print};
use serde::{Deserialize, Serialize};
use std::io::Write;

pub trait Render {
    fn render<W>(&self, w: W) -> anyhow::Result<()>
    where
        W: Write + Sized;
}

pub fn show_off_colors<W>(mut w: W, config: &ColorConfig) -> anyhow::Result<()>
where
    W: Write + Sized,
{
    config.render(&mut w)
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
pub const COLOR_NAMES: [&str; 15] = [
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
pub const DEFAULT_COLORS: [Color; 15] = [
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
