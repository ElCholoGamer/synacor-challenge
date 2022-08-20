#[derive(Debug)]
pub struct LimitedQueue<T> {
    contents: Vec<T>
}

impl<T> LimitedQueue<T> {
    pub fn new(max_length: usize) -> Self {
        Self {
            contents: Vec::with_capacity(max_length),
        }
    }

    pub fn push(&mut self, val: T) {
        if self.contents.len() == self.contents.capacity() {
            self.contents.drain(..1);
        }

        self.contents.push(val);
    }

    pub fn contents(&self) -> &Vec<T> { &self.contents }

    pub fn peek_last(&self) -> Option<&T> { self.contents.last() }

    pub fn len(&self) -> usize { self.contents.len() }
}
