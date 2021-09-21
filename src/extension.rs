use anyhow::{Context, Error, Result};
use std::collections::HashMap;

struct ExtensionHeader {
    pub code: [u8; 4],
    pub length: u16,
    pub checksum: u16,
}

impl ExtensionHeader {
    fn parse_from_cursor<T: AsRef<[u8]>>(
        reader: &mut std::io::Cursor<T>,
    ) -> Result<ExtensionHeader> {
        use byteorder::{LittleEndian, ReadBytesExt};
        use std::io::Read;

        // 4s
        let mut code = [0u8; 4];
        reader
            .read_exact(&mut code)
            .map_err(|_e| Error::msg("Failed to parse extension code"))?;

        // H
        let length = reader
            .read_u16::<LittleEndian>()
            .map_err(|_e| Error::msg("Failed to parse extension length"))?;

        // H
        let checksum = reader
            .read_u16::<LittleEndian>()
            .map_err(|_e| Error::msg("Failed to parse extension length"))?;

        Ok(ExtensionHeader {
            code,
            length,
            checksum,
        })
    }
}

#[derive(Debug)]
pub struct Extension {
    pub code: [u8; 4],
    pub bytes: Vec<u8>,
}

impl Extension {
    pub fn parse_extensions_from_cursor<T: AsRef<[u8]>>(
        reader: &mut std::io::Cursor<T>,
    ) -> Result<Vec<Extension>> {
        use byteorder::ReadBytesExt;
        use std::io::Read;
        let mut extension_checksums = HashMap::new();
        let mut extensions = vec![];

        while let Ok(header) = ExtensionHeader::parse_from_cursor(reader) {
            extension_checksums.insert(header.code, header.checksum);

            // extension data is represented as a null-terminated string,
            // but since the data can contain nulls we can't use read_string
            let mut extension_bytes = vec![0u8; header.length as usize];
            reader.read_exact(&mut extension_bytes).context(format!(
                "Failed to read {} bytes from extension",
                header.length
            ))?;

            reader
                .read_u8()
                .context("Failed to see trailing byte after extension")?;

            extensions.push(Extension {
                code: header.code,
                bytes: extension_bytes,
            });
        }

        Ok(extensions)
    }
}
