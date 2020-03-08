use crate::error;
use crate::lang::Error;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Stack<T> {
    overflow_message: &'static str,
    stack: Vec<T>,
}

impl<T> Stack<T> {
    fn overflow_check(&self) -> Result<()> {
        if self.stack.len() > u16::max_value() as usize {
            Err(error!(OutOfMemory; self.overflow_message))
        } else {
            Ok(())
        }
    }
    fn underflow_error(&self) -> Error {
        error!(InternalError; "UNDERFLOW")
    }
    pub fn new(overflow_message: &'static str) -> Stack<T> {
        Stack {
            overflow_message: overflow_message,
            stack: vec![],
        }
    }
    pub fn vec(&self) -> &Vec<T> {
        &self.stack
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.stack.get_mut(index)
    }
    pub fn clear(&mut self) {
        self.stack.clear()
    }
    pub fn drain<R>(&mut self, range: R) -> std::vec::Drain<'_, T>
    where
        R: std::ops::RangeBounds<usize>,
    {
        self.stack.drain(range)
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
