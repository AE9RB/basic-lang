extern crate ctrlc;
extern crate linefeed;
use crate::mach::{Event, Runtime};
use linefeed::interface::Interface;
use linefeed::reader::ReadResult;
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
    let interface = Interface::new("BASIC")?;
    let mut runtime = Runtime::new();
    let mut print_ready = true;

    loop {
        if interrupted.load(Ordering::SeqCst) {
            runtime.interrupt();
            interrupted.store(false, Ordering::SeqCst);
        };
        match runtime.execute(5000) {
            Event::PrintLn(s) => {
                interface.write_fmt(format_args!("{}\n", s))?;
            }
            Event::Stopped => {
                if print_ready {
                    print_ready = false;
                    interface.write_fmt(format_args!("READY.\n"))?;
                }
                match interface.read_line()? {
                    ReadResult::Input(input) => runtime.enter(&input),
                    ReadResult::Signal(_) | ReadResult::Eof => break,
                }
            }
            Event::Errors(errors) => {
                for error in errors {
                    interface.write_fmt(format_args!("?{}\n", error))?;
                }
                print_ready = true;
            }
            Event::Running => {}
        }
    }
    Ok(())
}
