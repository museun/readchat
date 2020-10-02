use std::collections::{vec_deque::Iter, VecDeque};

pub struct Queue<T> {
    buf: VecDeque<T>,
    size: usize,
}

#[allow(dead_code)]
impl<T> Queue<T> {
    pub fn with_size(size: usize) -> Self {
        Self {
            buf: VecDeque::with_capacity(size),
            size,
        }
    }

    pub fn push(&mut self, item: T) {
        while self.buf.len() >= self.size {
            self.buf.pop_front();
        }
        self.buf.push_back(item);
    }

    pub fn remove(&mut self, index: usize) {
        self.buf.remove(index);
    }

    pub fn clear(&mut self) {
        self.buf.clear()
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn last(&self) -> Option<&T> {
        self.buf.back()
    }

    pub fn iter(&self) -> Iter<T> {
        self.buf.iter()
    }
}
