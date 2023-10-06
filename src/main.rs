use bf::run;
use std::{
    env, fs,
    io::{self, Write},
};

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
