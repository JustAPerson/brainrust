use std;
use std::convert::From;
use std::io::{Read, Write};
use std::num::wrapping::OverflowingOps;

#[derive(Debug, Clone, PartialEq)]
pub enum Opcode {
    Add,
    Sub,
    Left,
    Right,
    Loop(Program),
    In,
    Out,
}

impl Opcode {
    pub fn size(&self) -> usize {
        match *self {
            Opcode::Loop(ref p) => p.size(),
            _ => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    opcodes: Vec<Opcode>,
}

impl Program {
    pub fn size(&self) -> usize {
        self.opcodes.iter().map(|op| op.size()).sum()
    }
}

impl From<String> for Program {
    fn from(s: String) -> Program {
        Program { opcodes: Builder::new(&s).parse() }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Interpreter {
    stack: Vec<u8>,
    sp: usize, // stack pointer
    ic: usize, // instruction counter
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            stack: vec![0],
            sp: 0,
            ic: 0,
        }
    }

    pub fn reset(&mut self) {
        self.sp = 0;
        self.ic = 0;
        self.stack.clear();
        self.stack.push(0);
    }

    pub fn exec<S: Into<Program>>(&mut self, source: S) {
        let program = source.into();
        debug!("Interpreter::exec(p) where p.size() = {}", program.size());
        self.exec_r(&program)
    }
    fn exec_r(&mut self, program: &Program) {
        use self::Opcode::*;
        for opcode in &program.opcodes {
            trace!("Interpreter::exec_r() opcode = {:?}", opcode);
            self.ic += 1;
            match *opcode {
                Add => { self.stack[self.sp] = self.stack[self.sp].overflowing_add(1).0 },
                Sub => { self.stack[self.sp] = self.stack[self.sp].overflowing_sub(1).0 },
                Left => { self.sp -= 1 },
                Right => {
                    // grow stack if necessary
                    self.sp += 1;
                    if self.sp == self.stack.len() { self.stack.push(0) }
                },
                Loop(ref os) => {
                    while self.stack[self.sp] != 0 {
                        self.exec_r(os);
                    }
                },
                In => {
                    self.stack[self.sp] = match std::io::stdin().take(1).bytes().next() {
                        Some(Ok(c)) => c,
                        _ => 0,
                    };
                },
                Out => {
                    let mut stdout = std::io::stdout();
                    write!(stdout, "{}", self.stack[self.sp] as char).ok();
                    stdout.flush().ok();
                }
            }
        }
    }
}


struct Builder<'a> {
    chars: std::str::Chars<'a>,
}

impl<'a> Builder<'a> {
    pub fn new(source: &'a String) -> Builder<'a> {
        Builder {
            chars: source.chars(),
        }
    }

    pub fn parse(&mut self) -> Vec<Opcode> {
        let mut opcodes = Vec::new();
        while let Some(c) = self.chars.next() {
            let opcode = match c {
                '+' => { Opcode::Add },
                '-' => { Opcode::Sub },
                '<' => { Opcode::Left },
                '>' => { Opcode::Right },
                '[' => { Opcode::Loop(Program { opcodes: self.parse() }) },
                ']' => { break },
                ',' => { Opcode::In },
                '.' => { Opcode::Out },
                _ => { continue },
            };
            opcodes.push(opcode);
        }
        opcodes
    }
}

