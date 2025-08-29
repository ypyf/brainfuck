# Brainfuck Compiler with Optimizer

A Brainfuck compiler and interpreter written in Rust, featuring an **optimizer** to improve program performance. This project allows you to execute Brainfuck programs, start an interactive REPL session, and generate optimized C code from Brainfuck scripts.

## Features

* **Execute Brainfuck programs** directly from a file.
* **Interactive REPL** for experimenting with Brainfuck code.
* **Generate optimized C code** from Brainfuck programs.
* **Built-in optimizer** reduces unnecessary instructions and improves performance.

## Installation

Make sure you have [Rust](https://www.rust-lang.org/tools/install) installed.

Clone the repository and build the project:

```bash
git clone <repository-url>
cd <project-directory>
cargo build --release
```

The compiled binary will be located at `target/release/bf`.

## Usage

```bash
bf [command] [options]
```

### Commands

* `run <filename>`
  Execute a Brainfuck script from a file. The optimizer is applied automatically to improve execution speed.

  ```bash
  bf run hello.bf
  ```

* `repl`
  Start an interactive Brainfuck REPL session. Optimizations are applied to your code in real-time.

  ```bash
  bf repl
  ```

* `gen-c <filename>`
  Generate optimized C source code from a Brainfuck script. The output file will have the same name as the input file but with a `.c` extension.

  ```bash
  bf gen-c hello.bf
  gcc hello.c -o hello
  ./hello
  ```

### Example

The examples directory contains several Brainfuck scripts demonstrating different features.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
