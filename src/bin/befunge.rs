extern crate berust;

use berust::interpreter::{InputOutput, Interpreter};
use berust::playfield::Playfield;
use std::env;
use std::fs::File;
use std::io;
use std::io::Read;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: ./befunge <file>");

        process::exit(1);
    }

    let mut file = File::open(&args[1]).unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let playfield = Playfield::new(&contents);
    let io = InputOutput::new(io::stdin(), io::stdout());

    let interpreter = Interpreter::new(playfield, io);

    for _ in interpreter {}
}
