use super::{Address, Function, Listing, Opcode, Operation, Program, Stack, Val, Var};
use crate::error;
use crate::lang::{Error, Line, LineNumber};
use std::convert::TryFrom;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

type Result<T> = std::result::Result<T, Error>;

const INTRO: &str = "64K BASIC";
const PROMPT: &str = "READY.";
const MAX_LINE_LEN: usize = 1024;

/// ## Virtual machine

pub struct Runtime {
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
    rand: (u32, u32, u32),
}

/// ## Events for the user interface

#[derive(Debug)]
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
            rand: (1, 1, 1),
        }
    }
}

impl Runtime {
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
            vv.push(Val::String(string.into()));
        } else {
            let mut start: usize = 0;
            let mut in_quote = false;
            for (i, ch) in string.char_indices() {
                match ch {
                    '"' => {
                        in_quote = !in_quote;
                    }
                    ',' if !in_quote => {
                        vv.push(Val::String(string[start..i].into()));
                        start = i + 1;
                    }
                    _ => {}
                }
            }
            vv.push(Val::String(string[start..].into()));
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

    fn ready_prompt(&mut self) -> Option<Event> {
        if self.entry_address != 0 {
            self.entry_address = 0;
            let mut s = String::new();
            if self.print_col > 0 {
                s.push('\n');
                self.print_col = 0;
            }
            s.push_str(PROMPT);
            s.push('\n');
            return Some(Event::Print(s));
        };
        None
    }

    /// Obtain a thread-safe Listing useful for line completion.
    pub fn get_listing(&self) -> Listing {
        self.source.clone()
    }

    /// Interrupt the program. Displays `BREAK` error.
    pub fn interrupt(&mut self) {
        self.cont = State::Interrupt;
        std::mem::swap(&mut self.state, &mut self.cont);
        self.cont_pc = self.pc;
        if self.pc >= self.entry_address {
            self.cont = State::Stopped;
            self.stack.clear();
        }
    }

    /// Use a large number for iterations but not so much
    /// that interrupts aren't responsive.
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
            Some(Val::String(s)) => s.to_string(),
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
        self.print_col += prompt.len();
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
                Opcode::Pop(var_name) => self.vars.store(&var_name, self.stack.pop()?)?,
                Opcode::Push(var_name) => self.stack.push(self.vars.fetch(&var_name))?,
                Opcode::PopArr(var_name) => {
                    let vec = self.stack.pop_vec()?;
                    let val = self.stack.pop()?;
                    self.vars.store_array(&var_name, vec, val)?;
                }
                Opcode::PushArr(var_name) => {
                    let vec = self.stack.pop_vec()?;
                    let val = self.vars.fetch_array(&var_name, vec)?;
                    self.stack.push(val)?;
                }
                Opcode::DimArr(var_name) => {
                    let vec = self.stack.pop_vec()?;
                    self.vars.dimension_array(&var_name, vec)?;
                }
                Opcode::IfNot(addr) => {
                    if match self.stack.pop()? {
                        Val::Return(_) | Val::String(_) | Val::Next(_) => {
                            return Err(error!(TypeMismatch))
                        }
                        Val::Integer(n) => n == 0,
                        Val::Single(n) => n == 0.0,
                        Val::Double(n) => n == 0.0,
                    } {
                        self.pc = addr;
                    }
                }
                Opcode::Jump(addr) => {
                    self.pc = addr;
                    if has_indirect_errors && self.pc < self.entry_address {
                        self.state = State::Stopped;
                        return Ok(Event::Errors(Arc::clone(&self.source.indirect_errors)));
                    }
                }
                Opcode::Return => loop {
                    match self.stack.pop() {
                        Ok(Val::Return(addr)) => {
                            self.pc = addr;
                            break;
                        }
                        Ok(_) => continue,
                        Err(_) => return Err(error!(ReturnWithoutGosub)),
                    }
                },

                Opcode::Clear => self.r#clear()?,
                Opcode::Cont => {
                    if let Some(event) = self.r#cont()? {
                        return Ok(event);
                    }
                }
                Opcode::End => return self.r#end(),
                Opcode::Input(var_name) => {
                    if let Some(event) = self.r#input(var_name)? {
                        return Ok(event);
                    }
                }
                Opcode::List => return self.r#list(),
                Opcode::New => return self.r#new_(),
                Opcode::On => self.r#on()?,
                Opcode::Next(var_name) => self.r#next(var_name)?,
                Opcode::Print => return self.r#print(),
                Opcode::Stop => return Err(error!(Break)),

                Opcode::Neg => self.stack.pop_1_push(&Operation::negate)?,
                Opcode::Exp => self.stack.pop_2_push(&Operation::exponentiation)?,
                Opcode::Mul => self.stack.pop_2_push(&Operation::multiply)?,
                Opcode::Div => self.stack.pop_2_push(&Operation::divide)?,
                Opcode::DivInt => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Mod => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Add => self.stack.pop_2_push(&Operation::sum)?,
                Opcode::Sub => self.stack.pop_2_push(&Operation::subtract)?,
                Opcode::Eq => self.stack.pop_2_push(&Operation::equal)?,
                Opcode::NotEq => self.stack.pop_2_push(&Operation::not_equal)?,
                Opcode::Lt => self.stack.pop_2_push(&Operation::less)?,
                Opcode::LtEq => self.stack.pop_2_push(&Operation::less_equal)?,
                Opcode::Gt => self.stack.pop_2_push(&Operation::greater)?,
                Opcode::GtEq => self.stack.pop_2_push(&Operation::greater_equal)?,
                Opcode::Not => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::And => self.stack.pop_2_push(&Operation::and)?,
                Opcode::Or => self.stack.pop_2_push(&Operation::or)?,
                Opcode::Xor => self.stack.pop_2_push(&Operation::xor)?,
                Opcode::Imp => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Eqv => self.stack.pop_2_push(&Operation::unimplemented)?,

                Opcode::Abs => self.stack.pop_1_push(&Function::abs)?,
                Opcode::Chr => self.stack.pop_1_push(&Function::chr)?,
                Opcode::Cos => self.stack.pop_1_push(&Function::cos)?,
                Opcode::Int => self.stack.pop_1_push(&Function::int)?,
                Opcode::Rnd => {
                    let vec = self.stack.pop_vec()?;
                    self.stack.push(Function::rnd(&mut self.rand, vec)?)?;
                }
                Opcode::Sin => self.stack.pop_1_push(&Function::sin)?,
                Opcode::Tab => {
                    let val = self.stack.pop()?;
                    self.stack.push(Function::tab(self.print_col, val)?)?;
                }
            }
        }
        Ok(Event::Running)
    }

    fn r#clear(&mut self) -> Result<()> {
        self.vars.clear();
        Ok(())
    }

    fn r#cont(&mut self) -> Result<Option<Event>> {
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
            Ok(None)
        } else {
            Ok(Some(Event::Running))
        }
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

    fn r#input(&mut self, var_name: Rc<str>) -> Result<Option<Event>> {
        if let State::Running = self.state {
            self.state = State::Input;
            self.pc -= 1;
            Ok(Some(Event::Running))
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
                        pop = Val::String(v.into());
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
            Ok(None)
        } else {
            Err(error!(InternalError))
        }
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

    fn r#new_(&mut self) -> Result<Event> {
        self.stack.clear();
        self.vars.clear();
        self.source.clear();
        self.dirty = true;
        self.state = State::Stopped;
        self.cont = State::Stopped;
        Ok(Event::Stopped)
    }

    fn r#next(&mut self, next_name: Rc<str>) -> Result<()> {
        loop {
            let next;
            loop {
                match self.stack.pop() {
                    Ok(Val::Next(addr)) => {
                        next = addr;
                        break;
                    }
                    Ok(_) => continue,
                    Err(_) => return Err(error!(NextWithoutFor)),
                }
            }
            let var_name_val = self.stack.pop()?;
            let step_val = self.stack.pop()?;
            let to_val = self.stack.pop()?;
            if let Val::String(var_name) = var_name_val {
                if !next_name.is_empty() && var_name != next_name {
                    //self.stack.push(Val::String(next_name))?;
                    continue;
                }
                let mut current = self.vars.fetch(&var_name);
                current = Operation::sum(current, step_val.clone())?;
                self.vars.store(&var_name, current.clone())?;
                if let Ok(step) = f64::try_from(step_val.clone()) {
                    let done = Val::Integer(-1)
                        == if step < 0.0 {
                            Operation::less(current, to_val.clone())?
                        } else {
                            Operation::less(to_val.clone(), current)?
                        };
                    if !done {
                        self.stack.push(to_val)?;
                        self.stack.push(step_val)?;
                        self.stack.push(Val::String(var_name))?;
                        self.stack.push(Val::Next(next))?;
                        self.pc = next;
                    }
                    return Ok(());
                }
            }
        }
    }

    fn r#on(&mut self) -> Result<()> {
        let select = i16::try_from(self.stack.pop()?)?;
        let len = i16::try_from(self.stack.pop()?)?;
        if select < 0 || len < 0 {
            return Err(error!(IllegalFunctionCall));
        }
        if select == 0 || select > len {
            self.pc += len as usize;
        } else {
            self.pc += select as usize - 1;
        }
        Ok(())
    }

    fn r#print(&mut self) -> Result<Event> {
        let mut s = String::new();
        let item = self.stack.pop()?;
        let val_str = match item {
            Val::String(s) => s,
            _ => format!("{} ", item).into(),
        };
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
