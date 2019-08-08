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
}

impl<'a, T> IntoIterator for &'a Queue<T> {
    type Item = &'a T;
    type IntoIter = QueueIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        QueueIter {
            queue: self,
            pos: 0,
        }
    }
}

pub struct QueueIter<'a, T> {
    queue: &'a Queue<T>,
    pos: usize,
}

impl<'a, T> Iterator for QueueIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.buf.get(self.pos).and_then(|d| {
            self.pos += 1;
            Some(d)
        })
    }
}
