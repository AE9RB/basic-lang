use super::{compile::compile, Address, Link, Opcode, Stack, Symbol, Val};
use crate::error;
use crate::lang::{Column, Error, Line, LineNumber};
use std::sync::Arc;

/// ## Program memory

#[derive(Debug)]
pub struct Program {
    ops: Stack<Opcode>,
    errors: Arc<Vec<Error>>,
    indirect_errors: Arc<Vec<Error>>,
    direct_address: Address,
    line_number: LineNumber,
    link: Link,
}

impl Default for Program {
    fn default() -> Self {
        Program {
            ops: Stack::new("PROGRAM TOO LARGE"),
            errors: Arc::default(),
            indirect_errors: Arc::default(),
            direct_address: 0,
            line_number: None,
            link: Link::new(),
        }
    }
}

impl Program {
    pub fn new() -> Program {
        Program::default()
    }

    pub fn error(&mut self, error: Error) {
        Arc::make_mut(&mut self.errors).push(error.in_line_number(self.line_number));
    }

    pub fn append(&mut self, ops: &mut Stack<Opcode>) -> Result<(), Error> {
        self.ops.append(ops)
    }

    pub fn push(&mut self, op: Opcode) -> Result<(), Error> {
        self.ops.push(op)
    }

    pub fn push_for(&mut self, col: Column, ident: String) -> Result<(), Error> {
        self.link.begin_for_loop(self.ops.len(), col, ident)?;
        self.ops.push(Opcode::For(0))
    }

    pub fn push_next(&mut self, col: Column, ident: String) -> Result<(), Error> {
        self.ops.push(Opcode::Literal(Val::String(ident.clone())))?;
        self.link.next_for_loop(self.ops.len(), col, ident)?;
        self.ops.push(Opcode::Jump(0))
    }

    pub fn push_goto(&mut self, col: Column, line_number: LineNumber) -> Result<(), Error> {
        let sym = self.link.symbol_for_line_number(line_number)?;
        self.link.link_addr_to_symbol(self.ops.len(), col, sym);
        self.ops.push(Opcode::Jump(0))
    }

    pub fn push_run(&mut self, col: Column, line_number: LineNumber) -> Result<(), Error> {
        self.ops.push(Opcode::Clear)?;
        if line_number.is_some() {
            let sym = self.link.symbol_for_line_number(line_number)?;
            self.link.link_addr_to_symbol(self.ops.len(), col, sym);
        }
        self.ops.push(Opcode::Jump(0))
    }

    pub fn get(&self, addr: Address) -> Option<&Opcode> {
        self.ops.get(addr)
    }

    pub fn line_number_for(&self, op_addr: Address) -> LineNumber {
        self.link.line_number_for(op_addr)
    }

    pub fn clear(&mut self) {
        self.ops.clear();
        self.errors = Arc::default();
        self.indirect_errors = Arc::default();
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
                if let Err(e) = self.ops.push(Opcode::End) {
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
            Some(Opcode::End) => {}
            _ => {
                if let Err(error) = self.ops.push(Opcode::End) {
                    Arc::make_mut(&mut self.errors).push(error);
                }
            }
        };
        let mut link_errors = self.link.link(&mut self.ops);
        if self.errors.is_empty() {
            Arc::make_mut(&mut self.errors).append(&mut link_errors);
        }
        if self.direct_address == 0 {
            self.indirect_errors = std::mem::take(&mut self.errors);
            self.direct_address = self.ops.len();
            self.link.set_start_of_direct(self.ops.len());
        }
        (
            self.direct_address,
            Arc::clone(&self.indirect_errors),
            Arc::clone(&self.errors),
        )
    }
}
