use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use futures_lite::Future;
use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;

pub fn partition(input: &str, max: usize) -> Vec<String> {
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

pub fn truncate_or_pad(input: &str, max: usize) -> String {
    if input.width() > max {
        input
            .graphemes(true)
            .take(max - 1)
            .chain(std::iter::once("…"))
            .collect()
    } else {
        let mut s = String::with_capacity(max);
        s.push_str(input);
        s.extend(std::iter::repeat(' ').take(max - input.width()));
        s
    }
}

#[derive(Debug)]
pub enum Select<A, B, C> {
    A(A),
    B(B),
    C(C),
}

pin_project_lite::pin_project! {
     pub struct Select3<A, B, C>{
        #[pin] a: A,
        #[pin] b: B,
        #[pin] c: C,
    }
}

impl<A, B, C> Future for Select3<A, B, C>
where
    A: Future + Send + Sync,
    A::Output: Send + Sync,

    B: Future + Send + Sync,
    B::Output: Send + Sync,

    C: Future + Send + Sync,
    C::Output: Send + Sync,
{
    type Output = Select<A::Output, B::Output, C::Output>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        macro_rules! poll {
            ($field:ident => $out:ident) => {
                if let Poll::Ready(t) = this.$field.poll(ctx) {
                    return Poll::Ready(Select::$out(t));
                }
            };
        }

        poll!(a => A);
        poll!(b => B);
        poll!(c => C);

        Poll::Pending
    }
}

pub fn select_3<A, B, C>(a: A, b: B, c: C) -> Select3<A, B, C> {
    Select3 { a, b, c }
}
