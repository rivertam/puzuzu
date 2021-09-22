use crate::square::Square;
use crate::Puzzle;

pub struct Grid<'a> {
    fill: &'a str,
    width: usize,
    height: usize,
}

impl<'a> Grid<'a> {
    pub fn for_puzzle(puzzle: &'a Puzzle) -> Grid<'a> {
        Grid {
            fill: &puzzle.fill,
            width: puzzle.header.width,
            height: puzzle.header.height,
        }
    }

    pub fn get_char(&self, index: usize) -> Option<char> {
        self.fill.chars().nth(index)
    }

    pub fn left(&self, index: usize) -> Option<char> {
        if self.col(index) == 0 {
            None
        } else {
            self.fill.chars().nth(index - 1)
        }
    }

    pub fn above(&self, index: usize) -> Option<char> {
        if self.row(index) == 0 {
            None
        } else {
            self.fill.chars().nth(index - self.width)
        }
    }

    pub fn len_across(&self, index: usize) -> usize {
        let col = self.col(index);
        for len in 0..(self.width - col) {
            let character = if let Some(character) = self.get_char(index + len) {
                character
            } else {
                return 0;
            };

            if Square::is_black_square(character) {
                return len + 1;
            }
        }

        self.width - col
    }

    pub fn len_down(&self, index: usize) -> usize {
        let row = self.row(index);
        for len in 0..(self.height - row) {
            let character = if let Some(character) = self.get_char(index + len * self.width) {
                character
            } else {
                return 0;
            };

            if Square::is_black_square(character) {
                return len + 1;
            }
        }

        self.height - row
    }

    pub fn col(&self, index: usize) -> usize {
        index % self.width
    }

    pub fn row(&self, index: usize) -> usize {
        index / self.width
    }
}
