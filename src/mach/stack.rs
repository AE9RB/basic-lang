use super::Val;
use crate::error;
use crate::lang::Error;

type Result<T> = std::result::Result<T, Error>;

pub struct Stack {
    stack: Vec<Val>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack { stack: vec![] }
    }
    pub fn clear(&mut self) {
        self.stack.clear()
    }
    pub fn push(&mut self, val: Val) -> Result<()> {
        self.stack.push(val);
        if self.stack.len() > u16::max_value() as usize {
            Err(error!(OutOfMemory; "STACK OVERFLOW"))
        } else {
            Ok(())
        }
    }
    pub fn pop(&mut self) -> Result<Val> {
        match self.stack.pop() {
            Some(v) => Ok(v),
            None => Err(error!(InternalError; "STACK UNDERFLOW")),
        }
    }
    pub fn pop_2(&mut self) -> Result<(Val, Val)> {
        let one = self.pop()?;
        let two = self.pop()?;
        Ok((one, two))
    }
    pub fn pop_n(&mut self, len: usize) -> Result<Vec<Val>> {
        if len > self.stack.len() {
            Err(error!(InternalError; "STACK UNDERFLOW"))
        } else {
            let range = (self.stack.len() - len as usize)..;
            Ok(self.stack.drain(range).collect())
        }
    }
    pub fn pop_vec(&mut self) -> Result<Vec<Val>> {
        match self.pop()? {
            Val::Integer(len) => self.pop_n(len as usize),
            _ => Err(error!(InternalError; "STACK NOT VECTOR")),
        }
    }
}
