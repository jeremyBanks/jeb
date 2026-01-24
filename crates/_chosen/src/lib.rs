//! Internal choices for algorithms and representations that could be changed
//! without breaking external APIs, or breaking any of our internal flows if
//! data is migrated and required properties are maintained.

pub fn bytes_to_text(bytes: &[u8]) -> String {
    // fuzzer augmented hex encoding
    unimplemented!()
}

pub fn text_to_bytes(text: &str) -> Vec<u8> {
    // fuzzer augmented hex decoding
    unimplemented!()
}

pub fn digest_bytes(bytes: &[u8]) -> Vec<u8> {
    // blake3 with default length
    unimplemented!()
}
