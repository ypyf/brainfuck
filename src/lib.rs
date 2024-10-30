use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::str::Chars;

#[path = "./gen-c.rs"]
pub mod gen_c;

#[derive(Debug)]
pub struct Machine {
    memory: [u8; 30000],
    i: i32,
}

impl Machine {
    pub fn new() -> Self {
        Machine {
            memory: [0; 30000],
            i: 1000, // keeps the pointer from out of bound
        }
    }
    pub fn get(&self, offset: i32) -> i32 {
        self.memory[(self.i + offset) as usize] as i32
    }

    pub fn set(&mut self, offset: i32, value: i32) {
        self.memory[(self.i + offset) as usize] = value as u8
    }

    pub fn add(&mut self, offset: i32, value: i32) {
        let pos = (self.i + offset) as usize;
        let lhs = self.memory[pos] as i32;
        self.memory[pos] = (lhs + value) as u8
    }

    pub fn move_ptr(&mut self, offset: i32) {
        self.i += offset
    }

    pub fn execute(&mut self, program: &[Command]) {
        for cmd in program {
            match *cmd {
                Command::Add(offset, value) => self.add(offset, value),
                Command::MultAdd(src, dest, value) => {
                    let tmp = self.get(src) * value;
                    self.add(dest, tmp);
                }
                Command::Assign(offset, value) => self.set(offset, value),
                Command::MultAssign(src, dest, value) => {
                    let tmp = self.get(src) * value;
                    self.set(dest, tmp);
                }
                Command::MovePtr(offset) => self.move_ptr(offset),
                Command::Input(offset) => {
                    let ch = std::io::stdin()
                        .bytes()
                        .next()
                        .and_then(|ch| ch.ok())
                        .map(|ch| ch as i32)
                        .unwrap_or(0);
                    self.set(offset, ch)
                }
                Command::Output(offset) => {
                    let ch = self.get(offset) as u8 as char;
                    print!("{}", ch);
                }
                Command::If(ref commands) => {
                    if self.get(0) != 0 {
                        self.execute(commands)
                    }
                }
                Command::Loop(ref commands) => {
                    while self.get(0) != 0 {
                        self.execute(commands)
                    }
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Error {
    pub message: &'static str,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Command {
    Assign(i32, i32),          // [a] = b
    MultAssign(i32, i32, i32), // [b] = [a] * c
    MultAdd(i32, i32, i32),    // [b] += [a] * c
    Add(i32, i32),             // +/-
    MovePtr(i32),              // >/<
    Output(i32),               // ,
    Input(i32),                // .
    If(Vec<Command>),
    Loop(Vec<Command>),
}

pub struct Compiler;

impl Compiler {
    pub fn compile(source: &str) -> Result<Vec<Command>, Error> {
        let mut result = Compiler::parse(source.chars())?;
        result = Compiler::optimize(&result);
        result = Compiler::optimize(&result);
        result = Compiler::optimize(&result);
        Ok(result)
    }

    fn parse(mut chars: Chars<'_>) -> Result<Vec<Command>, Error> {
        let mut result: Vec<Command> = vec![];
        while let Some(ch) = chars.next() {
            let cmd = if let Some(cmd) = Compiler::parse_command(ch) {
                cmd
            } else {
                match ch {
                    '[' => {
                        let body = Compiler::parse_loop_body(&mut chars)?;
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

    fn parse_loop_body(mut chars: &mut Chars<'_>) -> Result<Vec<Command>, Error> {
        let mut result: Vec<Command> = vec![];
        while let Some(ch) = chars.next() {
            let cmd = if let Some(cmd) = Compiler::parse_command(ch) {
                cmd
            } else {
                match ch {
                    '[' => {
                        let body = Compiler::parse_loop_body(&mut chars)?;
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

    fn parse_command(ch: char) -> Option<Command> {
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

    fn optimize(program: &[Command]) -> Vec<Command> {
        let mut result: Vec<Command> = vec![];
        let mut bp = 0;
        for cmd in program {
            match *cmd {
                Command::Assign(offset, value) => {
                    let actual_offset = bp + offset;
                    if let Some(prev) = result.last_mut() {
                        match *prev {
                            Command::Add(prev_offset, _) | Command::Assign(prev_offset, _)
                                if prev_offset == actual_offset =>
                            {
                                result.pop();
                            }
                            Command::MultAdd(_, prev_dest, _)
                            | Command::MultAssign(_, prev_dest, _)
                                if prev_dest == actual_offset =>
                            {
                                result.pop();
                            }
                            _ => (),
                        }
                    }
                    result.push(Command::Assign(actual_offset, value))
                }
                Command::MultAssign(src, dest, value) => {
                    result.push(Command::MultAssign(bp + src, bp + dest, value))
                }
                Command::Add(offset, cmd_value) => {
                    let actual_offset = bp + offset;
                    let mut fused = false;
                    if let Some(prev) = result.last_mut() {
                        fused = Compiler::merge_add(prev, actual_offset, cmd_value)
                    }
                    if !fused {
                        result.push(Command::Add(actual_offset, cmd_value))
                    }
                }
                Command::MultAdd(src, dest, value) => {
                    let actual_offset = bp + dest;
                    if let Some(prev) = result.last_mut() {
                        match *prev {
                            Command::Assign(prev_offset, prev_value) => {
                                if prev_offset == actual_offset && prev_value == 0 {
                                    result.pop();
                                    result.push(Command::MultAssign(
                                        bp + src,
                                        actual_offset,
                                        value,
                                    ));
                                    continue;
                                }
                            }
                            _ => (),
                        }
                    }
                    result.push(Command::MultAdd(bp + src, actual_offset, value))
                }
                Command::MovePtr(offset) => bp += offset,
                Command::Input(offset) => result.push(Command::Input(bp + offset)),
                Command::Output(offset) => result.push(Command::Output(bp + offset)),
                _ => {
                    if bp != 0 {
                        result.push(Command::MovePtr(bp));
                        bp = 0
                    }
                    match cmd {
                        Command::Loop(commands) => {
                            if let Some(commands) = Compiler::eliminate_loop(commands) {
                                result.extend(commands)
                            } else {
                                let cmd = Compiler::optimize_complex_loop(commands)
                                    .unwrap_or(Command::Loop(Compiler::optimize(commands)));
                                result.push(cmd)
                            }
                        }
                        Command::If(commands) => {
                            result.push(Command::If(Compiler::optimize(commands)))
                        }
                        _ => unreachable!(),
                    };
                }
            }
        }

        if bp != 0 {
            result.push(Command::MovePtr(bp));
        }
        result
    }

    fn eliminate_loop(commands: &[Command]) -> Option<Vec<Command>> {
        let mut deltas: BTreeMap<i32, i32> = BTreeMap::new();
        let mut bp = 0;
        for cmd in commands {
            match *cmd {
                Command::Add(offset, value) => {
                    let key = bp + offset;
                    let old = deltas.entry(key).or_default();
                    *old += value;
                }
                Command::MovePtr(offset) => bp += offset,
                _ => return None,
            }
        }

        if (bp != 0) || (*deltas.get(&0).unwrap_or(&0) != -1) {
            return None;
        }

        let mut result: Vec<Command> = vec![];
        deltas.remove(&0);
        for key in deltas.keys() {
            result.push(Command::MultAdd(0, *key, deltas[key]))
        }
        result.push(Command::Assign(0, 0));
        Some(result)
    }

    fn optimize_complex_loop(commands: &[Command]) -> Option<Command> {
        let mut result: Vec<Command> = vec![];
        let mut origin_delta = 0;
        let mut clears: BTreeSet<i32> = BTreeSet::new();
        clears.insert(0);

        for cmd in commands {
            match *cmd {
                Command::Add(offset, value) => {
                    if offset == 0 {
                        origin_delta += value
                    } else {
                        clears.remove(&offset);
                        result.push(Command::MultAdd(0, offset, value))
                    }
                }
                Command::MultAdd(_, dest, _) | Command::MultAssign(_, dest, _) => {
                    if dest == 0 {
                        return None;
                    }
                    clears.remove(&dest);
                    result.push(cmd.clone())
                }
                Command::Assign(offset, value) => {
                    if offset == 0 {
                        return None;
                    } else if value == 0 {
                        clears.insert(offset);
                    } else {
                        clears.remove(&offset);
                    }
                    result.push(cmd.clone())
                }
                _ => return None,
            }
        }

        if origin_delta != -1 {
            return None;
        }

        for cmd in &result {
            match cmd {
                Command::MultAdd(src, _, _) | Command::MultAssign(src, _, _)
                    if !clears.contains(&src) =>
                {
                    return None
                }
                _ => continue,
            }
        }

        result.push(Command::Assign(0, 0));
        Some(Command::If(result))
    }

    fn merge_add(prev: &mut Command, offset: i32, value: i32) -> bool {
        let mut fused = false;
        match prev {
            Command::Add(prev_offset, prev_value) | Command::Assign(prev_offset, prev_value) => {
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

#[cfg(test)]
mod tests {
    use super::Command::*;
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
    fn it_parses_loops() {
        let result = vec![Command::Loop(vec![
            Command::Add(0, 1),
            Command::Loop(vec![Command::Add(0, -1)]),
            Command::Add(0, 1),
        ])];
        assert_eq!(Compiler::parse("[+[-]+]".chars()), Ok(result));
    }

    #[test]
    fn it_optimizes() {
        let result = vec![
            Add(0, 3),
            Loop(vec![
                Output(0),
                Add(1, 3),
                MultAdd(1, 4, 1),
                Assign(1, 0),
                Add(0, -1),
            ]),
        ];
        let program = Compiler::compile("+++[.>+++[>>>+<<<-]<-]").unwrap();
        assert_eq!(program, result);
        let mut machine = Machine::new();
        machine.execute(&program);
        assert_eq!(machine.memory[0], 0);
    }
}
