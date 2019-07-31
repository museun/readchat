use std::collections::VecDeque;

/// Bounded queue
pub struct Queue<T> {
    buf: VecDeque<T>,
    size: usize,
}

impl<T> Queue<T> {
    /// Make a new bounded queue with a max `size`
    pub fn with_size(size: usize) -> Self {
        Self {
            buf: VecDeque::with_capacity(size),
            size,
        }
    }
    /// Push an element onto the back of the queue (removing any overflow from the front)
    pub fn push(&mut self, item: T) {
        while self.buf.len() >= self.size {
            self.buf.pop_front();
        }
        self.buf.push_back(item);
    }
    /// Returns the length of the queue
    pub fn len(&self) -> usize {
        self.buf.len()
    }
    /// Gets the last 'n' elements from the queue
    pub fn view_last<'a>(&'a self, n: usize) -> impl Iterator<Item = &'a T> {
        let max = self.len();
        let delta = max.saturating_sub(n);
        self.buf.iter().skip(delta)
    }
}
