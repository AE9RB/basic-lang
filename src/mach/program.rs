use super::{codegen::codegen, Address, Link, Opcode, Symbol, Val};
use crate::lang::{Error, Line, LineNumber};
use std::sync::Arc;

type Result<T> = std::result::Result<T, Error>;

/// ## Program memory

#[derive(Default, Debug)]
pub struct Program {
    errors: Arc<Vec<Error>>,
    indirect_errors: Arc<Vec<Error>>,
    direct_address: Address,
    line_number: LineNumber,
    link: Link,
}

impl Program {
    pub fn error(&mut self, error: Error) {
        Arc::make_mut(&mut self.errors).push(error.in_line_number(self.line_number));
    }

    pub fn append(&mut self, link: Link) -> Result<()> {
        self.link.append(link)
    }

    pub fn get(&self, addr: Address) -> Option<Opcode> {
        self.link.get(addr).cloned()
    }

    pub fn read_data(&mut self) -> Result<Val> {
        self.link.read_data()
    }

    pub fn restore_data(&mut self, addr: Address) {
        self.link.restore_data(addr)
    }

    pub fn line_number_for(&self, op_addr: Address) -> LineNumber {
        self.link.line_number_for(op_addr)
    }

    pub fn clear(&mut self) {
        self.errors = Arc::default();
        self.indirect_errors = Arc::default();
        self.direct_address = 0;
        self.line_number = None;
        self.link.clear();
    }

    pub fn codegen<'b, T: IntoIterator<Item = &'b Line>>(&mut self, lines: T) {
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
                self.link.push_symbol(line_number as Symbol);
            } else {
                debug_assert!(!direct_seen, "Can't handle multiple direct lines.");
                direct_seen = true;
                self.link.drain(self.direct_address..);
                Arc::make_mut(&mut self.errors).clear();
            }
            let ast = match line.ast() {
                Ok(ast) => ast,
                Err(error) => {
                    Arc::make_mut(&mut self.errors).push(error);
                    continue;
                }
            };
            codegen(self, &ast);
            if self.line_number.is_none() {
                if let Err(e) = self.link.push(Opcode::End) {
                    Arc::make_mut(&mut self.errors).push(e);
                }
            }
        }
    }

    pub fn link(&mut self) -> (Address, Arc<Vec<Error>>, Arc<Vec<Error>>) {
        match self.link.last() {
            Some(Opcode::End) => {}
            _ => {
                if let Err(error) = self.link.push(Opcode::End) {
                    Arc::make_mut(&mut self.errors).push(error);
                }
            }
        };
        let mut link_errors = self.link.link();
        if self.errors.is_empty() {
            Arc::make_mut(&mut self.errors).append(&mut link_errors);
        }
        if self.direct_address == 0 {
            self.indirect_errors = std::mem::take(&mut self.errors);
            self.direct_address = self.link.len();
            self.link.set_start_of_direct(self.link.len());
        }
        (
            self.direct_address,
            Arc::clone(&self.indirect_errors),
            Arc::clone(&self.errors),
        )
    }
}
