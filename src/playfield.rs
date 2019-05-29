use std::fmt;
use std::ops;

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
        let width = lines.iter().map(|s| s.chars().count()).max().unwrap();
        let height = lines.len();

        let mut field = Vec::with_capacity(width * height);

        for l in lines {
            field.extend(format!("{:1$}", l, width).bytes());
        }

        Self {
            field,
            width,
            height,
        }
    }

    /// Return the width of this playfield.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Return the height of this playfield.
    pub fn height(&self) -> usize {
        self.height
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
        for r in 0..self.height {
            let slice = &self.field[(r * self.width)..(r * self.width + self.width)];

            writeln!(f, "{}", std::str::from_utf8(slice).unwrap())?;
        }

        Ok(())
    }
}

/// The four movement directions
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
    pub(crate) field: Playfield,
    pos: (usize, usize),
    dir: Direction,
}

impl PlayfieldNavigator {
    /// Create a new navigator for the given playfield.
    ///
    /// Initially, we are looking right from position `(0, 0)`.
    pub fn new(field: Playfield) -> Self {
        Self {
            field,
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
                    self.pos.1 = self.field.height() - 1
                }
            }
            Direction::Down => {
                if self.pos.1 < self.field.height() - 1 {
                    self.pos.1 += 1
                } else {
                    self.pos.1 = 0
                }
            }
            Direction::Left => {
                if self.pos.0 > 0 {
                    self.pos.0 -= 1
                } else {
                    self.pos.0 = self.field.width() - 1
                }
            }
            Direction::Right => {
                if self.pos.0 < self.field.width() - 1 {
                    self.pos.0 += 1
                } else {
                    self.pos.0 = 0
                }
            }
        }
    }

    /// Return the value of the current field.
    pub fn get(&self) -> u8 {
        self.field[self.pos]
    }

    /// Turn into the given direction.
    pub fn turn(&mut self, dir: Direction) {
        self.dir = dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playfield() {
        let mut playfield = Playfield::new("abc\nde\nx yz\n");

        assert_eq!(4, playfield.width());
        assert_eq!(3, playfield.height());

        assert_eq!("abc \nde  \nx yz\n", playfield.to_string());

        assert_eq!('a', playfield[(0, 0)] as char);
        assert_eq!(' ', playfield[(3, 1)] as char);

        playfield[(3, 1)] = 0x62;

        assert_eq!('b', playfield[(3, 1)] as char);
    }

    #[test]
    fn playfield_navigator() {
        let playfield = Playfield::new("abc\nde\nx yz\n");
        let mut navigator = PlayfieldNavigator::new(playfield);

        assert_eq!('a', navigator.get() as char);

        navigator.step();

        assert_eq!('b', navigator.get() as char);

        navigator.step();

        assert_eq!('c', navigator.get() as char);

        navigator.step();

        assert_eq!(' ', navigator.get() as char);

        navigator.step();

        assert_eq!('a', navigator.get() as char);

        navigator.turn(Direction::Down);

        assert_eq!('a', navigator.get() as char);

        navigator.step();

        assert_eq!('d', navigator.get() as char);

        navigator.step();

        assert_eq!('x', navigator.get() as char);

        navigator.step();

        assert_eq!('a', navigator.get() as char);

        navigator.turn(Direction::Left);

        assert_eq!('a', navigator.get() as char);

        navigator.step();

        assert_eq!(' ', navigator.get() as char);

        navigator.turn(Direction::Up);

        assert_eq!(' ', navigator.get() as char);

        navigator.step();

        assert_eq!('z', navigator.get() as char);
    }

    #[test]
    fn playfield_navigator_modify() {
        let playfield = Playfield::new("abc\nde\nx yz\n");
        let mut navigator = PlayfieldNavigator::new(playfield);

        assert_eq!('a', navigator.get() as char);

        navigator.field[(0, 0)] = 0x62;

        assert_eq!('b', navigator.get() as char);
    }
}
