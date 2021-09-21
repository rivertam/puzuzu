use crate::puzzle_type::PuzzleType;
use crate::solution_state::SolutionState;
use anyhow::{Context, Error, Result};
use encoding::all::{ISO_8859_1, UTF_8};
use encoding::{DecoderTrap, EncoderTrap, Encoding};
use std::convert::TryFrom;

#[derive(Debug)]
pub struct Header {
    pub global_checksum: u16,
    pub header_checksum: u16,
    pub magic_checksum: u64,
    pub file_version: String,
    pub scrambled_checksum: u16,

    pub width: usize,
    pub height: usize,
    pub clue_count: usize,
    pub puzzle_type: PuzzleType,
    pub solution_state: SolutionState,
}

impl Header {
    pub fn from_cursor<T: AsRef<[u8]>>(reader: &mut std::io::Cursor<T>) -> Result<Header> {
        use byteorder::{LittleEndian, ReadBytesExt};
        use std::io::Read;

        // H
        let global_checksum = reader
            .read_u16::<LittleEndian>()
            .map_err(|_e| Error::msg("Failed to parse global checksum"))?;
        // 11s
        let mut across_down = [0u8; 11];
        reader
            .read_exact(&mut across_down)
            .map_err(|_e| Error::msg("Failed to parse ACROSS&DOWN"))?;
        // x
        let _ = reader
            .read_u8()
            .map_err(|_e| Error::msg("Failed to parse pad byte"))?;

        // H
        let header_checksum = reader
            .read_u16::<LittleEndian>()
            .map_err(|_e| Error::msg("Failed to parse header checksum"))?;

        // Q
        let magic_checksum = reader.read_u64::<LittleEndian>()?;

        // 4s
        let mut file_version = [0u8; 4];
        reader
            .read_exact(&mut file_version)
            .map_err(|_e| Error::msg("Failed to parse file version"))?;

        let file_version = std::str::from_utf8(&file_version)
            .map_err(|_e| Error::msg("Failed to parse file version"))?[..3]
            .to_string();

        // 2s unknown 1
        let mut unknown = [0u8; 2];
        reader
            .read_exact(&mut unknown)
            .map_err(|_e| Error::msg("Failed to parse unknown bytes"))?;

        // H
        let scrambled_checksum = reader
            .read_u16::<LittleEndian>()
            .map_err(|_e| Error::msg("Failed to parse scrambled checksum"))?;

        // 12s unknown 2
        let mut unknown = [0u8; 12];
        reader
            .read_exact(&mut unknown)
            .map_err(|_e| Error::msg("Failed to parse second set of unknown bytes"))?;

        // B
        let width = reader.read_u8().context("Failed to parse width")? as usize;
        // B
        let height = reader.read_u8().context("Failed to parse height")? as usize;

        // H
        let clue_count = reader
            .read_u16::<LittleEndian>()
            .context("Failed to parse clue count")? as usize;

        // H
        let puzzle_type = reader
            .read_u16::<LittleEndian>()
            .context("Failed to parse puzzle type")?;

        let puzzle_type = PuzzleType::try_from(puzzle_type)
            .map_err(|_e| Error::msg(format!("{} is not a known puzzle type", puzzle_type)))?;

        // H
        let solution_state = reader
            .read_u16::<LittleEndian>()
            .context("Failed to parse solution state")?;

        let solution_state = SolutionState::try_from(solution_state).map_err(|_e| {
            Error::msg(format!("{} is not a known solution state", solution_state))
        })?;

        Ok(Header {
            global_checksum,
            header_checksum,
            magic_checksum,
            file_version,
            scrambled_checksum,

            width,
            height,
            clue_count,
            puzzle_type,
            solution_state,
        })
    }

    /// Parse file version as (major, minor) tuple
    pub fn version_tuple(&self) -> Result<(u64, u64)> {
        let split = self.file_version.as_str().split(".").collect::<Vec<_>>();

        if split.len() != 2 {
            return Err(Error::msg(format!(
                "Expected 2 part file version; received {} parts",
                split.len()
            )));
        }

        let major = split[0]
            .parse::<u64>()
            .context(format!("Received non-integer major version: {}", split[0]))?;

        let minor = split[1]
            .parse::<u64>()
            .context(format!("Received non-integer minor version: {}", split[0]))?;

        Ok((major, minor))
    }

    pub fn get_decoder(&self) -> Result<fn(&[u8]) -> Result<String>> {
        if self.version_tuple()?.0 < 2 {
            Ok(|bytes| {
                ISO_8859_1
                    .decode(bytes, DecoderTrap::Strict)
                    .map_err(|_err| {
                        Error::msg(
                            "puz file has version < 3, but decoding using ISO8859-1 was unsuccessful",
                        )
                    })
            })
        } else {
            Ok(|bytes| {
                UTF_8.decode(bytes, DecoderTrap::Strict).map_err(|_err| {
                    Error::msg("puz file is version > 2 but decoding using UTF-8 was unsuccessful")
                })
            })
        }
    }

    pub fn get_encoder(&self) -> Result<fn(&str) -> Result<Vec<u8>>> {
        if self.version_tuple()?.0 < 2 {
            Ok(|string| {
                ISO_8859_1
                    .encode(string, EncoderTrap::Strict)
                    .map_err(|_err| {
                        Error::msg(format!("Encoding {} with ISO8859-1 failed", string))
                    })
            })
        } else {
            Ok(|string| {
                UTF_8
                    .encode(string, EncoderTrap::Strict)
                    .map_err(|_err| Error::msg(format!("Encoding {} with UTF-8 failed", string)))
            })
        }
    }
}
