use std::env;
use std::fs;
use std::io::{self, Write};

#[derive(Debug)]
struct Context {
    memory: [u8; 30000],
    pc: usize,
}

#[derive(Debug)]
struct Error {
    message: &'static str,
}

#[derive(PartialEq, Debug, Clone)]
enum Command {
    Inc,              // +
    Dec,              // -
    IncPtr,           // >
    DecPtr,           // <
    Read,             // ,
    Write,            // .
    EnterLoop(usize), // [
    ExitLoop(usize),  // ]
}

struct Compiler {
    pub program: Vec<Command>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler { program: vec![] }
    }

    pub fn compile(&mut self, source: &str) -> Result<&Vec<Command>, Error> {
        self.generate_instructions(source)?.calculate_jump_address();
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
                        todo!()
                    }
                }
                _ => (),
            }
            position += 1;
        }
        self
    }
}

fn execute(ctx: &mut Context, program: &Vec<Command>) {
    let mut i = 0;
    while ctx.pc < program.len() {
        match program[ctx.pc] {
            Command::Inc => ctx.memory[i] = ctx.memory[i].overflowing_add(1).0,
            Command::Dec => ctx.memory[i] -= 1,
            Command::IncPtr => i += 1,
            Command::DecPtr => i -= 1,
            Command::Read => {
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                ctx.memory[i] = input.as_bytes()[0]
            }
            Command::Write => print!("{}", ctx.memory[i] as char),
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
