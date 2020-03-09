use super::{Address, Op, Program, Stack, Val};
use crate::error;
use crate::lang::{Error, Line, LineNumber};
use std::collections::{BTreeMap, HashMap};
type Result<T> = std::result::Result<T, Error>;

pub struct Runtime {
    source: BTreeMap<LineNumber, Line>,
    dirty: bool,
    program: Program,
    stack: Stack<Val>,
    vars: HashMap<String, Val>,
}

enum Event {
    IndirectErrors,
    DirectErrors,
    End,
}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime {
            source: BTreeMap::new(),
            dirty: false,
            program: Program::new(),
            stack: Stack::new("STACK OVERFLOW"),
            vars: HashMap::new(),
        }
    }

    pub fn enter(&mut self, line: Line) {
        if line.is_direct() {
            if self.dirty {
                self.program.clear();
                self.program
                    .compile(self.source.iter().map(|(_, line)| line));
                self.dirty = false;
            }
            self.program.compile(&line);
            match self.start() {
                Ok(event) => match event {
                    Event::IndirectErrors => {
                        let (_, errors, _) = self.program.link();
                        for error in errors {
                            println!("?{}", error);
                        }
                    }
                    Event::DirectErrors => {
                        let (_, _, errors) = self.program.link();
                        for error in errors {
                            println!("?{}", error);
                        }
                    }
                    Event::End => {}
                },
                Err(error) => {
                    println!("?{}", error);
                }
            }
        } else {
            self.source.insert(line.number(), line);
            self.stack.clear();
            self.dirty = true;
        }
    }

    fn start(&mut self) -> Result<Event> {
        let (mut pc, _, _) = self.program.link();
        self.resume(&mut pc)
    }

    fn resume(&mut self, pc: &mut Address) -> Result<Event> {
        match self.execute(pc) {
            Ok(e) => Ok(e),
            Err(e) => {
                let line_number = self
                    .program
                    .line_number_for(if *pc > 0 { *pc - 1 } else { *pc });
                Err(e.in_line_number(line_number))
            }
        }
    }

    fn execute(&mut self, pc: &mut Address) -> Result<Event> {
        let (_, indirect_errors, direct_errors) = self.program.link();
        let watermark = *pc;
        let has_indirect_errors = if !indirect_errors.is_empty() {
            self.dirty = true;
            true
        } else {
            false
        };
        if !direct_errors.is_empty() {
            return Ok(Event::DirectErrors);
        }
        loop {
            let op = match self.program.ops().get(*pc) {
                Some(v) => v,
                None => return Err(error!(InternalError; "INVALID PC ADDRESS")),
            };
            *pc += 1;
            match op {
                Op::Literal(val) => self.stack.push(val.clone())?,
                Op::Push(var_name) => {
                    self.stack.push(match self.vars.get(var_name) {
                        Some(val) => val.clone(),
                        None => {
                            if var_name.ends_with("$") {
                                Val::String("".to_string())
                            } else if var_name.ends_with("!") {
                                Val::Single(0.0)
                            } else if var_name.ends_with("#") {
                                Val::Double(0.0)
                            } else if var_name.ends_with("%") {
                                Val::Integer(0)
                            } else {
                                Val::Single(0.0)
                            }
                        }
                    })?;
                }
                Op::Run => {
                    if has_indirect_errors {
                        return Ok(Event::IndirectErrors);
                    }
                    self.stack.clear();
                    self.vars.clear();
                    *pc = 0;
                }
                Op::Jump(addr) => {
                    *pc = *addr;
                    if has_indirect_errors && *pc < watermark {
                        return Ok(Event::IndirectErrors);
                    }
                }
                Op::End => return Ok(Event::End),
                Op::Neg => self.r#neg()?,
                Op::Add => self.r#add()?,
                Op::Print => self.r#print()?,
                _ => {
                    dbg!(&op);
                    return Err(error!(InternalError; "OP NOT YET RUNNING; PANIC"));
                }
            }
        }
    }

    fn r#neg(&mut self) -> Result<()> {
        let val = self.stack.pop()?;
        self.stack.push(Val::neg(val)?)?;
        Ok(())
    }

    fn r#add(&mut self) -> Result<()> {
        let (lhs, rhs) = self.stack.pop_2()?;
        self.stack.push(Val::add(lhs, rhs)?)?;
        Ok(())
    }

    fn r#print(&mut self) -> Result<()> {
        match self.stack.pop()? {
            Val::Integer(len) => {
                for item in self.stack.pop_n(len as usize)? {
                    print!("{}", item);
                }
                Ok(())
            }
            _ => return Err(error!(InternalError; "EXPECTED VECTOR ON STACK")),
        }
    }
}
