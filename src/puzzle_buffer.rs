use crate::extension::Extension;
use crate::header::Header;
use anyhow::{Context, Result};
use std::io::Cursor;
use thiserror::Error;

/// Wraps a data buffer
pub struct PuzzleBuffer<'a> {
    data: &'a [u8],
    cursor: Cursor<&'a [u8]>,
    decoder: fn(&[u8]) -> Result<String>,
}

#[derive(Error, Debug)]
pub enum PuzzleBufferError {
    #[error("Cannot find '{0}' in data")]
    SeekError(String),
    #[error("Encoding has not been inferred")]
    EncodingNotInferred,
}

impl<'a> PuzzleBuffer<'a> {
    pub fn new(data: &'a [u8]) -> PuzzleBuffer<'a> {
        PuzzleBuffer {
            data,
            cursor: Cursor::new(data),
            decoder: |_bytes| Err(PuzzleBufferError::EncodingNotInferred.into()),
        }
    }

    pub fn set_decoder(&mut self, decoder: fn(&[u8]) -> Result<String>) {
        self.decoder = decoder;
    }

    fn decode_string(&self, bytes: &[u8]) -> Result<String> {
        let decoder = self.decoder;
        decoder(bytes)
    }

    fn position(&self) -> usize {
        self.cursor.position() as usize
    }

    /// returns bytes that have been seen already
    pub fn seen(&self) -> &'a [u8] {
        &self.data[..self.position()]
    }

    /// returns bytes that will be seen later
    pub fn upcoming(&self) -> &'a [u8] {
        &self.data[self.position()..]
    }

    pub fn seek_to(&mut self, substring: &str, offset: i32) -> Result<()> {
        // Finds the index of a "substring" within the buffer in order
        // to set the cursor's position to that substring.
        // This is a naiive port of python's list index function.
        // Presumably there's a better way to do it.

        for (index, window) in self.data[self.position()..]
            .windows(substring.len())
            .enumerate()
        {
            if window == substring.as_bytes() {
                // set the cursor
                self.cursor.set_position((index as i32 + offset) as u64);
                return Ok(());
            }
        }

        Err(PuzzleBufferError::SeekError(substring.to_string()))?
    }

    pub fn unpack_header(&mut self) -> Result<Header> {
        Header::from_cursor(&mut self.cursor)
    }

    pub fn unpack_solution(&mut self, width: usize, height: usize) -> Result<String> {
        use std::io::Read;
        let mut solution = vec![0u8; width * height];
        self.cursor.read_exact(&mut solution)?;

        self.decode_string(&solution)
    }

    pub fn unpack_fill(&mut self, width: usize, height: usize) -> Result<String> {
        use std::io::Read;
        let mut fill = vec![0u8; width * height];
        self.cursor.read_exact(&mut fill)?;

        self.decode_string(&fill)
    }

    pub fn unpack_string(&mut self) -> Result<String> {
        use std::io::BufRead;
        let mut buf = vec![];
        self.cursor
            .read_until('\0' as u8, &mut buf)
            .context(format!("Failed to find null-terminated string"))?;

        buf.pop();

        self.decode_string(&buf)
    }

    pub fn unpack_extensions(&mut self) -> Result<Vec<Extension>> {
        Extension::parse_extensions_from_cursor(&mut self.cursor)
    }
}

#[cfg(test)]
mod tests {
    use super::PuzzleBuffer;

    #[test]
    fn test_seek_to() {
        let data = "Hello there".as_bytes();
        let mut buffer = PuzzleBuffer::new(&data);

        assert!(buffer.seek_to("there", 2).is_ok());
        assert_eq!(buffer.position(), "Hello ".len() + 2);

        let err = buffer.seek_to("there", 2);
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().to_string(), "Cannot find 'there' in data");
    }
}
