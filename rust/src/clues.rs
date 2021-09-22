use crate::grid::Grid;
use crate::square::Square;
use crate::Puzzle;
use anyhow::{Context, Error, Result};

#[derive(Debug, Eq, PartialEq)]
pub struct Clue {
    pub number: usize,
    pub clue: String,
    pub cell_index: usize,
    pub length: usize,
}

pub struct Clues {
    pub across: Vec<Clue>,
    pub down: Vec<Clue>,
}

impl Clues {
    pub fn for_puzzle<'a>(puzzle: &Puzzle) -> Result<Clues> {
        let grid = Grid::for_puzzle(puzzle);
        let mut across = vec![];
        let mut down = vec![];

        let mut clue_number = 1;

        let mut clue_iter = puzzle.all_clues.iter();

        for (index, character) in puzzle.fill.chars().enumerate() {
            if Square::is_black_square(character) {
                continue;
            }

            let mut square_has_clue = false;

            let square_to_the_left = grid.left(index).unwrap_or(Square::black());
            let is_across = Square::is_black_square(square_to_the_left);
            let len_across = grid.len_across(index);

            if is_across && len_across > 1 {
                across.push(Clue {
                    number: clue_number,
                    clue: clue_iter
                        .next()
                        .ok_or(Error::msg("Ran out of provided clues"))?
                        .to_string(),
                    cell_index: index,
                    length: len_across,
                });

                square_has_clue = true;
            }

            let square_above = grid.above(index).unwrap_or(Square::black());
            let is_down = Square::is_black_square(square_above);
            let len_down = grid.len_down(index);

            if is_down && len_down > 1 {
                down.push(Clue {
                    number: clue_number,
                    clue: clue_iter
                        .next()
                        .ok_or(Error::msg("Ran out of provided clues"))?
                        .to_string(),
                    cell_index: index,
                    length: len_across,
                });

                square_has_clue = true;
            }

            if square_has_clue {
                clue_number += 1;
            }
        }

        Ok(Self { across, down })
    }
}
