use std::env;
use std::fs;
use std::io::{self, Write};

struct Context {
    memory: [u8; 30000],
    pc: usize,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cmdline = if args.len() > 1 { Some(&args[1]) } else { None };

    let mut ctx = Context {
        memory: [0; 30000],
        pc: 0,
    };

    match cmdline {
        Some(filename) => {
            let input = read_file(filename);
            eval(&mut ctx, &input);
        }
        None => loop {
            print!("bf> ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => eval(&mut ctx, &input),
                Err(err) => println!("error: {}", err),
            }
        },
    }
}

fn read_file(filename: &str) -> String {
    return fs::read_to_string(filename).unwrap();
}

fn eval(ctx: &mut Context, source: &str) {
    let mut i = 0;
    let program = source.as_bytes();
    while ctx.pc < program.len() {
        match program[ctx.pc] {
            b'+' => ctx.memory[i] = ctx.memory[i].overflowing_add(1).0,
            b'-' => ctx.memory[i] -= 1,
            b'>' => i += 1,
            b'<' => {
                assert!(i > 0);
                i -= 1
            }
            b'.' => print!("{}", ctx.memory[i] as u8 as char),
            b',' => {
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                ctx.memory[i] = input.as_bytes()[0] as u8
            }
            b'[' => {
                if ctx.memory[i] == 0 {
                    let mut flag = 0;
                    while flag >= 0 {
                        ctx.pc += 1;
                        assert!(ctx.pc < program.len());
                        if program[ctx.pc] == b'[' {
                            flag += 1
                        } else if program[ctx.pc] == b']' {
                            flag -= 1
                        }
                    }
                }
            }
            b']' => {
                if ctx.memory[i] != 0 {
                    let mut flag = 0;
                    while flag >= 0 {
                        assert!(ctx.pc > 0);
                        ctx.pc -= 1;
                        if program[ctx.pc] == b']' {
                            flag += 1
                        } else if program[ctx.pc] == b'[' {
                            flag -= 1
                        }
                    }
                }
            }
            _ => (),
        }
        ctx.pc += 1;
    }
}
