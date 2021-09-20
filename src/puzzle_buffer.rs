use crate::header::Header;
use anyhow::Result;
use thiserror::Error;

const HEADER_FORMAT: &'static str = "<
 H 11s        xH
 Q       4s  2sH
 12s         BBH
 H H ";

/// Wraps a data buffer
pub struct PuzzleBuffer<'a> {
    data: &'a [u8],
    encoding: String,
    position: usize,
}

#[derive(Error, Debug)]
pub enum PuzzleBufferError {
    #[error("Cannot find '{0}' in data")]
    SeekError(String),
}

impl<'a> PuzzleBuffer<'a> {
    pub fn new(data: &'a [u8], encoding: String) -> PuzzleBuffer<'a> {
        PuzzleBuffer {
            data,
            encoding,
            position: 0,
        }
    }

    /// returns bytes that have been seen already
    pub fn seen(&self) -> &'a [u8] {
        &self.data[..self.position]
    }

    /// returns bytes that will be seen later
    pub fn upcoming(&self) -> &'a [u8] {
        &self.data[self.position..]
    }

    pub fn seek_to(&mut self, substring: &str, offset: i32) -> Result<()> {
        // Finds the index of a "substring" within the buffer.
        // This is a naiive port of python's list index function.
        // Presumably there's a better way to do it.

        for (index, window) in self.data[self.position..]
            .windows(substring.len())
            .enumerate()
        {
            if window == substring.as_bytes() {
                self.position = (index as i32 + offset) as usize;
                return Ok(());
            }
        }

        Err(PuzzleBufferError::SeekError(substring.to_string()))?
    }

    pub fn unpack_header(&self) -> Result<Header> {
        Header::from_bytes(self.upcoming())
    }
}

#[cfg(test)]
mod tests {
    use super::PuzzleBuffer;

    #[test]
    fn test_seek_to() {
        let data = "Hello there".as_bytes();
        let mut buffer = PuzzleBuffer::new(&data, "UTF-8".to_string());

        assert!(buffer.seek_to("there", 2).is_ok());
        assert_eq!(buffer.position, "Hello ".len() + 2);

        let err = buffer.seek_to("there", 2);
        println!("{:?}", err);
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().to_string(), "Cannot find 'there' in data");
    }
}
