#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let mut stdout = io::stdout();
    let stdin = io::stdin();

    print!("$ ");
    stdout.flush().unwrap();

    // Wait for user input
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();

    // Check if command is invalid
    println!("{}: command not found", input.trim())
}
