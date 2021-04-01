use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;

pub fn truncate_or_pad(input: &str, max: usize) -> String {
    if input.width() > max {
        return input
            .graphemes(true)
            .take(max - 1)
            .chain(std::iter::once("…"))
            .collect();
    }
    let mut s = String::with_capacity(max);
    s.push_str(input);
    s.extend(std::iter::repeat(' ').take(max - input.width()));
    s
}
