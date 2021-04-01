use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;

pub fn partition(input: &str, max: usize) -> Vec<String> {
    let cap = (input.width() as f64 / max as f64).round() as usize;
    let mut vec = Vec::with_capacity(cap);

    let new_string = || String::with_capacity(max);

    let mut budget = max;
    let mut temp = new_string();

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
            // TODO why isn't this an std::mem::take()? why is there a closure being used?
            vec.push(std::mem::replace(&mut temp, new_string()));
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
            // TODO why isn't this an std::mem::take()? why is there a closure being used?
            vec.push(std::mem::replace(&mut temp, new_string()));
            budget = max;

            word = right;
        }
    }

    if !temp.is_empty() {
        vec.push(temp)
    }

    // we've optimistically, likely allocated more than we need. drop the rest
    vec.shrink_to_fit();
    vec
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn partition() {
        let s = "hello       world !this !is, !some, !commands. And this is a new statement?";

        for a in s.split_word_bounds() {
            eprintln!("'{}'", a.escape_debug());
        }

        // for i in std::array::IntoIter::new([10, 20, 30, 40, 50]) {
        //     eprintln!("{}", "-".repeat(i));

        //     for line in super::partition(
        //         "hello       world !this !is, !some, !commands. And this is a new statement?",
        //         i,
        //     ) {
        //         eprintln!(
        //             "{}| {: >n$}",
        //             line.escape_debug(),
        //             line.len(),
        //             n = (52usize).saturating_sub(line.len())
        //         );
        //     }

        //     eprintln!("{}", "-".repeat(i));
        // }
    }
}
