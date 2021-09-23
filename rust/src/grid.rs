use crate::square::Square;
use crate::Puzzle;
use serde::ser::{SerializeSeq, Serializer};
use serde::Serialize;

#[derive(Serialize)]
pub struct Cell {
    black: bool,
}

pub struct Grid<'a> {
    pub fill: &'a str,
    pub width: usize,
    pub height: usize,
}

impl<'a> Serialize for Grid<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut rows = serializer.serialize_seq(Some(self.height))?;
        for row_index in 0..self.height {
            let mut row = vec![];
            for column_index in 0..self.width {
                let cell_index = row_index * self.width + column_index;
                let square = self
                    .get_char(cell_index)
                    .expect("Tried to serialize invalid grid");

                row.push(Cell {
                    black: Square::is_black_square(square),
                });
            }

            rows.serialize_element(&row)?;
        }

        rows.end()
    }
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
