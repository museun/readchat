use std::io::Write;

mod color;
mod color_mapping;
mod config;
mod interactions;
mod pair;
mod table;
mod uniqueish_colors;
mod user_defined_color;

pub use color_mapping::ColorMapping;
pub use config::ColorConfig;

use color::Color;
use pair::Pair;
use table::draw_table;
use user_defined_color::UserDefinedColor;

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
