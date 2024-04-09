use std::{
    io::{self, stdout, BufRead, Write},
    path::PathBuf,
};

use vm::*;

mod chunk;
mod compiler;
mod scanner;
mod token;
mod value;
mod vm;
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about= None)]
struct Cli {
    //path to .lox file for compilation
    filename: Option<PathBuf>,
}

fn main() {
    let mut vm = VM::new();

    let cli = Cli::parse();

    if let Some(filename) = cli.filename {
        let path = filename.to_str().expect("Expected non-empty path");
        run_file(&mut vm, path).expect("Could not run file");
    } else {
        println!("Starting Lox Repl");
        repl(&mut vm);
    }

    vm.free();
}

fn run_file(vm: &mut VM, path: &str) -> io::Result<()> {
    let buf = std::fs::read_to_string(path)?;
    let res = vm.interpret(&buf);
    match res {
        Err(InterpretResult::CompilerError) => std::process::exit(65),
        Err(InterpretResult::RuntimeError) => std::process::exit(70),
        Ok(_) => std::process::exit(0),
    }
}

fn repl(vm: &mut VM) {
    let stdin = io::stdin();
    print!("lox:>");
    let _ = stdout().flush();
    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            if line.is_empty() {
                break;
            }
            println!("{line}");
            let _ = vm.interpret(&line);
        } else {
            break;
        }
        print!("lox:>");
        let _ = stdout().flush();
    }
}
