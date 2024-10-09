#[derive(Clone)]
pub struct Stack<T: Clone> {
    data: Vec<T>,
}

impl<T: Clone> Stack<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }

    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }

    pub fn get(&self, offset: usize) -> Option<&T> {
        if offset < self.data.len() {
            Some(&self.data[self.data.len() - 1 - offset])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, offset: usize) -> Option<&mut T> {
        if offset < self.data.len() {
            let len = self.data.len();
            Some(&mut self.data[len - 1 - offset])
        } else {
            None
        }
    }

    pub fn drop(&mut self, length: usize) {
        if length <= self.data.len() {
            self.data.truncate(self.data.len() - length);
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    pub fn change(&mut self, first: usize, second: usize) {
        let len = self.data.len();
        if first < len && second < len {
            self.data.swap(len - 1 - first, len - 1 - second);
        }
    }

    pub fn pair(&self) -> Option<(&T, &T)> {
        let len = self.data.len();
        if len >= 2 {
            Some((self.get(1).unwrap(), self.get(0).unwrap()))
        } else {
            None
        }
    }

    pub fn pop_pair(&mut self) -> Option<(T, T)> {
        let second = self.pop()?;
        let first = self.pop()?;
        Some((first, second))
    }

    pub fn get_vector(&self) -> Vec<T> {
        let mut vector = self.data.clone();
        vector.reverse();
        vector
    }
}
