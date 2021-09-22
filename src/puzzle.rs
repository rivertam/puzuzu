use crate::data_checksum::data_checksum;
use crate::extension::Extension;
use crate::header::Header;
use crate::puzzle_buffer::PuzzleBuffer;
use crate::Clues;
use anyhow::{Context, Error, Result};
use std::collections::HashMap;

const ACROSSDOWN: &'static str = "ACROSS&DOWN";

/// Represents a puzzle
pub struct Puzzle {
    pub(crate) preamble: Vec<u8>,
    pub header: Header,
    pub(crate) postscript: Vec<u8>,
    pub title: String,
    pub author: String,
    pub copyright: String,

    pub(crate) fill: String,

    pub(crate) solution: String,

    pub(crate) all_clues: Vec<String>,
    pub(crate) notes: String,
    pub(crate) extensions: Vec<Extension>,
}

impl Puzzle {
    pub fn from_puz(data: Vec<u8>) -> Result<Puzzle> {
        let mut buffer = PuzzleBuffer::new(&data);

        // advance to start - files may contain some data before the
        // start of the puzzle use the ACROSS&DOWN magic string as a waypoint
        // save the preamble for round-tripping
        buffer.seek_to(ACROSSDOWN, -2)?;
        let preamble = buffer.seen().to_vec();

        let header = buffer.unpack_header()?;

        buffer.set_decoder(header.get_decoder()?);

        let solution = buffer.unpack_solution(header.width, header.height)?;

        let fill = buffer.unpack_fill(header.width, header.height)?;

        let title = buffer.unpack_string().context("Failed to parse title")?;
        let author = buffer.unpack_string().context("Failed to parse author")?;
        let copyright = buffer
            .unpack_string()
            .context("Failed to parse copyright")?;

        let all_clues =
            (0..header.clue_count).fold(Ok(vec![]), |previous, index| -> Result<Vec<String>> {
                previous.and_then(|mut clues| {
                    clues.push(
                        buffer
                            .unpack_string()
                            .context(format!("Failed to parse clue #{}", index))?,
                    );

                    Ok(clues)
                })
            })?;

        let notes = buffer.unpack_string().context("Failed to parse notes")?;

        let extensions = buffer
            .unpack_extensions()
            .context("Failed to unpack extensions")?;

        // sometimes there's some extra garbage at
        // the end of the file, usually \r\n
        let postscript = buffer.upcoming().into();

        let puz = Self {
            header,
            preamble,
            postscript,
            title,
            author,
            copyright,
            fill,
            solution,
            all_clues,
            notes,
            extensions,
        };

        let calculated_checksum = puz
            .calculate_global_checksum()
            .context("Failed to calculate global checksum")?;
        if calculated_checksum != puz.header.global_checksum {
            return Err(Error::msg(format!(
                "Calculated global checksum {} does not match header checksum {}",
                calculated_checksum, puz.header.global_checksum
            )));
        }

        let calculated_header_checksum = puz
            .header
            .calculate_checksum()
            .context("Failed to calculate header checksum")?;
        if calculated_header_checksum != puz.header.header_checksum {
            return Err(Error::msg(format!(
                "Calculated header checksum {} does not match header checksum {}",
                calculated_header_checksum, puz.header.header_checksum
            )));
        }

        let calculated_magic_checksum = puz
            .calculate_magic_checksum()
            .context("Failed to calculate magic checksum")?;
        if calculated_magic_checksum != puz.header.magic_checksum {
            return Err(Error::msg(format!(
                "Calculated magic checksum {} does not match header checksum {}",
                calculated_magic_checksum, puz.header.magic_checksum
            )));
        }

        Ok(puz)
    }

    fn calculate_global_checksum(&self) -> Result<u16> {
        let encode = self.header.get_encoder()?;
        let mut checksum = self.header.header_checksum;
        checksum = data_checksum(&encode(&self.solution)?, checksum);
        checksum = data_checksum(&encode(&self.fill)?, checksum);
        self.text_checksum(checksum)
    }

    fn calculate_magic_checksum(&self) -> Result<u64> {
        let encode = self.header.get_encoder()?;
        const MASK_STRING: &'static str = "ICHEATED";
        let magic_checksum =
            [
                self.header.calculate_checksum()?,
                data_checksum(&encode(&self.solution)?, 0),
                data_checksum(&encode(&self.fill)?, 0),
                self.text_checksum(0)?,
            ]
            .iter()
            .enumerate()
            .rev()
            .fold(0, |mut magic_checksum, (index, checksum)| {
                magic_checksum <<= 8;

                magic_checksum |= MASK_STRING.chars().nth(index).expect(
                    "index somehow out of range in mask string when calculating magic checksum",
                ) as u64
                    ^ (*checksum as u64 & 0x00ff);

                magic_checksum |= (MASK_STRING.chars().nth(index + 4).expect(
                    "index somehow out of range in mask string when calculating magic checksum",
                ) as u64
                    ^ (*checksum as u64 >> 8))
                    << 32;

                magic_checksum
            });

        Ok(magic_checksum)
    }

    fn text_checksum(&self, mut checksum: u16) -> Result<u16> {
        let encode = self.header.get_encoder()?;
        let encode_zstring = move |string| {
            encode(string).map(|mut encoded| {
                encoded.push('\0' as u8);
                encoded
            })
        };
        // for the checksum to work these fields must be added in order with
        // null termination, followed by all non-empty clues without null
        // termination, followed by notes (but only for version >= 1.3)
        checksum = data_checksum(&encode_zstring(&self.title)?, checksum);
        checksum = data_checksum(&encode_zstring(&self.author)?, checksum);
        checksum = data_checksum(&encode_zstring(&self.copyright)?, checksum);

        checksum = self
            .all_clues
            .iter()
            .fold(Ok(checksum), |sum: Result<u16>, clue| {
                Ok(data_checksum(&encode(clue)?, sum?))
            })?;

        let (major, minor) = self.header.version_tuple()?;
        // notes included in global checksum starting v1.3 of format
        if major > 1 || major == 1 && minor >= 3 {
            checksum = data_checksum(&encode_zstring(&self.notes)?, checksum)
        }

        Ok(checksum)
    }

    pub fn clues(&self) -> Result<Clues> {
        Clues::for_puzzle(&self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Clue, Puzzle, PuzzleType, SolutionState};
    use anyhow::Result;

    #[test]
    fn test_header_parsing() {
        let bytes = std::fs::read("./test_files/washpost.puz").unwrap();
        let puzzle = Puzzle::from_puz(bytes).unwrap();

        assert_eq!(puzzle.header.global_checksum, 2253);
        assert_eq!(puzzle.header.header_checksum, 59906);
        assert_eq!(puzzle.header.magic_checksum, 7331058286821292875);
        assert_eq!(puzzle.header.file_version, "1.2");
        assert_eq!(puzzle.header.scrambled_checksum, 0);
        assert_eq!(puzzle.header.width, 15);
        assert_eq!(puzzle.header.height, 15);
        assert_eq!(puzzle.header.clue_count, 78);
        assert_eq!(puzzle.header.puzzle_type, PuzzleType::Normal);
        assert_eq!(puzzle.header.solution_state, SolutionState::Unlocked);
    }

    #[test]
    fn test_solution_parsing() {
        let bytes = std::fs::read("./test_files/washpost.puz").unwrap();
        let puzzle = Puzzle::from_puz(bytes).unwrap();

        assert_eq!(puzzle.solution, "");
    }

    #[test]
    fn test_fill_parsing() {
        let bytes = std::fs::read("./test_files/washpost.puz").unwrap();
        let puzzle = Puzzle::from_puz(bytes).unwrap();

        assert_eq!(puzzle.fill, "");
    }

    #[test]
    fn test_clues() -> Result<()> {
        let bytes = std::fs::read("./test_files/washpost.puz").unwrap();
        let puzzle = Puzzle::from_puz(bytes).unwrap();

        let clues = puzzle.clues()?;
        assert_eq!(clues.across, Vec::<Clue>::new());

        assert_eq!(
            clues.across.len() * clues.down.len(),
            puzzle.header.clue_count
        );

        Ok(())
    }
}
