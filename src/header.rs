use anyhow::{Context, Error, Result};

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
    pub puzzle_type: u16,
    pub solution_state: u16,
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

        // H
        let solution_state = reader
            .read_u16::<LittleEndian>()
            .context("Failed to parse solution state")?;

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
}
