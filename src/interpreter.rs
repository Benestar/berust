use crate::playfield::*;
use rand::distributions;
use std::io::{BufRead, BufReader, Read, Write};

/// The current mode of the program
///
/// A program is either executing normally, parsing a string or has terminated.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Execute,
    Parse,
    Terminate,
}

/// The stack of an execution.
pub type Stack = Vec<i64>;

/// A provider of input and output operations.
pub struct InputOutput<R, W> {
    reader: BufReader<R>,
    writer: W,
}

impl<R, W> InputOutput<R, W>
where
    R: Read,
    W: Write,
{
    pub fn new(reader: R, writer: W) -> Self {
        let reader = BufReader::new(reader);

        Self { reader, writer }
    }

    pub fn reader(&self) -> &R {
        self.reader.get_ref()
    }

    pub fn writer(&self) -> &W {
        &self.writer
    }

    fn write_int(&mut self, val: i64) {
        write!(self.writer, "{} ", val).unwrap()
    }

    fn write_ascii(&mut self, val: i64) {
        write!(self.writer, "{}", val as u8 as char).unwrap()
    }

    fn read_int(&mut self) -> i64 {
        let mut input = String::new();

        if self.reader.read_line(&mut input).is_err() {
            return 0;
        }

        input.trim().parse().unwrap_or(0)
    }

    fn read_ascii(&mut self) -> i64 {
        let mut buf = [0; 1];

        if self.reader.read_exact(&mut buf).is_err() {
            return -1;
        }

        i64::from(buf[0])
    }
}

/// A Befunge interpreter
pub struct Interpreter<R, W> {
    field: Playfield,
    io: InputOutput<R, W>,
    nav: PlayfieldNavigator,
    stack: Stack,
    mode: Mode,
}

impl<R, W> Interpreter<R, W>
where
    R: Read,
    W: Write,
{
    /// Create a new interpreter for the given playfield.
    pub fn new(field: Playfield, io: InputOutput<R, W>) -> Self {
        let dimensions = field.dimensions();

        Self {
            field,
            io,
            nav: PlayfieldNavigator::new(dimensions),
            stack: Vec::new(),
            mode: Mode::Execute,
        }
    }

    /// Get a reference to the playfield.
    pub fn field(&self) -> &Playfield {
        &self.field
    }

    /// Get a reference to the input and output provider.
    pub fn io(&self) -> &InputOutput<R, W> {
        &self.io
    }

    /// Get a reference to the navigator.
    pub fn nav(&self) -> &PlayfieldNavigator {
        &self.nav
    }

    /// Get a reference to the stack.
    pub fn stack(&self) -> &Stack {
        &self.stack
    }

    /// Get the current mode.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    fn execute_step(&mut self, c: u8) -> Mode {
        match c {
            // Push this number on the stack
            b'0'...b'9' => self.stack.push(i64::from(c - 0x30)),

            // Addition: Pop a and b, then push a+b
            b'+' => {
                let a = self.stack.pop().unwrap_or(0);
                let b = self.stack.pop().unwrap_or(0);

                self.stack.push(a + b);
            }

            // Subtraction: Pop a and b, then push b-a
            b'-' => {
                let a = self.stack.pop().unwrap_or(0);
                let b = self.stack.pop().unwrap_or(0);

                self.stack.push(b - a);
            }

            // Multiplication: Pop a and b, then push a*b
            b'*' => {
                let a = self.stack.pop().unwrap_or(0);
                let b = self.stack.pop().unwrap_or(0);

                self.stack.push(a * b);
            }

            // Integer division: Pop a and b, then push b/a, rounded towards 0
            b'/' => {
                let a = self.stack.pop().unwrap_or(0);
                let b = self.stack.pop().unwrap_or(0);

                self.stack.push(b / a);
            }

            // Modulo: Pop a and b, then push the remainder of the integer division of b/a
            b'%' => {
                let a = self.stack.pop().unwrap_or(0);
                let b = self.stack.pop().unwrap_or(0);

                self.stack.push(b % a);
            }

            // Logical NOT: Pop a value. If the value is zero, push 1; otherwise, push zero.
            b'!' => {
                if self.stack.pop().unwrap_or(0) == 0 {
                    self.stack.push(1)
                } else {
                    self.stack.push(0)
                }
            }

            // Greater than: Pop a and b, then push 1 if b>a, otherwise zero.
            b'`' => {
                let a = self.stack.pop().unwrap_or(0);
                let b = self.stack.pop().unwrap_or(0);

                if b > a {
                    self.stack.push(1)
                } else {
                    self.stack.push(0)
                }
            }

            // Start moving right
            b'>' => self.nav.turn(Direction::Right),

            // Start moving left
            b'<' => self.nav.turn(Direction::Left),

            // Start moving up
            b'^' => self.nav.turn(Direction::Up),

            // Start moving down
            b'v' => self.nav.turn(Direction::Down),

            // Start moving in a random cardinal direction
            b'?' => self.nav.turn(rand::random()),

            // Pop a value; move right if value=0, left otherwise
            b'_' => {
                if self.stack.pop().unwrap_or(0) == 0 {
                    self.nav.turn(Direction::Right)
                } else {
                    self.nav.turn(Direction::Left)
                }
            }

            // Pop a value; move down if value=0, up otherwise
            b'|' => {
                if self.stack.pop().unwrap_or(0) == 0 {
                    self.nav.turn(Direction::Down)
                } else {
                    self.nav.turn(Direction::Up)
                }
            }

            // Start string mode: push each character's ASCII value all the way up to the next "
            b'"' => return Mode::Parse,

            // Duplicate value on top of the stack
            b':' => {
                let v = self.stack.pop().unwrap_or(0);

                self.stack.push(v);
                self.stack.push(v);
            }

            // Swap two values on top of the stack
            b'\\' => {
                let a = self.stack.pop().unwrap_or(0);
                let b = self.stack.pop().unwrap_or(0);

                self.stack.push(a);
                self.stack.push(b);
            }

            // Pop value from the stack and discard it
            b'$' => {
                self.stack.pop();
            }

            // Pop value and output as an integer followed by a space
            b'.' => self.io.write_int(self.stack.pop().unwrap_or(0)),

            // Pop value and output as ASCII character
            b',' => self.io.write_ascii(self.stack.pop().unwrap_or(0)),

            // Bridge: Skip next cell
            b'#' => self.nav.step(),

            // A "put" call (a way to store a value for later use).
            //
            // Pop y, x, and v, then change the character at (x,y) in the program to the character
            // with ASCII value v
            b'p' => {
                let y = self.stack.pop().unwrap_or(0);
                let x = self.stack.pop().unwrap_or(0);
                let v = self.stack.pop().unwrap_or(0);

                self.field[(x as usize, y as usize)] = v as u8
            }

            // A "get" call (a way to retrieve data in storage).
            //
            // Pop y and x, then push ASCII value of the character at that position in the program
            b'g' => {
                let y = self.stack.pop().unwrap_or(0);
                let x = self.stack.pop().unwrap_or(0);
                let v = self.field[(x as usize, y as usize)];

                self.stack.push(i64::from(v))
            }

            // Ask user for a number and push it
            b'&' => self.stack.push(self.io.read_int()),

            // Ask user for a character and push its ASCII value
            b'~' => self.stack.push(self.io.read_ascii()),

            // End program
            b'@' => return Mode::Terminate,

            // No-op. Does nothing
            b' ' => (),

            // Illegal characters
            _ => panic!("Illegal character: {}", c as char),
        }

        Mode::Execute
    }

    fn parse_step(&mut self, c: u8) -> Mode {
        if let b'"' = c {
            return Mode::Execute;
        }

        self.stack.push(i64::from(c));

        Mode::Parse
    }
}

impl<R, W> Iterator for Interpreter<R, W>
where
    R: Read,
    W: Write,
{
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.field[self.nav.pos()];

        self.mode = match self.mode {
            Mode::Execute => self.execute_step(val),
            Mode::Parse => self.parse_step(val),
            Mode::Terminate => Mode::Terminate,
        };

        if let Mode::Terminate = self.mode {
            return None;
        }

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

    fn test_program(field: &str, input: &str, output: &str, execution: Vec<(Mode, Stack)>) {
        let reader = input.bytes().collect::<Vec<_>>();
        let writer = Vec::new();

        let playfield = Playfield::new(field);
        let io = InputOutput::new(&reader[..], writer);
        let mut interpreter = Interpreter::new(playfield, io);

        for (mode, stack) in execution {
            assert_eq!(mode, interpreter.mode);
            assert_eq!(stack, interpreter.stack);

            interpreter.next();
        }

        assert_eq!(output.bytes().collect::<Vec<_>>(), interpreter.io.writer);
    }

    #[test]
    fn interpret_digits() {
        test_program(
            "0123456789",
            "",
            "",
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
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![7]),
                (Mode::Execute, vec![7, 3]),
                (Mode::Execute, vec![10]),
            ],
        );

        test_program(
            "73-",
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![7]),
                (Mode::Execute, vec![7, 3]),
                (Mode::Execute, vec![4]),
            ],
        );

        test_program(
            "73*",
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![7]),
                (Mode::Execute, vec![7, 3]),
                (Mode::Execute, vec![21]),
            ],
        );

        test_program(
            "73/",
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![7]),
                (Mode::Execute, vec![7, 3]),
                (Mode::Execute, vec![2]),
            ],
        );

        test_program(
            "73%",
            "",
            "",
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
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![0]),
                (Mode::Execute, vec![1]),
            ],
        );

        test_program(
            "5!",
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![5]),
                (Mode::Execute, vec![0]),
            ],
        );

        test_program(
            "73`",
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![7]),
                (Mode::Execute, vec![7, 3]),
                (Mode::Execute, vec![1]),
            ],
        );

        test_program(
            "45`",
            "",
            "",
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
            "",
            "",
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
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![5]),
            ],
        );

        test_program(
            "?5@5\n5\n@\n5",
            "",
            "",
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
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![0]),
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![5]),
            ],
        );

        test_program(
            "3_5",
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![3]),
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![3]),
            ],
        );

        test_program(
            "0|\n 5",
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![0]),
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![5]),
            ],
        );

        test_program(
            "3|\n 5\n 4",
            "",
            "",
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
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Parse, vec![]),
                (Mode::Parse, vec![0x61]),
                (Mode::Parse, vec![0x61, 0x62]),
                (Mode::Parse, vec![0x61, 0x62, 0x63]),
                (Mode::Execute, vec![0x61, 0x62, 0x63]),
            ],
        );

        test_program(
            "1\"23",
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![1]),
                (Mode::Parse, vec![1]),
                (Mode::Parse, vec![1, 0x32]),
                (Mode::Parse, vec![1, 0x32, 0x33]),
                (Mode::Parse, vec![1, 0x32, 0x33, 0x31]),
                (Mode::Execute, vec![1, 0x32, 0x33, 0x31]),
            ],
        );

        test_program(
            "v\n\"\na\nb\n\"",
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![]),
                (Mode::Parse, vec![]),
                (Mode::Parse, vec![0x61]),
                (Mode::Parse, vec![0x61, 0x62]),
                (Mode::Execute, vec![0x61, 0x62]),
            ],
        );
    }

    #[test]
    fn interpret_stack_manipulation() {
        test_program(
            "1:",
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![1]),
                (Mode::Execute, vec![1, 1]),
            ],
        );

        test_program(
            "12\\",
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![1]),
                (Mode::Execute, vec![1, 2]),
                (Mode::Execute, vec![2, 1]),
            ],
        );

        test_program(
            "1$",
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![1]),
                (Mode::Execute, vec![]),
            ],
        );
    }

    #[test]
    fn interpret_output() {
        test_program(
            "1.",
            "",
            "1 ",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![1]),
                (Mode::Execute, vec![]),
            ],
        );

        test_program(
            "\"a\",25*,",
            "",
            "a\n",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Parse, vec![]),
                (Mode::Parse, vec![0x61]),
                (Mode::Execute, vec![0x61]),
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![2]),
                (Mode::Execute, vec![2, 5]),
                (Mode::Execute, vec![10]),
                (Mode::Execute, vec![]),
            ],
        );
    }

    #[test]
    fn interpret_bridge() {
        test_program(
            "#01",
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![1]),
            ],
        );
    }

    #[test]
    fn interpret_field_manipulation() {
        test_program(
            "30g5",
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![3]),
                (Mode::Execute, vec![3, 0]),
                (Mode::Execute, vec![0x35]),
                (Mode::Execute, vec![0x35, 5]),
            ],
        );

        test_program(
            "77*40p1",
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![7]),
                (Mode::Execute, vec![7, 7]),
                (Mode::Execute, vec![49]),
                (Mode::Execute, vec![49, 4]),
                (Mode::Execute, vec![49, 4, 0]),
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![1]),
            ],
        );
    }

    #[test]
    fn interpret_user_input() {
        test_program(
            "&",
            "4\n",
            "",
            vec![(Mode::Execute, vec![]), (Mode::Execute, vec![4])],
        );

        test_program(
            "&",
            "abc\n",
            "",
            vec![(Mode::Execute, vec![]), (Mode::Execute, vec![0])],
        );

        test_program(
            "&",
            "",
            "",
            vec![(Mode::Execute, vec![]), (Mode::Execute, vec![0])],
        );

        test_program(
            "~~~",
            "ab",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![0x61]),
                (Mode::Execute, vec![0x61, 0x62]),
                (Mode::Execute, vec![0x61, 0x62, -1]),
            ],
        );
    }

    #[test]
    fn interpret_termination() {
        test_program(
            "@",
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Terminate, vec![]),
                (Mode::Terminate, vec![]),
            ],
        );

        test_program(
            "5@",
            "",
            "",
            vec![
                (Mode::Execute, vec![]),
                (Mode::Execute, vec![5]),
                (Mode::Terminate, vec![5]),
                (Mode::Terminate, vec![5]),
            ],
        );
    }

    #[test]
    #[should_panic(expected = "Illegal character: x")]
    fn interpret_illegal() {
        test_program("x", "", "", vec![(Mode::Execute, vec![])]);
    }
}
