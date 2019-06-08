use std::fmt;
use std::iter;
use std::ops;
use std::str;

/// A two-dimensional matrix of characters
#[derive(Debug)]
pub struct Playfield {
    field: Vec<u8>,
    width: usize,
    height: usize,
}

impl Playfield {
    /// Create a new playfield from the given input string.
    ///
    /// Each line in the input is padded with spaces to the length of the longest line.
    /// Width and height are defined as the length of the longest line and the number of lines in
    /// the input string.
    pub fn new(input: &str) -> Self {
        let lines: Vec<&str> = input.lines().collect();
        let width = lines.iter().map(|s| s.bytes().count()).max().unwrap();
        let height = lines.len();

        let mut field = Vec::with_capacity(width * height);

        for l in lines {
            field.extend(l.bytes().chain(iter::repeat(b' ')).take(width));
        }

        Self {
            field,
            width,
            height,
        }
    }

    /// Return the dimensions of this playfield.
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Return an iterator over the lines of this playfield.
    pub fn lines(&self) -> impl Iterator<Item = &[u8]> {
        self.field.chunks(self.width)
    }
}

impl ops::Index<(usize, usize)> for Playfield {
    type Output = u8;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.field[index.0 + self.width * index.1]
    }
}

impl ops::IndexMut<(usize, usize)> for Playfield {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.field[index.0 + self.width * index.1]
    }
}

impl fmt::Display for Playfield {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for l in self.lines() {
            writeln!(f, "{}", str::from_utf8(l).unwrap())?;
        }

        Ok(())
    }
}

/// The four movement directions
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// A navigator through the playfield
///
/// The navigator stores the current position and the direction at which we are looking.
pub struct PlayfieldNavigator {
    dim: (usize, usize),
    pos: (usize, usize),
    dir: Direction,
}

impl PlayfieldNavigator {
    /// Create a new navigator for the given dimensions.
    ///
    /// Initially, we are looking right from position `(0, 0)`.
    pub fn new(dim: (usize, usize)) -> Self {
        Self {
            dim,
            pos: (0, 0),
            dir: Direction::Right,
        }
    }

    /// Move one step in the field.
    ///
    /// When the border of the field is reached, the navigator wraps around and continues at the
    /// opposite side of the field.
    pub fn step(&mut self) {
        match self.dir {
            Direction::Up => {
                if self.pos.1 > 0 {
                    self.pos.1 -= 1
                } else {
                    self.pos.1 = self.dim.1 - 1
                }
            }
            Direction::Down => {
                if self.pos.1 < self.dim.1 - 1 {
                    self.pos.1 += 1
                } else {
                    self.pos.1 = 0
                }
            }
            Direction::Left => {
                if self.pos.0 > 0 {
                    self.pos.0 -= 1
                } else {
                    self.pos.0 = self.dim.0 - 1
                }
            }
            Direction::Right => {
                if self.pos.0 < self.dim.0 - 1 {
                    self.pos.0 += 1
                } else {
                    self.pos.0 = 0
                }
            }
        }
    }

    /// Turn into the given direction.
    pub fn turn(&mut self, dir: Direction) {
        self.dir = dir
    }

    /// Return the current position of the navigator.
    pub fn pos(&self) -> (usize, usize) {
        self.pos
    }

    /// Return the current direction the navigator is looking in.
    pub fn dir(&self) -> Direction {
        self.dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playfield() {
        let mut playfield = Playfield::new("abc\nde\nx yz\n");

        assert_eq!((4, 3), playfield.dimensions());
        assert_eq!("abc \nde  \nx yz\n", playfield.to_string());

        assert_eq!('a', playfield[(0, 0)] as char);
        assert_eq!(' ', playfield[(3, 1)] as char);

        playfield[(3, 1)] = 0x62;

        assert_eq!('b', playfield[(3, 1)] as char);
    }

    #[test]
    fn playfield_navigator() {
        let mut navigator = PlayfieldNavigator::new((4, 3));

        assert_eq!(Direction::Right, navigator.dir());
        assert_eq!((0, 0), navigator.pos());

        navigator.step();

        assert_eq!((1, 0), navigator.pos());

        navigator.step();

        assert_eq!((2, 0), navigator.pos());

        navigator.step();

        assert_eq!((3, 0), navigator.pos());

        navigator.step();

        assert_eq!((0, 0), navigator.pos());

        navigator.turn(Direction::Down);

        assert_eq!(Direction::Down, navigator.dir());
        assert_eq!((0, 0), navigator.pos());

        navigator.step();

        assert_eq!((0, 1), navigator.pos());

        navigator.step();

        assert_eq!((0, 2), navigator.pos());

        navigator.step();

        assert_eq!((0, 0), navigator.pos());

        navigator.turn(Direction::Left);

        assert_eq!(Direction::Left, navigator.dir());
        assert_eq!((0, 0), navigator.pos());

        navigator.step();

        assert_eq!((3, 0), navigator.pos());

        navigator.turn(Direction::Up);

        assert_eq!(Direction::Up, navigator.dir());
        assert_eq!((3, 0), navigator.pos());

        navigator.step();

        assert_eq!((3, 2), navigator.pos());
    }
}
