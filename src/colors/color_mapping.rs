use super::{Color, Render, UserDefinedColor, COLOR_NAMES, DEFAULT_COLORS};
use crate::array_iter;
use crossterm::style::style;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Write};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorMapping {
    #[serde(flatten)]
    pub(crate) map: HashMap<String, UserDefinedColor>,
}

impl Default for ColorMapping {
    fn default() -> Self {
        let keys = array_iter(COLOR_NAMES).map(ToString::to_string);
        let values = array_iter(DEFAULT_COLORS).map(UserDefinedColor::Color);
        let map = keys.zip(values).collect();
        Self { map }
    }
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

        super::draw_table(w, headers, &*rows)
    }
}

impl ColorMapping {
    pub(crate) fn as_array(&self) -> [UserDefinedColor; COLOR_NAMES.len()] {
        array_iter(COLOR_NAMES).enumerate().fold(
            [UserDefinedColor::Color(Color::default()); COLOR_NAMES.len()],
            |mut colors, (i, name)| {
                colors[i] = self.map[name];
                colors
            },
        )
    }
}
