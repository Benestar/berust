use crate::playfield::*;
use rand::distributions;
use std::io;

/// The current mode of the program
///
/// A program is either executing normally, parsing a string or has terminated.
#[derive(Debug, PartialEq)]
enum Mode {
    Execute,
    String,
    Terminate,
}

/// The stack of an execution.
pub type Stack = Vec<i64>;

/// A Befunge interpreter
pub struct Interpreter {
    field: Playfield,
    nav: PlayfieldNavigator,
    stack: Stack,
    mode: Mode,
}

impl Interpreter {
    /// Create a new interpreter for the given playfield.
    pub fn new(field: Playfield) -> Self {
        let dimensions = field.dimensions();

        Self {
            field,
            nav: PlayfieldNavigator::new(dimensions),
            stack: Vec::new(),
            mode: Mode::Execute,
        }
    }

    fn execute_step(&mut self, c: u8) -> Mode {
        match c as char {
            // Push this number on the stack
            '0'...'9' => self.stack.push(i64::from(c - 0x30)),

            // Addition: Pop a and b, then push a+b
            '+' => {
                let a = self.stack.pop().unwrap_or(0);
                let b = self.stack.pop().unwrap_or(0);

                self.stack.push(a + b);
            }

            // Subtraction: Pop a and b, then push b-a
            '-' => {
                let a = self.stack.pop().unwrap_or(0);
                let b = self.stack.pop().unwrap_or(0);

                self.stack.push(b - a);
            }

            // Multiplication: Pop a and b, then push a*b
            '*' => {
                let a = self.stack.pop().unwrap_or(0);
                let b = self.stack.pop().unwrap_or(0);

                self.stack.push(a * b);
            }

            // Integer division: Pop a and b, then push b/a, rounded towards 0
            '/' => {
                let a = self.stack.pop().unwrap_or(0);
                let b = self.stack.pop().unwrap_or(0);

                self.stack.push(b / a);
            }

            // Modulo: Pop a and b, then push the remainder of the integer division of b/a
            '%' => {
                let a = self.stack.pop().unwrap_or(0);
                let b = self.stack.pop().unwrap_or(0);

                self.stack.push(b % a);
            }

            // Logical NOT: Pop a value. If the value is zero, push 1; otherwise, push zero.
            '!' => {
                if self.stack.pop().unwrap_or(0) == 0 {
                    self.stack.push(1)
                } else {
                    self.stack.push(0)
                }
            }

            // Greater than: Pop a and b, then push 1 if b>a, otherwise zero.
            '`' => {
                let a = self.stack.pop().unwrap_or(0);
                let b = self.stack.pop().unwrap_or(0);

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
                if self.stack.pop().unwrap_or(0) == 0 {
                    self.nav.turn(Direction::Right)
                } else {
                    self.nav.turn(Direction::Left)
                }
            }

            // Pop a value; move down if value=0, up otherwise
            '|' => {
                if self.stack.pop().unwrap_or(0) == 0 {
                    self.nav.turn(Direction::Down)
                } else {
                    self.nav.turn(Direction::Up)
                }
            }

            // Start string mode: push each character's ASCII value all the way up to the next "
            '"' => return Mode::String,

            // Duplicate value on top of the stack
            ':' => {
                let v = self.stack.pop().unwrap_or(0);

                self.stack.push(v);
                self.stack.push(v);
            }

            // Swap two values on top of the stack
            '\\' => {
                let a = self.stack.pop().unwrap_or(0);
                let b = self.stack.pop().unwrap_or(0);

                self.stack.push(a);
                self.stack.push(b);
            }

            // Pop value from the stack and discard it
            '$' => {
                self.stack.pop().unwrap_or(0);
            }

            // Pop value and output as an integer followed by a space
            '.' => print!("{} ", self.stack.pop().unwrap_or(0)),

            // Pop value and output as ASCII character
            ',' => print!("{}", self.stack.pop().unwrap_or(0) as u8 as char),

            // Bridge: Skip next cell
            '#' => self.nav.step(),

            // A "put" call (a way to store a value for later use).
            //
            // Pop y, x, and v, then change the character at (x,y) in the program to the character
            // with ASCII value v
            'p' => {
                let y = self.stack.pop().unwrap_or(0);
                let x = self.stack.pop().unwrap_or(0);
                let v = self.stack.pop().unwrap_or(0);

                self.field[(x as usize, y as usize)] = v as u8
            }

            // A "get" call (a way to retrieve data in storage).
            //
            // Pop y and x, then push ASCII value of the character at that position in the program
            'g' => {
                let y = self.stack.pop().unwrap_or(0);
                let x = self.stack.pop().unwrap_or(0);
                let v = self.field[(x as usize, y as usize)];

                self.stack.push(i64::from(v))
            }

            // Ask user for a number and push it
            '&' => {
                let mut input = String::new();

                io::stdin().read_line(&mut input).unwrap();

                self.stack.push(input.trim().parse().unwrap())
            }

            // Ask user for a character and push its ASCII value
            '~' => {
                let mut input = String::new();

                io::stdin().read_line(&mut input).unwrap();

                self.stack.extend(input.trim().bytes().map(i64::from))
            }

            // End program
            '@' => return Mode::Terminate,

            // No-op. Does nothing
            _ => (),
        }

        Mode::Execute
    }

    fn string_step(&mut self, c: u8) -> Mode {
        if c as char == '"' {
            return Mode::Execute;
        }

        self.stack.push(i64::from(c));

        Mode::String
    }
}

impl Iterator for Interpreter {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.field[self.nav.pos()];

        self.mode = match self.mode {
            Mode::Execute => self.execute_step(val),
            Mode::String => self.string_step(val),
            Mode::Terminate => return None,
        };

        self.nav.step();

        Some(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playfield::Playfield;

    fn test_program(input: &str, execution: Vec<(Mode, Vec<u8>)>) {
        let playfield = Playfield::new(input);
        let mut interpreter = Interpreter::new(playfield);

        for (mode, stack) in execution {
            assert_eq!(mode, interpreter.mode);
            assert_eq!(stack, interpreter.stack);

            interpreter.step();
        }
    }

    #[test]
    fn interpret_digits() {
        test_program(
            "0123456789",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![0]),
                (Mode::Execute, vec![0, 1]),
                (Mode::Execute, vec![0, 1, 2]),
                (Mode::Execute, vec![0, 1, 2, 3]),
                (Mode::Execute, vec![0, 1, 2, 3, 4]),
                (Mode::Execute, vec![0, 1, 2, 3, 4, 5]),
                (Mode::Execute, vec![0, 1, 2, 3, 4, 5, 6]),
                (Mode::Execute, vec![0, 1, 2, 3, 4, 5, 6, 7]),
                (Mode::Execute, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]),
                (Mode::Execute, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            ],
        );
    }

    #[test]
    fn interpret_arithmetic() {
        test_program(
            "73+",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![7]),
                (Mode::Execute, vec![7, 3]),
                (Mode::Execute, vec![10]),
            ],
        );

        test_program(
            "73-",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![7]),
                (Mode::Execute, vec![7, 3]),
                (Mode::Execute, vec![4]),
            ],
        );

        test_program(
            "73*",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![7]),
                (Mode::Execute, vec![7, 3]),
                (Mode::Execute, vec![21]),
            ],
        );

        test_program(
            "73/",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![7]),
                (Mode::Execute, vec![7, 3]),
                (Mode::Execute, vec![2]),
            ],
        );

        test_program(
            "73%",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![7]),
                (Mode::Execute, vec![7, 3]),
                (Mode::Execute, vec![1]),
            ],
        );
    }

    #[test]
    fn interpret_logic() {
        test_program(
            "0!",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![0]),
                (Mode::Execute, vec![1]),
            ],
        );

        test_program(
            "5!",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![5]),
                (Mode::Execute, vec![0]),
            ],
        );

        test_program(
            "73`",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![7]),
                (Mode::Execute, vec![7, 3]),
                (Mode::Execute, vec![1]),
            ],
        );

        test_program(
            "45`",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![4]),
                (Mode::Execute, vec![4, 5]),
                (Mode::Execute, vec![0]),
            ],
        );
    }

    #[test]
    fn interpret_direction() {
        test_program(
            "v\n3\n>4",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![3]),
                (Mode::Execute, vec![3]),
                (Mode::Execute, vec![3, 4]),
            ],
        );

        test_program(
            "<@^\n  @\n  5",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![5]),
            ],
        );

        // TODO

        test_program(
            "?5@5\n5\n@\n5",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![5]),
            ],
        );
    }

    #[test]
    fn interpret_controlflow() {
        test_program(
            "0_5",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![0]),
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![5]),
            ],
        );

        test_program(
            "3_5",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![3]),
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![3]),
            ],
        );

        test_program(
            "0|\n 5",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![0]),
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![5]),
            ],
        );

        test_program(
            "3|\n 5\n 4",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![3]),
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![4]),
            ],
        );
    }

    #[test]
    fn interpret_string() {
        test_program(
            "\"abc\"",
            vec![
                (Mode::Execute, vec![]),
                (Mode::String, vec![]),
                (Mode::String, vec![0x61]),
                (Mode::String, vec![0x61, 0x62]),
                (Mode::String, vec![0x61, 0x62, 0x63]),
                (Mode::Execute, vec![0x61, 0x62, 0x63]),
            ],
        );

        test_program(
            "1\"xy",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![1]),
                (Mode::String, vec![1]),
                (Mode::String, vec![1, 0x78]),
                (Mode::String, vec![1, 0x78, 0x79]),
                (Mode::String, vec![1, 0x78, 0x79, 0x31]),
                (Mode::Execute, vec![1, 0x78, 0x79, 0x31]),
            ],
        );

        test_program(
            "v\n\"\na\nb\n\"",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![]),
                (Mode::String, vec![]),
                (Mode::String, vec![0x61]),
                (Mode::String, vec![0x61, 0x62]),
                (Mode::Execute, vec![0x61, 0x62]),
            ],
        );
    }

    #[test]
    fn interpret_stack_manipulation() {
        test_program(
            "1:",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![1]),
                (Mode::Execute, vec![1, 1]),
            ],
        );

        test_program(
            "12\\",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![1]),
                (Mode::Execute, vec![1, 2]),
                (Mode::Execute, vec![2, 1]),
            ],
        );

        test_program(
            "1$",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![1]),
                (Mode::Execute, vec![]),
            ],
        );
    }

    #[test]
    fn interpret_output() {
        // TODO

        test_program(
            "1.",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![1]),
                (Mode::Execute, vec![]),
            ],
        );

        test_program(
            "\"a\",",
            vec![
                (Mode::Execute, vec![]),
                (Mode::String, vec![]),
                (Mode::String, vec![0x61]),
                (Mode::Execute, vec![0x61]),
                (Mode::Execute, vec![]),
            ],
        );
    }

    #[test]
    fn interpret_bridge() {
        test_program(
            "#01",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![1]),
            ],
        );
    }

    #[test]
    fn interpret_field_manipulation() {
        // TODO
    }

    #[test]
    fn interpret_user_input() {
        // TODO
    }

    #[test]
    fn interpret_termination() {
        test_program(
            "@",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Terminate, vec![]),
                (Mode::Terminate, vec![]),
            ],
        );

        test_program(
            "5@",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![5]),
                (Mode::Terminate, vec![5]),
                (Mode::Terminate, vec![5]),
            ],
        );
    }
}
