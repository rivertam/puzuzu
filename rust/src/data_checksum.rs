/// helper functions for cksums and scrambling
pub(crate) fn data_checksum(data: &[u8], initial_checksum: u16) -> u16 {
    data.iter()
        .fold(initial_checksum, |mut checksum, &byte| -> u16 {
            // right-shift one with wrap-around
            let low_bit = checksum & 0x0001;
            checksum = checksum >> 1;
            if low_bit > 0 {
                checksum = checksum | 0x8000;
            }

            // then add in the data and clear any carried bit past 16
            ((checksum as u32 + byte as u32) & 0xffff) as u16
        })
}
