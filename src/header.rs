use anyhow::{Error, Result};

#[derive(Debug)]
pub struct Header {
    pub global_checksum: u16,
    pub header_checksum: u16,
    pub magic_checksum: u64,
    pub file_version: String,
    pub scrambled_checksum: u16,

    pub width: u8,
    pub height: u8,
    pub clue_count: u16,
    pub puzzle_type: u16,
    pub solution_state: u16,
}

impl Header {
    pub fn from_bytes(bytes: &[u8]) -> Result<Header> {
        use byteorder::{LittleEndian, ReadBytesExt};
        use std::io::Cursor;
        use std::io::Read;

        let mut reader = Cursor::new(bytes);
        // H
        let global_checksum = reader
            .read_u16::<LittleEndian>()
            .map_err(|_e| Error::msg("Failed to parse global checksum"))?;
        // 11s
        let mut across_down = [0u8; 11];
        reader.read_exact(&mut across_down);
        // x
        let _ = reader
            .read_u8()
            .map_err(|_e| Error::msg("Failed to receive pad byte"))?;

        // H
        let header_checksum = reader.read_u16::<LittleEndian>()?;
        // Q
        let magic_checksum = reader.read_u64::<LittleEndian>()?;

        // 4s
        let mut file_version = [0u8; 4];
        reader.read_exact(&mut file_version)?;
        let file_version = std::str::from_utf8(&file_version)
            .map_err(|_e| Error::msg("Failed to parse file version"))?
            .to_string();

        // 2s unknown 1
        let mut unknown = [0u8; 2];
        reader.read_exact(&mut unknown)?;

        // H
        let scrambled_checksum = reader.read_u16::<LittleEndian>()?;

        // 12s unknown 2
        let mut unknown = [0u8; 12];
        reader.read_exact(&mut unknown)?;

        // B
        let width = reader.read_u8()?;
        // B
        let height = reader.read_u8()?;

        // H
        let clue_count = reader.read_u16::<LittleEndian>()?;

        // H
        let puzzle_type = reader.read_u16::<LittleEndian>()?;

        // H
        let solution_state = reader.read_u16::<LittleEndian>()?;

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
}
