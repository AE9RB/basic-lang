use super::{Address, Function, Listing, Opcode, Operation, Program, Stack, Val, Var};
use crate::error;
use crate::lang::{Error, Line, LineNumber};
use std::convert::TryFrom;
use std::ops::Range;
use std::sync::Arc;

type Result<T> = std::result::Result<T, Error>;

const INTRO: &str = "64K BASIC";
const MAX_LINE_LEN: usize = 1024;

/// ## Virtual machine

pub struct Runtime {
    prompt: String,
    source: Listing,
    dirty: bool,
    program: Program,
    pc: Address,
    entry_address: Address,
    stack: RuntimeStack,
    vars: Var,
    state: State,
    cont: State,
    cont_pc: Address,
    print_col: usize,
}

/// ## Events for the user interface

pub enum Event {
    Errors(Arc<Vec<Error>>),
    Input(String, bool),
    Print(String),
    List((String, Vec<Range<usize>>)),
    Running,
    Stopped,
}

#[derive(Debug)]
enum State {
    Intro,
    Stopped,
    Listing(Range<LineNumber>),
    RuntimeError(Error),
    Running,
    Input,
    InputRedo,
    InputRunning,
    Interrupt,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime {
            prompt: "READY.".to_owned(),
            source: Listing::default(),
            dirty: false,
            program: Program::default(),
            pc: 0,
            entry_address: 1,
            stack: Stack::new("STACK OVERFLOW"),
            vars: Var::new(),
            state: State::Intro,
            cont: State::Stopped,
            cont_pc: 0,
            print_col: 0,
        }
    }
}

impl Runtime {
    pub fn new(prompt: &str) -> Runtime {
        let mut rt = Runtime::default();
        rt.prompt = prompt.to_owned();
        rt
    }

    pub fn get_listing(&self) -> Listing {
        self.source.clone()
    }

    /// Enters a line of BASIC or INPUT.
    /// Returns true if good candidate for history.
    pub fn enter(&mut self, string: &str) -> bool {
        if let State::Input = self.state {
            self.enter_input(string);
            self.print_col = 0;
            return true;
        }
        debug_assert!(matches!(self.state, State::Stopped | State::Intro));
        if string.len() > MAX_LINE_LEN {
            self.state = State::RuntimeError(error!(OutOfMemory));
            return false;
        }
        let line = Line::new(string);
        if line.is_direct() {
            if line.is_empty() {
                false
            } else {
                self.enter_direct(line);
                true
            }
        } else {
            self.enter_indirect(line);
            false
        }
    }

    fn enter_direct(&mut self, line: Line) {
        if self.dirty {
            self.program.clear();
            self.program.compile(self.source.lines());
            self.dirty = false;
        }
        self.program.compile(&line);
        let (pc, indirect_errors, direct_errors) = self.program.link();
        self.pc = pc;
        self.entry_address = pc;
        self.source.indirect_errors = indirect_errors;
        self.source.direct_errors = direct_errors;
        self.state = State::Running;
    }

    fn enter_indirect(&mut self, line: Line) {
        self.cont = State::Stopped;
        self.stack.clear();
        if line.is_empty() {
            self.dirty = self.source.remove(line.number()).is_some();
        } else {
            self.source.insert(line);
            self.dirty = true;
        }
    }

    fn enter_input(&mut self, string: &str) {
        if string.len() > MAX_LINE_LEN {
            self.state = State::InputRedo;
            return;
        }
        if self.do_input(string).is_err() {
            self.state = State::Stopped;
            self.cont = State::Stopped;
            debug_assert!(false, "BAD INPUT STACK");
        }
    }

    fn do_input(&mut self, string: &str) -> Result<()> {
        let len = match self.stack.last() {
            Some(Val::Integer(n)) => *n,
            _ => return Err(error!(InternalError)),
        };
        let mut vv: Vec<Val> = vec![];
        if len <= 1 {
            vv.push(Val::String(string.to_string()));
        } else {
            let mut start: usize = 0;
            let mut in_quote = false;
            for (i, ch) in string.char_indices() {
                match ch {
                    '"' => {
                        in_quote = !in_quote;
                    }
                    ',' if !in_quote => {
                        vv.push(Val::String(string[start..i].to_string()));
                        start = i + 1;
                    }
                    _ => {}
                }
            }
            vv.push(Val::String(string[start..].to_string()));
            if len as usize != vv.len() {
                self.state = State::InputRedo;
                return Ok(());
            }
        }
        self.stack.push(Val::Return(self.pc))?;
        while let Some(v) = vv.pop() {
            self.stack.push(v)?;
        }
        self.state = State::InputRunning;
        Ok(())
    }

    pub fn interrupt(&mut self) {
        self.cont = State::Interrupt;
        std::mem::swap(&mut self.state, &mut self.cont);
        self.cont_pc = self.pc;
        if self.pc >= self.entry_address {
            self.cont = State::Stopped;
            self.stack.clear();
        }
    }

    fn ready_prompt(&mut self) -> Option<Event> {
        if self.entry_address != 0 {
            self.entry_address = 0;
            let mut s = String::new();
            if self.print_col > 0 {
                s.push('\n');
                self.print_col = 0;
            }
            s.push_str(&self.prompt);
            s.push('\n');
            return Some(Event::Print(s));
        };
        None
    }

    pub fn execute(&mut self, iterations: usize) -> Event {
        fn line_number(this: &Runtime) -> LineNumber {
            let mut pc = this.pc;
            if pc > 0 {
                pc -= 1
            }
            this.program.line_number_for(pc)
        }
        match &self.state {
            State::Intro => {
                self.state = State::Stopped;
                let mut s = INTRO.to_string();
                if let Some(version) = option_env!("CARGO_PKG_VERSION") {
                    s.push(' ');
                    s.push_str(version);
                }
                #[cfg(debug_assertions)]
                s.push_str("+debug");
                s.push('\n');
                return Event::Print(s);
            }
            State::Stopped => match self.ready_prompt() {
                Some(e) => return e,
                None => return Event::Stopped,
            },
            State::Interrupt => {
                self.state = State::RuntimeError(error!(Break, line_number(&self)));
            }
            State::Listing(range) => {
                let mut range = range.clone();
                if let Some((string, columns)) = self.source.list_line(&mut range) {
                    self.state = State::Listing(range);
                    return Event::List((string, columns));
                } else {
                    self.state = State::Running;
                }
            }
            State::Input => match self.execute_input() {
                Ok(r) => return r,
                Err(e) => self.state = State::RuntimeError(e.in_line_number(line_number(&self))),
            },
            State::InputRedo => {
                self.state = State::Input;
                return Event::Errors(Arc::new(vec![error!(RedoFromStart)]));
            }
            State::InputRunning | State::Running => {
                if !self.source.direct_errors.is_empty() {
                    self.state = State::Stopped;
                    return Event::Errors(Arc::clone(&self.source.direct_errors));
                }
            }
            State::RuntimeError(_) => {}
        }
        if let State::RuntimeError(_) = self.state {
            if self.print_col > 0 {
                self.print_col = 0;
                return Event::Print('\n'.to_string());
            }
            let mut state = State::Stopped;
            std::mem::swap(&mut self.state, &mut state);
            if let State::RuntimeError(e) = state {
                return Event::Errors(Arc::new(vec![e]));
            }
        }
        debug_assert!(matches!(self.state, State::Running | State::InputRunning));
        match self.execute_loop(iterations) {
            Ok(event) => {
                if let State::Stopped = self.state {
                    match event {
                        Event::Stopped => match self.ready_prompt() {
                            Some(e) => e,
                            None => event,
                        },
                        _ => event,
                    }
                } else {
                    event
                }
            }
            Err(error) => {
                if let State::InputRunning = self.state {
                    loop {
                        match self.stack.pop() {
                            Err(_) => break,
                            Ok(Val::Return(addr)) => {
                                self.pc = addr;
                                break;
                            }
                            _ => continue,
                        }
                    }
                    self.state = State::InputRedo;
                } else {
                    self.cont = State::RuntimeError(error.in_line_number(line_number(&self)));
                    std::mem::swap(&mut self.cont, &mut self.state);
                    self.cont_pc = self.pc;
                    if self.pc >= self.entry_address || self.stack.is_full() {
                        self.stack.clear();
                        self.cont = State::Stopped;
                    }
                }
                Event::Running
            }
        }
    }

    fn execute_input(&mut self) -> Result<Event> {
        let len = self.stack.pop()?;
        let caps = self.stack.pop()?;
        let mut prompt = match self.stack.last() {
            Some(Val::String(s)) => s.clone(),
            _ => return Err(error!(InternalError)),
        };
        prompt.push('?');
        prompt.push(' ');
        let is_caps = match caps {
            Val::Integer(i) if i == 0 => false,
            _ => true,
        };
        self.stack.push(caps)?;
        self.stack.push(len)?;
        Ok(Event::Input(prompt, is_caps))
    }

    #[allow(clippy::cognitive_complexity)]
    fn execute_loop(&mut self, iterations: usize) -> Result<Event> {
        let has_indirect_errors = !self.source.indirect_errors.is_empty();
        for _ in 0..iterations {
            let op = match self.program.get(self.pc) {
                Some(v) => v,
                None => return Err(error!(InternalError; "INVALID PC ADDRESS")),
            };
            self.pc += 1;
            match op {
                Opcode::Literal(val) => self.stack.push(val.clone())?,
                Opcode::Pop(var_name) => self.vars.store(var_name, self.stack.pop()?)?,
                Opcode::Push(var_name) => self.stack.push(self.vars.fetch(var_name))?,
                Opcode::PopArr(var_name) => {
                    let vec = self.stack.pop_vec()?;
                    let val = self.stack.pop()?;
                    self.vars.store_array(var_name, vec, val)?;
                }
                Opcode::PushArr(var_name) => {
                    let vec = self.stack.pop_vec()?;
                    let val = self.vars.fetch_array(var_name, vec)?;
                    self.stack.push(val)?;
                }
                Opcode::DimArr(var_name) => {
                    let vec = self.stack.pop_vec()?;
                    self.vars.dimension_array(var_name, vec)?;
                }
                Opcode::For(addr) => {
                    let addr = *addr;
                    self.r#for(addr)?;
                }
                Opcode::IfNot(addr) => {
                    if match self.stack.pop()? {
                        Val::Return(_) | Val::String(_) => return Err(error!(TypeMismatch)),
                        Val::Integer(n) => n == 0,
                        Val::Single(n) => n == 0.0,
                        Val::Double(n) => n == 0.0,
                    } {
                        self.pc = *addr;
                    }
                }
                Opcode::Jump(addr) => {
                    self.pc = *addr;
                    if has_indirect_errors && self.pc < self.entry_address {
                        self.state = State::Stopped;
                        return Ok(Event::Errors(Arc::clone(&self.source.indirect_errors)));
                    }
                }
                Opcode::Return => {
                    return Err(error!(InternalError; "'RETURN' NOT YET IMPLEMENTED; PANIC"))
                }
                Opcode::Clear => {
                    self.stack.clear();
                    self.vars.clear();
                }
                Opcode::Cont => {
                    if let State::Stopped = self.cont {
                        return Err(error!(CantContinue));
                    }
                    if let State::Running = self.state {
                        self.state = State::Stopped;
                        std::mem::swap(&mut self.cont, &mut self.state);
                        self.pc = self.cont_pc;
                    } else {
                        return Err(error!(CantContinue));
                    }
                    if let State::Running = self.state {
                    } else {
                        return Ok(Event::Running);
                    }
                }
                Opcode::Input(var_name) => {
                    if let State::Running = self.state {
                        self.state = State::Input;
                        self.pc -= 1;
                        return Ok(Event::Running);
                    } else if let State::InputRunning = self.state {
                        if var_name.is_empty() {
                            self.state = State::Running;
                            self.stack.pop()?;
                            self.stack.pop()?;
                            self.stack.pop()?;
                            self.stack.pop()?;
                        } else {
                            let mut pop = self.stack.pop()?;
                            if let Val::String(v) = pop {
                                let mut v = v.trim();
                                if var_name.ends_with('$') {
                                    if v.len() >= 2 && v.starts_with('"') && v.ends_with('"') {
                                        v = &v[1..v.len() - 1];
                                    }
                                    pop = Val::String(v.to_string());
                                } else if v.is_empty() {
                                    pop = Val::Integer(0);
                                } else {
                                    pop = Val::from(v);
                                }
                                self.stack.push(pop)?;
                            } else {
                                self.stack.push(pop)?;
                                debug_assert!(false, "input stack corrupt");
                            }
                        }
                    } else {
                        return Err(error!(InternalError));
                    }
                }
                Opcode::List => return self.r#list(),
                Opcode::End => return self.r#end(),
                Opcode::Print => return self.r#print(),
                Opcode::Stop => {
                    self.r#end()?;
                    return Err(error!(Break));
                }

                Opcode::Neg => self.stack.pop_1_push(&Operation::negate)?,
                Opcode::Exp => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Mul => self.stack.pop_2_push(&Operation::multiply)?,
                Opcode::Div => self.stack.pop_2_push(&Operation::divide)?,
                Opcode::DivInt => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Mod => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Add => self.stack.pop_2_push(&Operation::sum)?,
                Opcode::Sub => self.stack.pop_2_push(&Operation::subtract)?,
                Opcode::Eq => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::NotEq => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Lt => self.stack.pop_2_push(&Operation::less)?,
                Opcode::LtEq => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Gt => self.stack.pop_2_push(&Operation::greater)?,
                Opcode::GtEq => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Not => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::And => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Or => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Xor => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Imp => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Eqv => self.stack.pop_2_push(&Operation::unimplemented)?,

                Opcode::Cos => self.stack.pop_1_push(&Function::cos)?,
                Opcode::Sin => self.stack.pop_1_push(&Function::sin)?,
                Opcode::Tab => {
                    let val = self.stack.pop()?;
                    self.stack.push(Function::tab(val, self.print_col)?)?;
                }
            }
        }
        Ok(Event::Running)
    }

    fn r#for(&mut self, addr: Address) -> Result<()> {
        loop {
            if self.stack.len() < 4 {
                break;
            }
            let (first_iter, next_name) = match self.stack.pop()? {
                Val::String(s) => (false, s),
                _ => (true, "".to_string()),
            };
            let var_name_val = self.stack.pop()?;
            let to_val = self.stack.pop()?;
            let step_val = self.stack.pop()?;
            if let Val::String(var_name) = var_name_val {
                if !next_name.is_empty() && var_name != next_name {
                    self.stack.push(Val::String(next_name))?;
                    continue;
                }
                let mut current = self.vars.fetch(&var_name);
                if !first_iter {
                    current = Operation::sum(current, step_val.clone())?;
                    self.vars.store(&var_name, current.clone())?;
                }
                if let Ok(step) = f64::try_from(step_val.clone()) {
                    let done = Val::Integer(-1)
                        == if step < 0.0 {
                            Operation::less(current, to_val.clone())?
                        } else {
                            Operation::less(to_val.clone(), current)?
                        };
                    if done {
                        self.pc = addr;
                    } else {
                        self.stack.push(step_val)?;
                        self.stack.push(to_val)?;
                        self.stack.push(Val::String(var_name))?;
                    }
                    return Ok(());
                }
            }
            break;
        }
        Err(error!(NextWithoutFor; "MISSING STACK FRAME"))
    }

    fn r#list(&mut self) -> Result<Event> {
        let (from, to) = self.stack.pop_2()?;
        let from = LineNumber::try_from(from)?;
        let to = LineNumber::try_from(to)?;
        if to < from {
            return Err(error!(UndefinedLine; "INVALID RANGE"));
        }
        self.state = State::Listing(from..to);
        Ok(Event::Running)
    }

    fn r#end(&mut self) -> Result<Event> {
        self.cont = State::Stopped;
        std::mem::swap(&mut self.cont, &mut self.state);
        self.cont_pc = self.pc;
        if self.pc >= self.entry_address {
            self.cont = State::Stopped;
            self.stack.clear();
        }
        Ok(Event::Stopped)
    }

    fn r#print(&mut self) -> Result<Event> {
        let mut s = String::new();
        let item = self.stack.pop()?;
        let val_str = format!("{}", item);
        for ch in val_str.chars() {
            s.push(ch);
            match ch {
                '\n' => self.print_col = 0,
                _ => self.print_col += 1,
            }
        }
        Ok(Event::Print(s))
    }
}

type RuntimeStack = Stack<Val>;

trait RuntimeStackTrait<T> {
    fn pop_1_push<F: Fn(Val) -> Result<Val>>(&mut self, func: &F) -> Result<()>;
    fn pop_2_push<F: Fn(Val, Val) -> Result<Val>>(&mut self, func: &F) -> Result<()>;
    fn pop_vec(&mut self) -> Result<Vec<Val>>;
}

impl RuntimeStackTrait<Val> for RuntimeStack {
    fn pop_1_push<F: Fn(Val) -> Result<Val>>(&mut self, func: &F) -> Result<()> {
        let val = self.pop()?;
        self.push(func(val)?)?;
        Ok(())
    }
    fn pop_2_push<F: Fn(Val, Val) -> Result<Val>>(&mut self, func: &F) -> Result<()> {
        let (val1, val2) = self.pop_2()?;
        self.push(func(val1, val2)?)?;
        Ok(())
    }
    fn pop_vec(&mut self) -> Result<Vec<Val>> {
        if let Val::Integer(n) = self.pop()? {
            self.pop_n(n as usize)
        } else {
            Err(error!(InternalError; "NO VECTOR ON STACK"))
        }
    }
}
