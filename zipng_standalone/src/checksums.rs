use crc::Crc;

/// CRC-32 configured as per ISO 3309
pub fn crc32(bytes: &[u8]) -> u32 {
    const CRC_32_ISO_HDLC: Crc<u32> = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
    let mut hasher = CRC_32_ISO_HDLC.digest();
    hasher.update(bytes);
    hasher.finalize()
}

// Adler-32
pub fn adler32(bytes: &[u8]) -> u32 {
    simd_adler32::adler32(&bytes)
}
