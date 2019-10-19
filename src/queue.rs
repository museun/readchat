use std::collections::VecDeque;

pub struct Queue<T> {
    buf: VecDeque<T>,
    size: usize,
}

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

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> + DoubleEndedIterator {
        self.buf.iter()
    }
}
