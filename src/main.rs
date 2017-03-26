use std::io::{self, Read, Write};

fn main() {
    let mut memory: [i8; 30000] = [0; 30000];
    let mut i = 0;
    loop {
        print!("=> ");
        io::stdout().flush();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let mut pc = 0;
                let program = input.as_bytes();
                while pc < program.len() {
                    //println!("pc={} cell[{}]={}", pc, i, memory[i]);
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
                            let stdin = io::stdin();
                            for byte in stdin.lock().bytes() {
                                let c = byte.unwrap();
                                if c == b'\n' {
                                    break;
                                }
                                memory[i] = c as i8;
                            }
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
