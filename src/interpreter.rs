use std;
use std::convert::From;
use std::io::{Read, Write};
use std::num::wrapping::OverflowingOps;

#[derive(Debug, Clone, PartialEq)]
pub enum Opcode {
    Add(u8),
    Sub(u8),
    Left(usize),
    Right(usize),
    Loop(Program),
    In,
    Out,
}

impl Opcode {
    pub fn size(&self) -> usize {
        match *self {
            Opcode::Loop(ref p) => 1 + p.size(),
            _ => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    opcodes: Vec<Opcode>,
}

impl Program {
    /// Count the number of opcodes
    pub fn size(&self) -> usize {
        self.opcodes.iter().map(|op| op.size()).sum()
    }

    /// Reduce repeated opcodes
    ///
    /// Using the first opcodes as a starting point, look at the following
    /// opcodes. While they are of the same category as the current
    /// opcodes (i.e. both either Add/Sub), combine them.
    ///
    /// Before: Add(1), Add(1), Add(1), Sub(1), Right(1), Right(1)...
    /// After:  Add(2), Right(2) ...
    pub fn reduce(&mut self) {
        use self::Opcode::*;
        let mut reduced = Vec::new();
        {
            let mut iter = self.opcodes.drain(..);
            let mut curr = match iter.next() {
                Some(o) => o,
                None => { return },
            };
            while let Some(opcode) = iter.next() {
                curr = match (opcode, curr) {
                    // TODO deal with overflows here
                    (Add(a), Add(b)) => { Add(a + b) },
                    (Add(a), Sub(b)) => { Sub(b - a) },
                    (Sub(a), Add(b)) => { Add(b - a) },
                    (Sub(a), Sub(b)) => { Sub(a + b) },
                    (Left(a), Left(b)) => { Left(a + b) },
                    (Left(a), Right(b)) => { Right(b - a) },
                    (Right(a), Left(b)) => { Left(b - a) },
                    (Right(a), Right(b)) => { Right(a + b) },
                    (Loop(mut p), prev) => {
                        reduced.push(prev);
                        p.reduce();
                        Loop(p)
                    }
                    (instr, prev) => {
                        reduced.push(prev);
                        instr
                    },
                }
            }
            reduced.push(curr);
        }
        self.opcodes = reduced;
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
    /// Stack pointer
    sp: usize,
    /// Instruction counter
    ic: usize,
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
            self.ic += 1;
            match *opcode {
                Add(n) => { self.stack[self.sp] = self.stack[self.sp].overflowing_add(n).0 },
                Sub(n) => { self.stack[self.sp] = self.stack[self.sp].overflowing_sub(n).0 },
                // TODO underflow
                Left(n) => { self.sp -= n },
                Right(n) => {
                    // grow stack if necessary
                    self.sp += n;
                    while self.sp >= self.stack.len() { self.stack.push(0) }
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
                '+' => { Opcode::Add(1) },
                '-' => { Opcode::Sub(1) },
                '<' => { Opcode::Left(1) },
                '>' => { Opcode::Right(1) },
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

