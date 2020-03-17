extern crate ansi_term;
extern crate ctrlc;
extern crate linefeed;
use crate::mach::{Event, Runtime};
use ansi_term::Style;
use linefeed::complete::{Completer, Completion};
use linefeed::terminal::Terminal;
use linefeed::{Interface, Prompter, ReadResult, Signal};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn main() {
    let interrupted = Arc::new(AtomicBool::new(false));
    let int_moved = interrupted.clone();
    ctrlc::set_handler(move || {
        int_moved.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    if let Err(error) = main_loop(interrupted) {
        eprintln!("{}", error);
    }
}

fn main_loop(interrupted: Arc<AtomicBool>) -> std::io::Result<()> {
    let command = Interface::new("BASIC")?;
    let input = Interface::new("INPUT")?;
    input.set_report_signal(Signal::Interrupt, true);
    let mut runtime = Arc::new(Runtime::default());

    loop {
        if interrupted.load(Ordering::SeqCst) {
            Arc::get_mut(&mut runtime).unwrap().interrupt();
            interrupted.store(false, Ordering::SeqCst);
        };
        match Arc::get_mut(&mut runtime).unwrap().execute(5000) {
            Event::Stopped => {
                let saved_completer = command.completer();
                command.set_completer(Arc::new(LineCompleter::new(Arc::clone(&runtime))));
                let string = match command.read_line()? {
                    ReadResult::Input(string) => string,
                    ReadResult::Signal(_) | ReadResult::Eof => break,
                };
                command.set_completer(saved_completer);
                if Arc::get_mut(&mut runtime).unwrap().enter(&string) {
                    command.add_history_unique(string);
                }
            }
            Event::Input(prompt) => {
                input.set_prompt(&prompt)?;
                match input.read_line()? {
                    ReadResult::Input(string) => {
                        if Arc::get_mut(&mut runtime).unwrap().enter(&string) {
                            input.add_history_unique(string);
                        }
                    }
                    ReadResult::Signal(Signal::Interrupt) => {
                        input.set_buffer("")?;
                        Arc::get_mut(&mut runtime).unwrap().interrupt();
                    }
                    ReadResult::Signal(_) | ReadResult::Eof => break,
                };
            }
            Event::Errors(errors) => {
                for error in errors.iter() {
                    let error = format!("{}", error);
                    command.write_fmt(format_args!("{}\n", Style::new().bold().paint(error)))?;
                }
            }
            Event::Running => {}
            Event::Print(s) => {
                command.write_fmt(format_args!("{}", s))?;
            }
            Event::List((s, columns)) => {
                command.write_fmt(format_args!("{}\n", list(&s, &columns)))?;
            }
        }
    }
    Ok(())
}

struct LineCompleter {
    runtime: Arc<Runtime>,
}

impl LineCompleter {
    fn new(runtime: Arc<Runtime>) -> LineCompleter {
        LineCompleter { runtime }
    }
}

impl<'a, Term: Terminal> Completer<Term> for LineCompleter {
    fn complete(
        &self,
        _word: &str,
        prompter: &Prompter<Term>,
        _start: usize,
        _end: usize,
    ) -> Option<Vec<Completion>> {
        if let Ok(num) = prompter.buffer().parse::<usize>() {
            if let Some((s, _)) = self.runtime.line(num) {
                let mut comp_list = Vec::new();
                let mut comp = Completion::simple(s);
                comp.suffix = linefeed::complete::Suffix::None;
                comp_list.push(comp);
                return Some(comp_list);
            }
        }
        None
    }
}

fn list(ins: &str, columns: &[std::ops::Range<usize>]) -> String {
    let mut under_on = false;
    let mut out = String::new();
    let style = Style::new().underline();
    let prefix = format!("{}", style.prefix());
    let suffix = format!("{}", style.suffix());
    let mut index = 0;
    for char in ins.chars() {
        let do_under = columns.iter().any(|c| c.contains(&index));
        if under_on {
            if !do_under {
                out.push_str(&suffix);
            }
        } else if do_under {
            out.push_str(&prefix);
        }
        under_on = do_under;
        out.push(char);
        index += 1;
    }
    if columns.iter().any(|c| c.start == index) {
        under_on = true;
        out.push_str(&prefix);
        out.push(' ');
    }
    if under_on {
        out.push_str(&suffix);
    }
    out
}
