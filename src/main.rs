use bf::{gen_c::bf_to_c, Compiler, Machine};
use std::{
    env,
    fs::{self, File},
    io::{self, Write},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    match args.get(1).map(|f| f.to_owned()).as_deref() {
        Some("run") => match args.get(2) {
            Some(filename) => {
                let input = fs::read_to_string(filename).unwrap();
                let mut machine = Machine::new();
                match Compiler::compile(&input) {
                    Ok(program) => machine.execute(&program),
                    Err(err) => println!("error: {}", err.message),
                }
            }
            None => {
                eprintln!("error: no input files");
            }
        },
        Some("repl") => loop {
            print!("bf> ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let mut machine = Machine::new();
                    match Compiler::compile(&input) {
                        Ok(program) => machine.execute(&program),
                        Err(err) => println!("error: {}", err.message),
                    }
                }
                Err(err) => println!("error: {}", err),
            }
        },
        Some("gen-c") => match args.get(2) {
            Some(filename) => {
                let source = fs::read_to_string(filename).unwrap();
                match Compiler::compile(&source) {
                    Ok(program) => {
                        let code = bf_to_c(&program);
                        let mut file = File::create(format!("{}.c", filename))?;
                        file.write_all(code.as_bytes())?;
                    }
                    Err(err) => println!("error: {}", err.message),
                }
            }
            None => {
                eprintln!("error: no input files")
            }
        },
        Some(cmd) => {
            eprintln!("error: unknown command: {}", cmd)
        }
        None => {
            println!("Usage: bf [command] [options]\n");
            println!("Commands:\n");
            println!("  run\t\tExecute a brainfuck script");
            println!("  repl\t\tStart a REPL session");
            println!("  gen-c\t\tGenerate a C source file from a brainfuck script");
        }
    }
    Ok(())
}
