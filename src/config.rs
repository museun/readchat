use crate::Color;

pub enum UserDefinedColor {
    Color(Color),
    Pair { fg: Color, bg: Color },
}
