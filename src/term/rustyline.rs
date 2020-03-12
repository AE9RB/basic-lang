extern crate ctrlc;
extern crate rustyline;
use crate::mach::{Event, Runtime};
use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn main() {
    let interrupted = Arc::new(AtomicBool::new(false));
    let int_moved = interrupted.clone();
    ctrlc::set_handler(move || {
        int_moved.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    main_loop(interrupted);
}

fn main_loop(interrupted: Arc<AtomicBool>) {
    let mut editor = Editor::<()>::new();
    let mut runtime = Runtime::new();
    let mut print_ready = true;

    loop {
        if interrupted.load(Ordering::SeqCst) {
            runtime.interrupt();
            interrupted.store(false, Ordering::SeqCst);
        };
        match runtime.execute(5000) {
            Event::PrintLn(s) => {
                println!("{}",s);
            }
            Event::Stopped => {
                if print_ready {
                    print_ready = false;
                    println!("READY.");
                }
                match editor.readline("") {
                    Ok(input) => {
                        runtime.enter(&input)
                    }
                    Err(ReadlineError::Interrupted) => {
                        //print!("^C");
                    }
                    Err(ReadlineError::Eof) => {
                        break;
                    }
                    Err(err) => {
                        eprintln!("{:?}", err);
                        break;
                    }
                }
            }
            Event::Errors(errors) => {
                for error in errors {
                    println!("?{}", error);
                }
                print_ready = true;
            }
            Event::Running => {}
        }
    }
}