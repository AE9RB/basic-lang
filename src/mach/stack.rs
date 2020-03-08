use crate::error;
use crate::lang::Error;

type Result<T> = std::result::Result<T, Error>;

pub struct Stack<T> {
    stack: Vec<T>,
}

impl<T> Stack<T> {
    fn overflow_check(&self) -> Result<()> {
        if self.stack.len() > u16::max_value() as usize {
            Err(error!(OutOfMemory; "STACK OVERFLOW"))
        } else {
            Ok(())
        }
    }
    fn underflow_error(&self) -> Error {
        error!(OutOfMemory; "STACK UNDERFLOW")
    }
    pub fn new() -> Stack<T> {
        Stack { stack: vec![] }
    }
    pub fn clear(&mut self) {
        self.stack.clear()
    }
    pub fn len(&self) -> usize {
        self.stack.len()
    }
    pub fn append(&mut self, other: &mut Stack<T>) -> Result<()> {
        self.stack.append(&mut other.stack);
        self.overflow_check()
    }
    pub fn push(&mut self, val: T) -> Result<()> {
        self.stack.push(val);
        self.overflow_check()
    }
    pub fn pop(&mut self) -> Result<T> {
        match self.stack.pop() {
            Some(v) => Ok(v),
            None => Err(self.underflow_error()),
        }
    }
    pub fn pop_2(&mut self) -> Result<(T, T)> {
        let one = self.pop()?;
        let two = self.pop()?;
        Ok((one, two))
    }
    pub fn pop_n(&mut self, len: usize) -> Result<Vec<T>> {
        if len > self.stack.len() {
            Err(self.underflow_error())
        } else {
            let range = (self.stack.len() - len as usize)..;
            Ok(self.stack.drain(range).collect())
        }
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<Vec<T>> for Stack<T> {
    fn from(vec: Vec<T>) -> Self {
        Stack { stack: vec }
    }
}

impl<T> Into<Vec<T>> for Stack<T> {
    fn into(self) -> Vec<T> {
        self.stack
    }
}

impl<T> std::iter::IntoIterator for Stack<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.stack.into_iter()
    }
}

impl<T> std::iter::FromIterator<T> for Stack<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Stack {
            stack: iter.into_iter().collect(),
        }
    }
}
