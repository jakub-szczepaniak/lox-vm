use std::{
    io::{self, stdout, BufRead, Write},
    path::PathBuf,
};

use chunk::*;
use vm::*;

mod chunk;
mod compiler;
mod scanner;
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
        println!("{}", filename.to_str().unwrap())
    } else {
        println!("Starting Lox Repl");
        repl(&mut vm);
    }

    vm.free();
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
