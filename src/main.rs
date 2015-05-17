#![feature(core, collections_drain)]

#[macro_use] extern crate log;
extern crate env_logger;
extern crate docopt;

use std::io::{BufRead, Read, Write};

mod interpreter;
use interpreter::{Interpreter, Program};

static VERSION: &'static str = "Brainrust 0.0.1";
static USAGE: &'static str = "
Brainrust, a Brainfuck interpreter written in Rust.

Usage: brainrust [options]
       brainrust [options] <file>

Options:
    -h --help     Prints this screen
    -v --version  Prints the current version
    -O            Optimize
";

struct Config<'a> {
    file: Option<&'a str>,
    optimize: bool,
}

impl<'a> Config<'a> {
    pub fn new(o: &'a docopt::ArgvMap) -> Config<'a> {
        Config {
            file: match o.get_str("<file>") { "" => None, s => Some(s) },
            optimize: o.get_bool("-O"),
        }
    }

    pub fn run(&self) {
        let result = match self.file {
            Some(_) => { self.file() },
            None => { self.repl() },
        };

        match result {
            // Nothing went wrong
            Ok(_) => { },
            // Report error to user
            Err(_) => {
                let source = self.file.unwrap_or("<anon>");
                write!(std::io::stderr(), "Unable to open file `{}`\n", source).ok();
            }
        }
    }

    fn file(&self) -> std::io::Result<()> {
        let handle = self.file.unwrap();
        debug!("handle_file({:?})", handle);

        let mut file = try!(std::fs::File::open(handle));

        let mut source = String::new();
        try!(file.read_to_string(&mut source));

        let mut i = Interpreter::new();
        let mut p = Program::from(source);
        if self.optimize { p.reduce() }
        i.exec(p);
        debug!("handle_file() Interpreter = {:?}", i);

        Ok(())
    }

    fn repl(&self) -> std::io::Result<()> {
        let stdin = std::io::stdin();
        let mut i = Interpreter::new();

        debug!("handle_repl()");

        for line in stdin.lock().lines() {
            let mut p = Program::from(try!(line));
            if self.optimize { p.reduce() }
            i.exec(p);
            println!("{:?}", i);
            i.reset();
        }

        Ok(())
    }
}

fn main() {
    env_logger::init().unwrap();

    let opts = docopt::Docopt::new(USAGE)
                              .and_then(|d| d.help(true)
                                             .version(Some(String::from(VERSION)))
                                             .parse())
                              .unwrap_or_else(|e| e.exit());

    Config::new(&opts).run()
}
