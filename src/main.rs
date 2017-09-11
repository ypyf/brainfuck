use std::io::{self, Write};

fn main() {
    let mut memory: [i8; 30000] = [0; 30000];
    let mut i = 0;
    loop {
        print!("=> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let mut pc = 0;
                let program = input.as_bytes();
                while pc < program.len() {
                    match program[pc] {
                        b'+' => memory[i] += 1,
                        b'-' => memory[i] -= 1,
                        b'>' => i += 1,
                        b'<' => {
                            assert!(i > 0);
                            i -= 1
                        }
                        b'.' => print!("{}", memory[i] as u8 as char),
                        b',' => {
                            let mut input = String::new();
                            io::stdin().read_line(&mut input).unwrap();
                            memory[i] = input.as_bytes()[0] as i8
                        }
                        b'[' => {
                            if memory[i] == 0 {
                                let mut flag = 0;
                                while flag >= 0 {
                                    pc += 1;
                                    assert!(pc < program.len());
                                    if program[pc] == b'[' {
                                        flag += 1
                                    } else if program[pc] == b']' {
                                        flag -= 1
                                    }
                                }
                            }
                        }
                        b']' => {
                            if memory[i] != 0 {
                                let mut flag = 0;
                                while flag >= 0 {
                                    assert!(pc > 0);
                                    pc -= 1;
                                    if program[pc] == b']' {
                                        flag += 1
                                    } else if program[pc] == b'[' {
                                        flag -= 1
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                    pc += 1;
                }
            }
            Err(err) => println!("error: {}", err),
        }
    }
}
