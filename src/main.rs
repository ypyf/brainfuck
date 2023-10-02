use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::str::Chars;

#[derive(Debug)]
struct Context {
    memory: [i32; 30000],
    i: isize,
    pc: usize,
}

impl Context {
    pub fn get(&self, offset: isize) -> i32 {
        self.memory[(self.i + offset) as usize]
    }

    pub fn set(&mut self, offset: isize, value: i32) {
        self.memory[(self.i + offset) as usize] = value
    }

    pub fn add(&mut self, offset: isize, value: i32) {
        self.memory[(self.i + offset) as usize] += value
    }

    pub fn move_ptr(&mut self, offset: isize) {
        self.i += offset
    }
}

#[derive(Debug, PartialEq)]
struct Error {
    message: &'static str,
}

#[derive(PartialEq, Debug, Clone)]
enum Command {
    Add(isize, i32), // +/-
    MovePtr(isize),  // >/<
    Output(isize),   // ,
    Input(isize),    // .
    Loop(Vec<Command>),
    If(Vec<Command>),
}

struct Compiler {}

impl Compiler {
    pub fn new() -> Self {
        Compiler {}
    }

    pub fn compile(&mut self, source: &str) -> Result<Vec<Command>, Error> {
        let mut result = Compiler::parse(source.chars())?;
        result = Compiler::optimize(&result);
        Ok(result)
    }

    fn parse(mut chars: Chars<'_>) -> Result<Vec<Command>, Error> {
        let mut result: Vec<Command> = vec![];
        while let Some(ch) = chars.next() {
            let cmd = if let Some(cmd) = Compiler::gen_command(ch) {
                cmd
            } else {
                match ch {
                    '[' => {
                        let body = Compiler::parse_body(&mut chars)?;
                        Command::Loop(body)
                    }
                    ']' => {
                        return Err(Error {
                            message: "Extra loop closing",
                        })
                    }
                    _ => continue, // ignore other characters
                }
            };

            result.push(cmd);
        }

        Ok(result)
    }

    fn parse_body(mut chars: &mut Chars<'_>) -> Result<Vec<Command>, Error> {
        let mut result: Vec<Command> = vec![];
        while let Some(ch) = chars.next() {
            let cmd = if let Some(cmd) = Compiler::gen_command(ch) {
                cmd
            } else {
                match ch {
                    '[' => {
                        let body = Compiler::parse_body(&mut chars)?;
                        Command::Loop(body)
                    }
                    ']' => return Ok(result),
                    _ => continue, // ignore other characters
                }
            };

            result.push(cmd);
        }

        Err(Error {
            message: "Unclosed loop",
        })
    }

    fn gen_command(ch: char) -> Option<Command> {
        match ch {
            '+' => Some(Command::Add(0, 1)),
            '-' => Some(Command::Add(0, -1)),
            '>' => Some(Command::MovePtr(1)),
            '<' => Some(Command::MovePtr(-1)),
            ',' => Some(Command::Input(0)),
            '.' => Some(Command::Output(0)),
            _ => None,
        }
    }

    fn optimize(code: &Vec<Command>) -> Vec<Command> {
        let mut result: Vec<Command> = vec![];
        let mut ptr_offset = 0;
        for command in code {
            match command {
                Command::Add(offset, cmd_value) => {
                    let actual_offset = offset + ptr_offset;
                    let mut fused = false;
                    if let Some(prev) = result.last_mut() {
                        fused = Compiler::fuse_add(prev, actual_offset, *cmd_value)
                    }
                    if !fused {
                        result.push(Command::Add(actual_offset, *cmd_value))
                    }
                }
                Command::MovePtr(offset) => ptr_offset += offset,
                Command::Input(offset) => result.push(Command::Input(offset + ptr_offset)),
                Command::Output(offset) => result.push(Command::Output(offset + ptr_offset)),
                _ => {
                    if ptr_offset != 0 {
                        result.push(Command::MovePtr(ptr_offset));
                        ptr_offset = 0
                    }
                    result.extend(Compiler::optimize_loop(command))
                }
            }
        }

        if ptr_offset != 0 {
            result.push(Command::MovePtr(ptr_offset))
        }
        result
    }

    fn optimize_loop(command: &Command) -> Vec<Command> {
        match command {
            Command::Loop(commands) => {
                if let Some(result) = Compiler::optimize_simple_loop(commands) {
                    result
                } else {
                    let cmd = Compiler::optimize_complex_loop(commands)
                        .unwrap_or(Command::Loop(Compiler::optimize(commands)));
                    vec![cmd]
                }
            }
            Command::If(commands) => vec![Command::If(Compiler::optimize(commands))],
            _ => unreachable!(),
        }
    }

    fn optimize_simple_loop(commands: &Vec<Command>) -> Option<Vec<Command>> {
        None
    }

    fn optimize_complex_loop(commands: &Vec<Command>) -> Option<Command> {
        None
    }

    fn fuse_add(prev: &mut Command, offset: isize, value: i32) -> bool {
        let mut fused = false;
        match prev {
            Command::Add(prev_offset, prev_value) => {
                if *prev_offset == offset {
                    *prev_value += value;
                    fused = true
                }
            }
            _ => (),
        }
        fused
    }
}

fn execute(ctx: &mut Context, program: &Vec<Command>) {
    for i in 0..program.len() {
        match program[i] {
            Command::Add(offset, value) => ctx.add(offset, value),
            Command::MovePtr(offset) => ctx.move_ptr(offset),
            Command::Input(offset) => {
                let ch = std::io::stdin()
                    .bytes()
                    .next()
                    .and_then(|ch| ch.ok())
                    .map(|ch| ch as i32)
                    .unwrap_or(0);
                ctx.set(offset, ch)
            }
            Command::Output(offset) => print!("{}", ctx.get(offset) as u8 as char),
            Command::Loop(ref commands) => {
                while ctx.get(0) != 0 {
                    execute(ctx, commands)
                }
            }
            _ => todo!(),
        }
        ctx.pc += 1;
    }
}

fn run(source: &str) {
    let mut ctx = Context {
        memory: [0; 30000],
        i: 0,
        pc: 0,
    };
    let mut compiler = Compiler::new();
    match compiler.compile(&source) {
        Ok(program) => {
            execute(&mut ctx, &program)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let result = vec![
            Command::Add(0, 1),
            Command::Add(0, -1),
            Command::MovePtr(1),
            Command::MovePtr(-1),
            Command::Input(0),
            Command::Output(0),
        ];
        assert_eq!(Compiler::parse("+-><,.".chars()), Ok(result));
    }

    #[test]
    fn it_parses_loop() {
        let result = vec![Command::Loop(vec![Command::Add(0, -1)])];
        assert_eq!(Compiler::parse("[-]".chars()), Ok(result));
    }

    #[test]
    fn it_parses_nested_loops() {
        let result = vec![Command::Loop(vec![
            Command::Add(0, 1),
            Command::Loop(vec![Command::Add(0, -1)]),
            Command::Add(0, 1),
        ])];
        assert_eq!(Compiler::parse("[+[-]+]".chars()), Ok(result));
    }
}
