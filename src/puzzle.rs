use crate::extension::Extension;
use crate::header::Header;
use crate::puzzle_buffer::PuzzleBuffer;
use anyhow::{Context, Error, Result};
use encoding::all::{ISO_8859_1, UTF_8};
use encoding::{DecoderTrap, Encoding};
use std::collections::HashMap;

const ACROSSDOWN: &'static str = "ACROSS&DOWN";

/// Represents a puzzle
pub struct Puzzle {
    pub preamble: Vec<u8>,
    pub header: Header,
    pub postscript: Vec<u8>,
    pub title: String,
    pub author: String,
    pub copyright: String,

    pub fill: String,

    pub solution: String,

    pub clues: Vec<String>,
    pub notes: String,
    pub extensions: Vec<Extension>,

    /// Add-ons like Rebus
    pub helpers: HashMap<String, String>,
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

        // once we have the file version, we can guess the encoding
        if header.version_tuple()?.0 < 2 {
            buffer.set_decoder(|bytes| {
                ISO_8859_1
                    .decode(bytes, DecoderTrap::Strict)
                    .map_err(|_err| {
                        Error::msg(
                            "puz file has version < 3, but decoding using ISO8859-1 was unsuccessful",
                        )
                    })
            });
        } else {
            buffer.set_decoder(|bytes| {
                UTF_8.decode(bytes, DecoderTrap::Strict).map_err(|_err| {
                    Error::msg("puz file is version > 2 but decoding using UTF-8 was unsuccessful")
                })
            });
        };

        let solution = buffer.unpack_solution(header.width, header.height)?;

        let fill = buffer.unpack_fill(header.width, header.height)?;

        let title = buffer.unpack_string().context("Failed to parse title")?;
        let author = buffer.unpack_string().context("Failed to parse author")?;
        let copyright = buffer
            .unpack_string()
            .context("Failed to parse copyright")?;

        let clues =
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
            clues,
            notes,
            extensions,
            helpers: HashMap::new(),
        };

        Ok(puz)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Puzzle, PuzzleType, SolutionState};
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
    fn test_clues() {
        let bytes = std::fs::read("./test_files/washpost.puz").unwrap();
        let puzzle = Puzzle::from_puz(bytes).unwrap();

        assert_eq!(puzzle.clues, Vec::<String>::new());
    }
}
