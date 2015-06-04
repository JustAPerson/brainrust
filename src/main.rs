#![feature(core, collections_drain)]

#[macro_use] extern crate log;
extern crate env_logger;
extern crate docopt;

use std::io::{BufRead, Read, Write};

mod interpreter;
mod profiler;
use interpreter::{Interpreter, Program};
use profiler::Profiler;

static VERSION: &'static str = "Brainrust 0.0.1";
static USAGE: &'static str = "
Brainrust, a Brainfuck interpreter written in Rust.

Usage: brainrust [options]
       brainrust [options] <file>

Options:
    -h --help     Prints this screen
    -v --version  Prints the current version
    -O            Optimize
    -p --profile  Profile code. Causes slower execution.
";

struct Config<'a> {
    file: Option<&'a str>,
    optimize: bool,
    profile: bool,

    interpreter: Interpreter,
}

impl<'a> Config<'a> {
    pub fn new(o: &'a docopt::ArgvMap) -> Config<'a> {
        Config {
            file: match o.get_str("<file>") { "" => None, s => Some(s) },
            optimize: o.get_bool("-O"),
            profile: o.get_bool("-p"),

            interpreter: Interpreter::new(),
        }
    }

    pub fn run(&mut self) {
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

    fn exec<S: Into<Program>>(&mut self, source: S) {
        let mut p: Program = source.into();
        if self.optimize { p.reduce() }

        self.interpreter.reset();
        if self.profile {
            let mut profiler = Profiler::new(p.clone());
            self.interpreter.exec(p, |step| { profiler.step(step) });
            profiler.print();
        } else {
            self.interpreter.exec(p, |_| {});
        }

        // Report interpreter state
        match self.file {
            // TODO wishlist: output to file?
            Some(_) => { debug!("Config::exec() {:?}", self.interpreter) }
            // output for REPL
            None => { println!("{:?}", self.interpreter) },
        }
    }

    fn file(&mut self) -> std::io::Result<()> {
        let handle = self.file.unwrap();
        debug!("handle_file({:?})", handle);

        let mut file = try!(std::fs::File::open(handle));

        let mut source = String::new();
        try!(file.read_to_string(&mut source));

        self.exec(source);

        Ok(())
    }

    fn repl(&mut self) -> std::io::Result<()> {
        debug!("handle_repl()");

        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            self.exec(try!(line));
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
