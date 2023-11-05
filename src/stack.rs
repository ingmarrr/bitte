#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Stack<const N: usize, E> {
    stack: Vec<E>,
}

impl<const N: usize, E: Copy> Stack<N, E> {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(N),
        }
    }

    pub fn push(&mut self, e: E) {
        self.stack.push(e);
    }

    pub fn pop(&mut self) -> Result<E, StackError> {
        if self.stack.is_empty() {
            return Err(StackError::Empty);
        }
        Ok(self.stack.pop().unwrap())
    }

    pub fn pop_sure(&mut self) -> E {
        self.stack.pop().unwrap()
    }

    pub fn peek(&self) -> Result<E, StackError> {
        if self.stack.is_empty() {
            return Err(StackError::Empty);
        }
        Ok(self.stack[self.stack.len() - 1])
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn has_some(&self) -> bool {
        !self.is_empty()
    }
}

#[derive(Debug, PartialEq)]
pub enum StackError {
    Full,
    Empty,
    Duplicate,
}
