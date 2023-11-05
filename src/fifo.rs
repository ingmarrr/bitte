#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Fifo<E: Copy> {
    fifo: Vec<E>,
}

impl<E: Copy> Fifo<E> {
    pub fn new() -> Self {
        Self { fifo: Vec::new() }
    }

    pub fn push(&mut self, e: E) {
        self.fifo.push(e);
    }

    pub fn pop(&mut self) -> Result<E, FifoError> {
        if self.fifo.is_empty() {
            return Err(FifoError::Empty);
        }
        Ok(self.fifo.remove(0))
    }

    pub fn pop_sure(&mut self) -> E {
        self.fifo.remove(0)
    }

    pub fn peek(&self) -> Result<E, FifoError> {
        if self.fifo.is_empty() {
            return Err(FifoError::Empty);
        }
        Ok(self.fifo[0])
    }

    pub fn len(&self) -> usize {
        self.fifo.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn has_some(&self) -> bool {
        !self.is_empty()
    }
}

#[derive(Debug, PartialEq)]
pub enum FifoError {
    Empty,
    Duplicate,
}
