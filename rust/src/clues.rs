use crate::grid::Grid;
use crate::square::Square;
use anyhow::{Context, Error, Result};
use serde::Serialize;

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct Clue {
    #[serde(rename = "clueNumber")]
    pub clue_number: usize,
    pub text: String,
    pub column: usize,
    pub row: usize,
    pub length: usize,
}

#[derive(Serialize)]
pub struct Clues {
    pub across: Vec<Clue>,
    pub down: Vec<Clue>,
}

impl Clues {
    pub fn new<'a, I: std::iter::IntoIterator<Item = &'a String>>(
        grid: Grid,
        clue_iter: I,
    ) -> Result<Clues> {
        let mut clue_iter = clue_iter.into_iter();
        let mut across = vec![];
        let mut down = vec![];

        let mut clue_number = 1;

        for (index, character) in grid.fill.chars().enumerate() {
            if Square::is_black_square(character) {
                continue;
            }

            let mut square_has_clue = false;

            let square_to_the_left = grid.left(index).unwrap_or(Square::black());
            let is_across = Square::is_black_square(square_to_the_left);
            let len_across = grid.len_across(index);

            if is_across && len_across > 1 {
                across.push(Clue {
                    clue_number: clue_number,
                    text: clue_iter
                        .next()
                        .ok_or(Error::msg("Ran out of provided clues"))?
                        .to_string(),
                    row: grid.row(index),
                    column: grid.col(index),
                    length: len_across,
                });

                square_has_clue = true;
            }

            let square_above = grid.above(index).unwrap_or(Square::black());
            let is_down = Square::is_black_square(square_above);
            let len_down = grid.len_down(index);

            if is_down && len_down > 1 {
                down.push(Clue {
                    clue_number: clue_number,
                    text: clue_iter
                        .next()
                        .ok_or(Error::msg("Ran out of provided clues"))?
                        .to_string(),
                    row: grid.row(index),
                    column: grid.col(index),
                    length: len_down,
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
