use crate::Command;

fn emit(code: &str, level: i32) -> String {
    let mut result = String::new();
    for _i in 0..level {
        result += "\t"
    }
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

pub fn bf_to_c(program: &[Command]) -> String {
    let mut result = String::new();
    result += &emit("#include <stdint.h>", 0);
    result += &emit("#include <stdio.h>", 0);
    result += &emit("#include <stdlib.h>", 0);
    result += &emit("", 0);
    result += &emit("static uint8_t read() {", 0);
    result += &emit("int temp = getchar();", 1);
    result += &emit("return (uint8_t)(temp != EOF ? temp : 0);", 1);
    result += &emit("}", 0);
    result += &emit("", 0);
    result += &emit("int main(void) {", 0);
    result += &emit("uint8_t mem[1000000] = {0};", 1);
    result += &emit("uint8_t *p = &mem[1000];", 1);
    result += &emit("", 0);
    result += &emit_commands(program, 1);
    result += &emit("", 0);
    result += &emit("return EXIT_SUCCESS;", 1);
    result += &emit("}", 0);
    result
}

fn emit_commands(program: &[Command], level: i32) -> String {
    let mut result = String::new();

    for cmd in program {
        match cmd.clone() {
            Command::Assign(offset, value) => {
                result += &emit(&format!("p[{}] = {};", offset, value), level)
            }
            Command::Add(offset, value) => {
                let mut s = format!("p[{}]", offset);
                if value == 1 {
                    s += "++;"
                } else if value == -1 {
                    s += "--;"
                } else {
                    s += &format!(" {}= {};", sign(value), i32::abs(value))
                }
                result += &emit(&s, level)
            }
            Command::MultAssign(src, dest, value) => {
                if value == 1 {
                    result += &emit(&format!("p[{}] = p[{}];", dest, src), level)
                } else {
                    result += &emit(&format!("p[{}] = p[{}] * {};", dest, src, value), level)
                }
            }
            Command::MultAdd(src, dest, value) => {
                if i32::abs(value) == 1 {
                    result += &emit(&format!("p[{}] {}= p[{}];", dest, sign(value), src), level)
                } else {
                    result += &emit(
                        &format!("p[{}] {}= p[{}] * {};", dest, sign(value), src, value),
                        level,
                    )
                }
            }
            Command::MovePtr(offset) => {
                if offset == 1 {
                    result += &emit("p++;", level)
                } else if offset == -1 {
                    result += &emit("p--;", level)
                } else {
                    result += &emit(&format!("p {}= {};", sign(offset), i32::abs(offset)), level)
                }
            }
            Command::Input(offset) => result += &emit(&format!("p[{}] = read();", offset), level),
            Command::Output(offset) => result += &emit(&format!("putchar(p[{}]);", offset), level),
            Command::If(commands) => {
                result += &emit("if (*p != 0) {", level);
                result += &emit_commands(&commands, level + 1);
                result += &emit("}", level)
            }
            Command::Loop(commands) => {
                result += &emit("while (*p != 0) {", level);
                result += &emit_commands(&commands, level + 1);
                result += &emit("}", level)
            }
        }
    }
    result
}
