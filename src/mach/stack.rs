use crate::error;
use crate::lang::Error;

type Result<T> = std::result::Result<T, Error>;

/// ## Stack enforced and size limited vector

pub struct Stack<T> {
    overflow_message: &'static str,
    vec: Vec<T>,
}

impl<T: std::fmt::Debug> std::fmt::Debug for Stack<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.vec)
    }
}

impl<T> Stack<T> {
    pub fn new(overflow_message: &'static str) -> Stack<T> {
        Stack {
            overflow_message,
            vec: vec![],
        }
    }
    fn max_len(&self) -> usize {
        u16::max_value() as usize
    }
    fn overflow_check(&self) -> Result<()> {
        if self.vec.len() > self.max_len() {
            Err(error!(OutOfMemory; self.overflow_message))
        } else {
            Ok(())
        }
    }
    fn underflow_error(&self) -> Error {
        error!(InternalError; "UNDERFLOW")
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.vec.get_mut(index)
    }
    pub fn clear(&mut self) {
        self.vec.clear()
    }
    pub fn drain<R>(&mut self, range: R) -> std::vec::Drain<'_, T>
    where
        R: std::ops::RangeBounds<usize>,
    {
        debug_assert!(range.end_bound() == std::ops::Bound::Unbounded);
        self.vec.drain(range)
    }
    pub fn len(&self) -> usize {
        self.vec.len()
    }
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
    pub fn is_full(&self) -> bool {
        self.vec.len() > self.max_len() - 32
    }
    pub fn last(&self) -> Option<&T> {
        self.vec.last()
    }
    pub fn get(&self, idx: usize) -> Option<&T> {
        self.vec.get(idx)
    }
    pub fn append(&mut self, other: &mut Stack<T>) -> Result<()> {
        self.vec.append(&mut other.vec);
        self.overflow_check()
    }
    pub fn push(&mut self, val: T) -> Result<()> {
        self.vec.push(val);
        self.overflow_check()
    }
    pub fn pop(&mut self) -> Result<T> {
        match self.vec.pop() {
            Some(v) => Ok(v),
            None => Err(self.underflow_error()),
        }
    }
    pub fn pop_2(&mut self) -> Result<(T, T)> {
        let two = self.pop()?;
        let one = self.pop()?;
        Ok((one, two))
    }
    pub fn pop_n(&mut self, len: usize) -> Result<Vec<T>> {
        if len > self.vec.len() {
            Err(self.underflow_error())
        } else {
            let range = (self.vec.len() - len as usize)..;
            Ok(self.vec.drain(range).collect())
        }
    }
}
