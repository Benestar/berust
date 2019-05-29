use crate::playfield::*;
use rand::distributions;
use std::io;

/// A Befunge interpreter
pub struct Interpreter {
    nav: PlayfieldNavigator,
    stack: Vec<u8>,
}

impl Interpreter {
    /// Create a new interpreter for the given playfield.
    pub fn new(field: Playfield) -> Self {
        Self {
            nav: PlayfieldNavigator::new(field),
            stack: Vec::new(),
        }
    }

    /// Run the program.
    pub fn run(&mut self) {
        while self.read() {
            self.nav.step();
        }
    }

    fn read(&mut self) -> bool {
        match self.nav.get() as char {
            // Push this number on the stack
            '0'...'9' => self.stack.push(self.nav.get() - 0x30),

            // Addition: Pop a and b, then push a+b
            '+' => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();

                self.stack.push(a + b);
            }

            // Subtraction: Pop a and b, then push b-a
            '-' => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();

                self.stack.push(b - a);
            }

            // Multiplication: Pop a and b, then push a*b
            '*' => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();

                self.stack.push(a * b);
            }

            // Integer division: Pop a and b, then push b/a, rounded towards 0
            '/' => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();

                self.stack.push(b / a);
            }

            // Modulo: Pop a and b, then push the remainder of the integer division of b/a
            '%' => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();

                self.stack.push(b % a);
            }

            // Logical NOT: Pop a value. If the value is zero, push 1; otherwise, push zero.
            '!' => {
                if self.stack.pop().unwrap() == 0 {
                    self.stack.push(1)
                } else {
                    self.stack.push(0)
                }
            }

            // Greater than: Pop a and b, then push 1 if b>a, otherwise zero.
            '`' => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();

                if b > a {
                    self.stack.push(1)
                } else {
                    self.stack.push(0)
                }
            }

            // Start moving right
            '>' => self.nav.turn(Direction::Right),

            // Start moving left
            '<' => self.nav.turn(Direction::Left),

            // Start moving up
            '^' => self.nav.turn(Direction::Up),

            // Start moving down
            'v' => self.nav.turn(Direction::Down),

            // Start moving in a random cardinal direction
            '?' => self.nav.turn(rand::random()),

            // Pop a value; move right if value=0, left otherwise
            '_' => {
                if self.stack.pop().unwrap() == 0 {
                    self.nav.turn(Direction::Right)
                } else {
                    self.nav.turn(Direction::Left)
                }
            }

            // Pop a value; move down if value=0, up otherwise
            '|' => {
                if self.stack.pop().unwrap() == 0 {
                    self.nav.turn(Direction::Down)
                } else {
                    self.nav.turn(Direction::Up)
                }
            }

            // Start string mode: push each character's ASCII value all the way up to the next "
            '"' => self.read_string(),

            // Duplicate value on top of the stack
            ':' => {
                let v = self.stack.pop().unwrap();

                self.stack.push(v);
                self.stack.push(v);
            }

            // Swap two values on top of the stack
            '\\' => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();

                self.stack.push(a);
                self.stack.push(b);
            }

            // Pop value from the stack and discard it
            '$' => {
                self.stack.pop().unwrap();
            }

            // Pop value and output as an integer followed by a space
            '.' => print!("{} ", self.stack.pop().unwrap()),

            // Pop value and output as ASCII character
            ',' => print!("{}", self.stack.pop().unwrap() as char),

            // Bridge: Skip next cell
            '#' => {
                self.nav.step();
            }

            // A "put" call (a way to store a value for later use).
            //
            // Pop y, x, and v, then change the character at (x,y) in the program to the character
            // with ASCII value v
            'p' => {
                let y = self.stack.pop().unwrap();
                let x = self.stack.pop().unwrap();
                let v = self.stack.pop().unwrap();

                self.nav.field[(x as usize, y as usize)] = v
            }

            // A "get" call (a way to retrieve data in storage).
            //
            // Pop y and x, then push ASCII value of the character at that position in the program
            'g' => {
                let y = self.stack.pop().unwrap();
                let x = self.stack.pop().unwrap();

                self.stack.push(self.nav.field[(x as usize, y as usize)])
            }

            // Ask user for a number and push it
            '&' => {
                let mut input = String::new();

                io::stdin().read_line(&mut input).unwrap();

                self.stack.push(input.parse().unwrap())
            }

            // Ask user for a character and push its ASCII value
            '~' => {
                let mut input = String::new();

                io::stdin().read_line(&mut input).unwrap();

                self.stack.extend(input.trim().bytes())
            }

            // End program
            '@' => return false,

            // No-op. Does nothing
            _ => (),
        }

        true
    }

    fn read_string(&mut self) {
        self.nav.step();

        while self.nav.get() as char != '"' {
            self.stack.push(self.nav.get());

            self.nav.step();
        }
    }
}

impl distributions::Distribution<Direction> for distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0, 4) {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            _ => Direction::Right,
        }
    }
}
