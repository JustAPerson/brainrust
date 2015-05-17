#![feature(core)]

#[macro_use] extern crate log;
extern crate env_logger;

use std::io::{BufRead, Read, Write};

mod interpreter;
use interpreter::Interpreter;

fn main() {
    env_logger::init().unwrap();

    let args = std::env::args().collect::<Vec<_>>();

    let source;
    let result = match args.len() {
        1 => {
            source = "<anon>";
            handle_repl()
        },
        2 => {
            source = &args[1];
            handle_file(source)
        },
        _ => {
            write!(std::io::stderr(), "Usage:   brainrust [file]\n").ok();
            return;
        }
    };

    match result {
        // Nothing went wrong
        Ok(_) => { },
        // Report error to user
        Err(_) => {
            write!(std::io::stderr(), "Unable to open file `{}`\n", source).ok();
        }
    }
}

fn handle_repl() -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let mut i = Interpreter::new();

    debug!("handle_repl()");

    for line in stdin.lock().lines() {
        i.exec(try!(line));
        println!("{:?}", i);
        i.reset();
    }

    Ok(())
}

fn handle_file(handle: &str) -> std::io::Result<()> {
    debug!("handle_file({:?})", handle);

    let mut file = try!(std::fs::File::open(handle));

    let mut source = String::new();
    try!(file.read_to_string(&mut source));

    let mut i = Interpreter::new();
    i.exec(source);
    debug!("handle_file() Interpreter = {:?}", i);

    Ok(())
}

