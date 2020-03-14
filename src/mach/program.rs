use super::{compile::compile, Address, Link, Op, Stack, Symbol};
use crate::error;
use crate::lang::{Column, Error, Line, LineNumber};
use std::sync::Arc;

/// ## Program memory

#[derive(Debug)]
pub struct Program {
    ops: Stack<Op>,
    errors: Arc<Vec<Error>>,
    indirect_errors: Arc<Vec<Error>>,
    direct_address: Address,
    line_number: LineNumber,
    pub link: Link,
}

impl Program {
    pub fn new() -> Program {
        Program {
            ops: Stack::new("PROGRAM TOO LARGE"),
            errors: Arc::new(vec![]),
            indirect_errors: Arc::new(vec![]),
            direct_address: 0,
            line_number: None,
            link: Link::new(),
        }
    }

    pub fn error(&mut self, error: Error) {
        Arc::make_mut(&mut self.errors).push(error.in_line_number(self.line_number));
    }

    pub fn append(&mut self, ops: &mut Stack<Op>) -> Result<(), Error> {
        self.ops.append(ops)
    }

    pub fn push(&mut self, op: Op) -> Result<(), Error> {
        self.ops.push(op)
    }

    pub fn push_jump_to_line(
        &mut self,
        col: &Column,
        line_number: &LineNumber,
    ) -> Result<(), Error> {
        let sym = self.link.symbol_for_line_number(line_number)?;
        self.link.link_addr_to_symbol(self.ops.len(), col, sym);
        self.ops.push(Op::Jump(0))
    }

    pub fn get(&self, addr: Address) -> Option<&Op> {
        self.ops.get(addr)
    }

    pub fn line_number_for(&self, op_addr: Address) -> LineNumber {
        if self.direct_address == 0 || op_addr < self.direct_address {
            self.link.line_number_for(op_addr)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.ops.clear();
        self.errors = Arc::new(vec![]);
        self.indirect_errors = Arc::new(vec![]);
        self.direct_address = 0;
        self.line_number = None;
        self.link.clear();
    }

    pub fn compile<'b, T: IntoIterator<Item = &'b Line>>(&mut self, lines: T) {
        let is_out_of_mem = |this: &Self| this.ops.len() > Address::max_value() as usize;
        if is_out_of_mem(self) {
            return;
        }
        let mut direct_seen = false;
        for line in lines {
            if let Some(line_number) = line.number() {
                debug_assert!(
                    self.direct_address == 0,
                    "Can't go back to indirect mode without clear()."
                );
                if let Some(self_line_number) = self.line_number {
                    debug_assert!(line_number > self_line_number, "Lines out of order.");
                }
            } else {
                self.link();
            }
            self.line_number = line.number();
            if let Some(line_number) = self.line_number {
                self.link.insert(line_number as Symbol, self.ops.len());
            } else {
                debug_assert!(!direct_seen, "Can't handle multiple direct lines.");
                direct_seen = true;
                self.ops.drain(self.direct_address..);
                Arc::make_mut(&mut self.errors).clear();
            }
            let ast = match line.ast() {
                Ok(ast) => ast,
                Err(error) => {
                    Arc::make_mut(&mut self.errors).push(error);
                    continue;
                }
            };
            compile(self, &ast);
            if self.line_number.is_none() {
                if let Err(e) = self.ops.push(Op::End) {
                    Arc::make_mut(&mut self.errors).push(e);
                }
            }
            if is_out_of_mem(self) {
                Arc::make_mut(&mut self.errors).push(error!(OutOfMemory));
                return;
            }
        }
    }

    pub fn link(&mut self) -> (Address, Arc<Vec<Error>>, Arc<Vec<Error>>) {
        match self.ops.last() {
            Some(Op::End) => {}
            _ => {
                if let Err(error) = self.ops.push(Op::End) {
                    Arc::make_mut(&mut self.errors).push(error);
                }
            }
        };
        Arc::make_mut(&mut self.errors).append(&mut self.link.link(&mut self.ops));
        if self.direct_address == 0 {
            self.indirect_errors = std::mem::take(&mut self.errors);
            self.direct_address = self.ops.len();
        }
        (
            self.direct_address,
            Arc::clone(&self.indirect_errors),
            Arc::clone(&self.errors),
        )
    }
}
