use std::env;
use std::fs;
use std::io::{self, Read, Write};

#[derive(Debug)]
struct Context {
    memory: [i32; 30000],
    pc: usize,
}

#[derive(Debug)]
struct Error {
    message: &'static str,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum Command {
    Inc,              // +
    Dec,              // -
    IncPtr,           // >
    DecPtr,           // <
    Read,             // ,
    Write,            // .
    EnterLoop(usize), // [
    ExitLoop(usize),  // ]
    Zero,             // [-]
    SuperInc(i32),
    SuperDec(i32),
    SuperIncPtr(usize),
    SuperDecPtr(usize),
}

struct Compiler {
    pub program: Vec<Command>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler { program: vec![] }
    }

    pub fn compile(&mut self, source: &str) -> Result<&Vec<Command>, Error> {
        self.generate_instructions(source)?
            .optimize_code()
            .calculate_jump_address();
        Ok(&self.program)
    }

    fn generate_instructions(&mut self, source: &str) -> Result<&mut Self, Error> {
        let mut position: usize = 0;
        self.program = source
            .chars()
            .filter_map(|ch| {
                let command = match ch {
                    '+' => Some(Command::Inc),
                    '-' => Some(Command::Dec),
                    '>' => Some(Command::IncPtr),
                    '<' => Some(Command::DecPtr),
                    ',' => Some(Command::Read),
                    '.' => Some(Command::Write),
                    '[' => Some(Command::EnterLoop(0)),
                    ']' => Some(Command::ExitLoop(0)),
                    _ => None,
                };

                if command.is_some() {
                    position += 1
                }

                command
            })
            .collect();

        Ok(self)
    }

    fn calculate_jump_address(&mut self) -> &mut Self {
        let mut position: usize = 0;
        let mut jumps: Vec<&mut Command> = vec![];
        for command in &mut self.program {
            match command {
                Command::EnterLoop(_) => {
                    *command = Command::ExitLoop(position);
                    jumps.push(command)
                }
                Command::ExitLoop(_) => {
                    *command = Command::EnterLoop(position);
                    if let Some(top) = jumps.pop() {
                        std::mem::swap(top, command)
                    } else {
                        // TODO handle error
                        todo!()
                    }
                }
                _ => (),
            }
            position += 1;
        }

        if !jumps.is_empty() {
            // TODO handle error
            todo!()
        }
        self
    }

    fn optimize_code(&mut self) -> &mut Self {
        let mut i: usize = 0;
        let mut transformed: Vec<Command> = vec![];
        while i < self.program.len() {
            let command = self.program[i];
            match command {
                Command::Inc => merge_inc(&mut transformed, command),
                Command::Dec => merge_dec(&mut transformed, command),
                Command::IncPtr => merge_incptr(&mut transformed, command),
                Command::DecPtr => merge_decptr(&mut transformed, command),
                Command::EnterLoop(_) => {
                    if self.program.len() - i >= 2 {
                        match (self.program[i + 1], self.program[i + 2]) {
                            (Command::Dec, Command::ExitLoop(_)) => {
                                transformed.push(Command::Zero);
                                i += 2
                            }
                            _ => transformed.push(command),
                        }
                    }
                }
                _ => transformed.push(command),
            }
            i += 1
        }
        self.program = transformed;
        self
    }
}

fn merge_inc(transformed: &mut Vec<Command>, command: Command) {
    let mut superinstr = command;
    if let Some(prev) = transformed.last() {
        if *prev == command {
            transformed.pop();
            superinstr = Command::SuperInc(2)
        } else if let Command::SuperInc(n) = *prev {
            transformed.pop();
            superinstr = Command::SuperInc(n + 1)
        }
    }
    transformed.push(superinstr)
}

fn merge_dec(transformed: &mut Vec<Command>, command: Command) {
    let mut superinstr = command;
    if let Some(prev) = transformed.last() {
        if *prev == command {
            transformed.pop();
            superinstr = Command::SuperDec(2)
        } else if let Command::SuperDec(n) = *prev {
            transformed.pop();
            superinstr = Command::SuperDec(n + 1)
        }
    }
    transformed.push(superinstr)
}

fn merge_incptr(transformed: &mut Vec<Command>, command: Command) {
    let mut superinstr = command;
    if let Some(prev) = transformed.last() {
        if *prev == command {
            transformed.pop();
            superinstr = Command::SuperIncPtr(2)
        } else if let Command::SuperIncPtr(n) = *prev {
            transformed.pop();
            superinstr = Command::SuperIncPtr(n + 1)
        }
    }
    transformed.push(superinstr)
}

fn merge_decptr(transformed: &mut Vec<Command>, command: Command) {
    let mut superinstr = command;
    if let Some(prev) = transformed.last() {
        if *prev == command {
            transformed.pop();
            superinstr = Command::SuperDecPtr(2)
        } else if let Command::SuperDecPtr(n) = *prev {
            transformed.pop();
            superinstr = Command::SuperDecPtr(n + 1)
        }
    }
    transformed.push(superinstr)
}

fn execute(ctx: &mut Context, program: &Vec<Command>) {
    let mut i = 0;
    while ctx.pc < program.len() {
        match program[ctx.pc] {
            Command::Inc => ctx.memory[i] += 1,
            Command::Dec => ctx.memory[i] -= 1,
            Command::IncPtr => i += 1,
            Command::DecPtr => i -= 1,
            Command::Read => {
                ctx.memory[i] = std::io::stdin()
                    .bytes()
                    .next()
                    .and_then(|ch| ch.ok())
                    .map(|ch| ch as i32)
                    .unwrap_or(0)
            }
            Command::Write => print!("{}", ctx.memory[i] as u8 as char),
            Command::EnterLoop(disp) => {
                if ctx.memory[i] == 0 {
                    ctx.pc = disp
                }
            }
            Command::ExitLoop(disp) => {
                if ctx.memory[i] != 0 {
                    ctx.pc = disp
                }
            }
            Command::Zero => ctx.memory[i] = 0,
            Command::SuperInc(n) => ctx.memory[i] += n,
            Command::SuperDec(n) => ctx.memory[i] -= n,
            Command::SuperIncPtr(n) => i += n,
            Command::SuperDecPtr(n) => i -= n,
        }
        ctx.pc += 1;
    }
}

fn run(source: &str) {
    let mut ctx = Context {
        memory: [0; 30000],
        pc: 0,
    };
    let mut compiler = Compiler::new();
    match compiler.compile(&source) {
        Ok(program) => {
            // println!("{:?}", program);
            execute(&mut ctx, program)
        }
        Err(err) => println!("error: {}", err.message),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1) {
        Some(filename) => {
            let input = fs::read_to_string(filename).unwrap();
            run(&input)
        }
        None => loop {
            print!("bf> ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => run(&input),
                Err(err) => println!("error: {}", err),
            }
        },
    }
}
