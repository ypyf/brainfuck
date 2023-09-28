use std::cell::RefCell;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::rc::Rc;

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
    Inc,                    // +
    Dec,                    // -
    IncPtr,                 // >
    DecPtr,                 // <
    Read,                   // ,
    Write,                  // .
    JZ(Rc<RefCell<usize>>), // [
    JNZ(usize),             // ]
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1) {
        Some(filename) => {
            let input = read_file(filename);
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

fn run(source: &str) {
    let mut ctx = Context {
        memory: [0; 30000],
        pc: 0,
    };
    match compile(&source) {
        Ok(program) => {
            // println!("{:?}", program);
            execute(&mut ctx, &program)
        }
        Err(err) => println!("error: {}", err.message),
    }
}

fn compile(source: &str) -> Result<Vec<Command>, Error> {
    let mut position: usize = 0;
    let mut labels: Vec<Rc<RefCell<usize>>> = vec![];
    let mut program: Vec<Command> = vec![];
    for ch in source.chars() {
        match ch {
            '+' => program.push(Command::Inc),
            '-' => program.push(Command::Dec),
            '>' => program.push(Command::IncPtr),
            '<' => program.push(Command::DecPtr),
            ',' => program.push(Command::Read),
            '.' => program.push(Command::Write),
            '[' => {
                let label = Rc::new(RefCell::new(position));
                let command = Command::JZ(label.clone());
                labels.push(label);
                program.push(command)
            }
            ']' => {
                if let Some(label) = labels.pop() {
                    let disp = position - *label.borrow();
                    *label.borrow_mut() = disp;
                    program.push(Command::JNZ(disp))
                } else {
                    return Err(Error {
                        message: "too many close brackets",
                    });
                }
            }
            _ => continue,
        }
        position += 1;
    }

    if !labels.is_empty() {
        return Err(Error {
            message: "too many open brackets",
        });
    }

    Ok(program)
}

fn execute(ctx: &mut Context, program: &Vec<Command>) {
    let mut i = 0;
    while ctx.pc < program.len() {
        match &program[ctx.pc] {
            Command::Inc => ctx.memory[i] = ctx.memory[i].overflowing_add(1).0,
            Command::Dec => ctx.memory[i] -= 1,
            Command::IncPtr => i += 1,
            Command::DecPtr => i -= 1,
            Command::Read => {
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                ctx.memory[i] = input.as_bytes()[0]
            }
            Command::Write => print!("{}", ctx.memory[i] as u8 as char),
            Command::JZ(disp) => {
                if ctx.memory[i] == 0 {
                    ctx.pc += *disp.borrow();
                }
            }
            Command::JNZ(disp) => {
                if ctx.memory[i] != 0 {
                    ctx.pc -= disp
                }
            }
        }
        ctx.pc += 1;
    }
}

fn read_file(filename: &str) -> String {
    return fs::read_to_string(filename).unwrap();
}
