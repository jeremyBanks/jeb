use {
    crate::generic::default,
    brotli::enc::BrotliEncoderParams,
    bstr::{BStr, BString, ByteSlice, Bytes},
};

pub fn compress(bytes: &[u8]) -> Vec<u8> {
    let mut bytes = bytes.as_bytes();
    let mut buffer = Vec::<u8>::new();
    brotli::BrotliCompress(&mut bytes, &mut buffer, &default())
        .expect("brotli compression must not fail");
    buffer
}

pub fn decompress(bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = bytes.as_bytes();
    let mut buffer = Vec::<u8>::new();
    brotli::BrotliDecompress(&mut bytes, &mut buffer)?;
    Ok(buffer)
}
