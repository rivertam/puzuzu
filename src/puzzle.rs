use crate::header::Header;
use crate::puzzle_buffer::PuzzleBuffer;
use crate::puzzle_type::PuzzleType;
use crate::solution_state::SolutionState;

const ENCODING: &'static str = "ISO-8859-1";
const ACROSSDOWN: &'static str = "ACROSS&DOWN";

/// Represents a puzzle
pub struct Puzzle {
    pub preamble: Vec<u8>,
    pub header: Header,
    pub postscript: String,
    pub title: String,
    pub author: String,
    pub copyright: String,
    pub version: String,
    pub encoding: String,

    pub fill: String,

    pub solution: String,

    pub clues: Vec<String>,
    pub notes: String,
    pub extensions: Vec<String>,
    /// The following is so that we can round-trip values in order
    pub extensions_order: Vec<String>,
    pub puzzle_type: PuzzleType,
    pub solution_state: SolutionState,

    /// Add-ons like Rebus
    pub helpers: std::collections::HashMap<String, String>,
}

impl Puzzle {
    pub fn from_puz(data: Vec<u8>) -> anyhow::Result<Puzzle> {
        let mut buffer = PuzzleBuffer::new(&data, ENCODING.to_owned());

        // advance to start - files may contain some data before the
        // start of the puzzle use the ACROSS&DOWN magic string as a waypoint
        // save the preamble for round-tripping
        buffer.seek_to(ACROSSDOWN, -2)?;
        let preamble = buffer.seen().to_vec();

        let header = buffer.unpack_header()?;

        let puz = Self {
            header,
            preamble,
            postscript: "".to_string(),
            title: "".to_string(),
            author: "".to_string(),
            copyright: "".to_string(),
            version: "1.3".to_string(),
            encoding: ENCODING.to_string(),
            fill: "".to_string(),
            solution: "".to_string(),
            clues: vec![],
            notes: "".to_string(),
            extensions: vec![],
            extensions_order: vec![],
            puzzle_type: PuzzleType::Normal,
            solution_state: SolutionState::Unlocked,
            helpers: std::collections::HashMap::new(),
        };

        Ok(puz)
    }
}

#[cfg(test)]
mod tests {
    use super::Puzzle;
    #[test]
    fn test_clue_numbering() {
        assert_eq!(2 + 2, 4);
        let bytes = std::fs::read("./test_files/washpost.puz").unwrap();
        let puzzle = Puzzle::from_puz(bytes).unwrap();

        assert_eq!(puzzle.header.global_checksum, 2253);
        assert_eq!(puzzle.header.header_checksum, 59906);
        assert_eq!(puzzle.header.magic_checksum, 7331058286821292875);
        assert_eq!(puzzle.header.file_version, "1.2c");
        assert_eq!(puzzle.header.scrambled_checksum, 0);
        assert_eq!(puzzle.header.width, 15);
        assert_eq!(puzzle.header.height, 15);
        assert_eq!(puzzle.header.clue_count, 78);
        assert_eq!(puzzle.header.puzzle_type, 1);
        assert_eq!(puzzle.header.solution_state, 0);

        // let clues = puzzle.clue_numbering();
        // assert_eq!(puzzle.clues.len(), clues.across.len() + clues.down.len());
        // assert!(puzzle.clues.len() > 0);
    }
}
