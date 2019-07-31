use std::borrow::Cow;
use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;

/// Split a string into parts of 'max' size
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

/// Truncate a string to a size of 'max', appending an ellipsis
pub fn truncate<'a>(input: &'a str, max: usize) -> Cow<'a, str> {
    if input.width() > max {
        let mut input = input.graphemes(true).take(max - 1).collect::<String>();
        input.push('…');
        input.into()
    } else {
        (*input).into()
    }
}
