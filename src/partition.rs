use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;

pub(crate) fn partition(input: &str, max: usize) -> Vec<String> {
    let mut vec = vec![];

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
            vec.push(std::mem::replace(&mut temp, new_string()));
            budget = max;

            word = right;
        }
    }

    if !temp.is_empty() {
        vec.push(temp)
    }
    vec
}
