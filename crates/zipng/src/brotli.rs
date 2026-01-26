use brotli::enc::BrotliEncoderParams;
use bstr::BStr;
use bstr::BString;
use bstr::ByteSlice;
use bstr::Bytes;

use crate::generic::default;

pub fn compress(bytes: &[u8]) -> Vec<u8> {
    let mut bytes = bytes.as_bytes();
    let mut buffer = Vec::<u8>::new();
    brotli::BrotliCompress(&mut bytes, &mut buffer, &default())
        .expect("brotli compression must not fail");
    buffer.into()
}

pub fn decompress(bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = bytes.as_bytes();
    let mut buffer = Vec::<u8>::new();
    brotli::BrotliDecompress(&mut bytes, &mut buffer)?;
    Ok(buffer.into())
}
