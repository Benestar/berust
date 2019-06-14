//! A Befunge interpreter written in Rust
//!
//! The [`playfield`] module provides all functionality to navigate through a Befunge program
//! and the [`interpreter`] module implements roughly the [Befunge-93 semantics].
//!
//! # Example
//!
//! ```
//! # use berust::interpreter::{InputOutput, Interpreter};
//! # use berust::playfield::Playfield;
//! let playfield = Playfield::new("23*.@");
//! let io = InputOutput::from_std();
//!
//! let interpreter = Interpreter::new(playfield, io);
//!
//! // prints 6 to stdout
//! for _ in interpreter {}
//! ```
//!
//! [`playfield`]: playfield/index.html
//! [`interpreter`]: interpreter/index.html
//! [Befunge-93 semantics]: https://en.wikipedia.org/wiki/Befunge#Befunge-93_instruction_list

extern crate rand;

pub mod interpreter;
pub mod playfield;
