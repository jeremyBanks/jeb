use {
    crate::Offset,
    crc::Crc,
    std::{fmt::Debug, hash::Hasher, io::Read},
};

/// Compute the CRC-32 checksum of a byte slice as per ISO 3309 and ITU-T V.42.
pub fn crc32(bytes: &[u8]) -> u32 {
    const CRC_32_ISO_HDLC: Crc<u32> = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
    let mut hasher = CRC_32_ISO_HDLC.digest();
    hasher.update(bytes);
    hasher.finalize()
}

/// Compute the Adler-32 checksum of a byte slice
pub fn adler32(bytes: &[u8]) -> u32 {
    simd_adler32::adler32(&bytes)
}

#[cfg(feature = "twox-hash")]
/// Computes the XXH3 64-bit hash of a byte slice
pub fn xxh3_64(bytes: &[u8]) -> u64 {
    let mut hasher = twox_hash::Xxh3Hash64::default();
    hasher.write(bytes);
    hasher.finish()
}

#[cfg(feature = "blake3")]
/// Computes the BLAKE3 cryptographic hash digest of a byte slice, as an
/// infinite stream of bytes. Using fewer than 32 bytes of the output will
/// reduce the security level proportionally.
pub fn blake3(bytes: &[u8]) -> impl Clone + Debug + Read + Offset + Send + Sync {
    let mut hasher = blake3::Hasher::new();
    hasher.update(bytes);
    hasher.finalize_xof()
}
