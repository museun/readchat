use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;

pub fn partition(input: &str, max: usize) -> Vec<String> {
    let mut vec = vec![];

    let mut budget = max;
    let mut temp = String::with_capacity(max);

    for mut word in input.split_word_bounds() {
        if temp.is_empty() && word.chars().all(char::is_whitespace) {
            continue;
        }

        let width = word.width();
        if width < budget {
            budget -= width;
            temp.push_str(word);
            continue;
        }

        if !temp.is_empty() {
            vec.push(std::mem::replace(&mut temp, String::with_capacity(max)));
            budget = max;
        }

        loop {
            if word.width() <= budget {
                if !temp.is_empty() || !word.chars().all(char::is_whitespace) {
                    temp.push_str(word);
                }
                budget -= word.width();
                break;
            }

            let mut target = budget;
            let (left, right) = loop {
                if word.is_char_boundary(target) {
                    break word.split_at(target);
                }
                target -= 1; // this should never underflow
            };

            temp.push_str(left);
            vec.push(std::mem::replace(&mut temp, String::with_capacity(max)));
            budget = max;

            word = right;
        }
    }

    if !temp.is_empty() {
        vec.push(temp)
    }
    vec
}

pub fn truncate_or_pad(input: &str, max: usize) -> String {
    match input.width() > max {
        true => input
            .graphemes(true)
            .take(max - 1)
            .chain(std::iter::once("…"))
            .collect(),
        false => {
            let mut s = String::with_capacity(max);
            s.push_str(input);
            s.extend(std::iter::repeat(' ').take(max - input.width()));
            s
        }
    }
}

use twitchchat::color::RGB;

pub fn normalize_color(RGB(r, g, b): RGB, conf: f64) -> RGB {
    use std::cmp::{max, min};

    fn to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
        let max = max(max(r, g), b);
        let min = min(min(r, g), b);

        let (r, g, b) = (
            f64::from(r) / 255.0,
            f64::from(g) / 255.0,
            f64::from(b) / 255.0,
        );

        let (min, max) = (f64::from(min) / 255.0, f64::from(max) / 255.0);
        let lum = (max + min) / 2.0;
        let delta = max - min;
        if delta == 0.0 {
            return (0.0, 0.0, ((lum * 100.0).round() / 100.0) * 100.0);
        }

        let sat = if lum < 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };

        let norm_r = (((max - r) / 6.0) + (delta / 2.0)) / delta;
        let norm_g = (((max - g) / 6.0) + (delta / 2.0)) / delta;
        let norm_b = (((max - b) / 6.0) + (delta / 2.0)) / delta;

        let hue = match match max {
            // TODO compare against the floating point epsilon
            x if (x - r).abs() < 0.001 => norm_b - norm_g,
            x if (x - g).abs() < 0.001 => (1.0 / 3.0) + norm_r - norm_b,
            _ => (2.0 / 3.0) + norm_g + norm_r,
        } {
            h if h < 0.0 => h + 1.0,
            h if h > 1.0 => h - 1.0,
            h => h,
        };

        let hue = (hue * 360.0 * 100.0).round() / 100.0;
        let sat = ((sat * 100.0).round() / 100.0) * 100.0;
        let lum = ((lum * 100.0).round() / 100.0) * 100.0;

        (hue, sat, lum)
    }

    fn to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r, g, b) = match () {
            _ if 0.0 <= h && h <= 60.0 => (c, x, 0.0),
            _ if 60.0 <= h && h <= 120.0 => (x, c, 0.0),
            _ if 120.0 <= h && h <= 180.0 => (0.0, c, x),
            _ if 180.0 <= h && h <= 240.0 => (0.0, x, c),
            _ if 240.0 <= h && h <= 300.0 => (x, 0.0, c),
            _ if 300.0 <= h && h <= 360.0 => (c, 0.0, x),
            _ => unreachable!(),
        };

        (
            ((r + m) * 255.0).round() as _,
            ((g + m) * 255.0).round() as _,
            ((b + m) * 255.0).round() as _,
        )
    }

    let (h, s, mut l) = to_hsl(r, g, b);
    if s == 0.0 && l == 0.0 {
        // this is grey
        return RGB(
            (l * r as f64).round() as _,
            (l * g as f64).round() as _,
            (l * b as f64).round() as _,
        );
    }
    if l <= conf {
        l = (l + conf).min(100.0);
    }
    let (r, g, b) = to_rgb(h, s, l);
    RGB(r, g, b)
}
