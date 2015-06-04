use interpreter::{Opcode, Program, Step};

// The profiler works by tracking loop iterations.
// Because loops are brainfuck's only means of branching,
// this effectively measures all codepaths.
#[derive(Debug)]
pub struct Profiler {
    map: Record,
    program: Program,
}

impl Profiler {
    pub fn new(p: Program) -> Profiler {
        Profiler {
            map: Record::new(&p),
            program: p,
        }
    }
    pub fn step(&mut self, step: Step) {
        trace!("Profiler::step({:?}) record={:?}", step, self.map);
        match step {
            Step::EnterLoop => {
                self.map.enter();
            },
            Step::LeaveLoop(count) => {
                self.map.get().count += count;
                self.map.leave();
            },
        }
    }
    pub fn print(&self) {
        PrintContext::new(self).print();
    }
}

/// A Record represents a single loop structure
#[derive(Debug)]
struct Record {
    count: u64,
    // None    => use this instance
    // Some(i) => inspect children[i]
    state: Option<usize>,
    children: Vec<Record>,
}

impl Record {
    pub fn new(program: &Program) -> Record {
        Record {
            count: 0,
            state: None,
            children: program.opcodes.iter().filter_map(Opcode::get_program)
                                     .map(|p| Record::new(p)).collect(),
        }
    }
    pub fn get(&mut self) -> &mut Record {
        match self.state {
            None => { self },
            Some(i) => {
                self.children[i].get()
            }
        }
    }
    pub fn enter(&mut self) {
        enum Step {
            Done,
            Continue,
        }

        fn step(r: &mut Record) -> Step {
            match r.state {
                None if r.children.len() == 0 => { Step::Continue },
                None => {
                    r.state = Some(0);
                    Step::Done
                },
                Some(i) => {
                    match step(&mut r.children[i]) {
                        Step::Done => { Step::Done },
                        Step::Continue => {
                            if i + 1 < r.children.len() {
                                r.state = Some(i + 1);
                                Step::Done
                            } else {
                                Step::Continue
                            }
                        }
                    }
                }
            }
        }

        step(self);
    }
    pub fn leave(&mut self) {
        match self.state {
            None => {
                // TODO fix panic in profiler
                // Several files in ./tests break here, twos.bf being the most simple
                // Not sure what causes this yet.
                // Seems to be a nontrivial case.
                // Ignoring causes incorrect results
                panic!();
            },
            Some(i) => {
                match self.children[i].state {
                    None => {
                        if i + 1 < self.children.len() {
                            self.state = Some(i + 1);
                        } else {
                            self.state = None;
                        }
                    },
                    Some(_) => { self.children[i].leave() },
                }
            }
        }
    }
}

struct PrintContext<'a> {
    program: &'a Program,
    total: u64,
    count: Count,
}

impl<'a> PrintContext<'a> {
    pub fn new<'b>(profiler: &'b Profiler) -> PrintContext<'b> {
        let count = Count::new(&profiler.map, &profiler.program);
        PrintContext {
            program: &profiler.program,
            total: count.oc + count.oe_total,
            count: count,
        }
    }
    pub fn print(&self) {
        println!("Instructions executed: {}", self.total);
        self.print_step(0, self.program, &self.count);
    }
    fn print_step(&self, level: usize, program: &Program, count: &Count) {
        use std::iter;
        let prefix = iter::repeat(' ').take(level * 2).collect::<String>();

        let mut record_iter = count.children.iter();

        for opcode in &program.opcodes {
            print!("{}", prefix); // indent some with some whitespace
            if let Opcode::Loop(ref p) = *opcode {
                let record = record_iter.next().unwrap();
                let percent: f64 = (record.oe_total as f64) / (self.total as f64) * 100.0;
                print!("Loop ");
                print!("{} iterations ", record.iterations);
                print!("{} instr ({:.4} % total)", record.oe_total, percent);
                println!("");
                self.print_step(level + 1, p, record);
            } else {
                print!("{:?}", opcode);
                println!("");
            }
        }
    }
}

struct Count {
    iterations: u64,
    // opcode count, not including the lengths of subloops
    oc: u64,
    // total instructions executed inside this loop
    // and all subloops for all iterations
    oe_total: u64,
    children: Vec<Count>,
}
impl Count {
    pub fn new(r: &Record, p: &Program) -> Count {
        let subprograms = p.opcodes.iter().filter_map(Opcode::get_program);
        let children: Vec<_> = r.children.iter().zip(subprograms)
                                         .map(|(r, p)| Count::new(r, p)).collect();
        let oc = p.opcodes.len() as u64;
        let oe_partial = children.iter().map(|r| r.oe_total).sum::<u64>();
        Count {
            iterations: r.count,
            oc: oc,
            oe_total: r.count * oc + oe_partial,
            children: children,
        }
    }
}

