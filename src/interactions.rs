use crate::{colors::Render, user_defined_color::UserDefinedColor};
use crossterm::style::{style, Print};
use serde::{Deserialize, Serialize};
use std::io::Write;

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
        crate::table::draw_table(&mut w, headers, &*rows)?;
        crossterm::queue!(w, Print("\n"))?;

        crossterm::queue!(
            w,
            Print(style("mention").with(crossterm::style::Color::Yellow)),
            Print(":\n")
        )?;
        let rows = make_rows(&self.mention);
        crate::table::draw_table(&mut w, headers, &*rows)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColoredTerm {
    pub term: String,
    pub color: UserDefinedColor,
    // TODO: should there be a partial / fuzzy option?
    pub case_insensitive: bool, // exact?
}
