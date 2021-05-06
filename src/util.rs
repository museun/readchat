use unicode_width::UnicodeWidthStr;

pub fn array_iter<T: Clone, const N: usize>(
    array: [T; N],
) -> impl Iterator<Item = T> //
       + ExactSizeIterator
       + DoubleEndedIterator
       + Clone {
    std::array::IntoIter::new(array)
}

// TODO this requires unicode-segmentation
// pub fn fit_string(input: &str, max: usize) -> String {
//     assert!(max > 0, "cannot fit to an empty string");

//     if input.width() > max {
//         return input
//             .graphemes(true)
//             .take(max - 1)
//             .chain(std::iter::once("…"))
//             .collect();
//     }
//     let mut s = String::with_capacity(max);
//     s.push_str(input);
//     s.extend(std::iter::repeat(' ').take(max - input.width()));
//     s
// }

pub fn is_probably_a_uri(input: &str) -> bool {
    url::Url::parse(input).is_ok()
}

pub fn whitespace_partition(
    input: &str,
    max: usize,
    dont_split: impl Fn(&str) -> bool,
) -> Vec<String> {
    fn reset(input: &mut String, max: usize) -> String {
        std::mem::replace(input, String::with_capacity(max))
    }

    let cap = (input.width() as f64 / max as f64).round() as usize;
    let mut vec = Vec::with_capacity(cap);

    let mut budget = max;
    let mut temp = reset(&mut String::new(), max);

    for mut word in input.split_whitespace() {
        let width = word.width();
        if width < budget {
            budget -= width;
            if !temp.is_empty() {
                temp.push(' ');
                budget = budget.saturating_sub(1);
            }
            temp.push_str(word);
            continue;
        }

        if dont_split(word) {
            budget = max;
            if !temp.is_empty() {
                temp.push(' ');
            }
            temp.push_str(word);
            vec.push(reset(&mut temp, max));
            continue;
        }

        if !temp.is_empty() {
            vec.push(reset(&mut temp, max));
            budget = max;
        }

        loop {
            if word.width() <= budget {
                if temp.is_empty() {
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
                assert!(target > 0, "partition target should never underflow");
                target -= 1;
            };

            temp.push_str(left);
            vec.push(reset(&mut temp, max));
            budget = max;
            word = right;
        }
    }

    if !temp.is_empty() {
        vec.push(reset(&mut temp, max));
    }

    vec.shrink_to_fit();
    vec
}

pub fn trim_end_in_place(input: &mut String, pattern: char) {
    if input.ends_with(pattern) {
        match input.chars().rev().position(|c| c != pattern) {
            Some(n) => {
                input.truncate(input.len() - n);
                input.shrink_to_fit()
            }
            None => input.clear(),
        }
    }
}
