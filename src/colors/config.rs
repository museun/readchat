use std::io::Write;

use super::{
    color::Color, color_mapping::ColorMapping, interactions::ColoredTerm,
    interactions::Interactions, pair::Pair, user_defined_color::UserDefinedColor, Render,
};
use crossterm::style::{style, Print};
use serde::{Deserialize, Serialize};

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
