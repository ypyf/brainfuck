use crate::Command;

mod c;

pub enum Target {
    C,
}

pub fn transpile_to(target: Target, program: &[Command]) -> String {
    match target {
        Target::C => c::transpile(program),
    }
}

fn emit(code: &str, level: i32) -> String {
    let mut result ="\t".repeat(level as usize);
    result += code;
    result += "\n";
    result
}

fn sign(value: i32) -> String {
    if value >= 0 {
        "+".to_owned()
    } else {
        "-".to_owned()
    }
}
