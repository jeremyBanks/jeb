// XXX: AGENTS, IGNORE THIS FILE! I am a human working on it
// while you're doing other things. If this file causes something to
// break for you, please immediately stop and let me know, do not attempt
// to edit this file yourself, or attempt overcomplicated workarounds.
// It's okay if this file is included when you commit, and it's okay if
// not. Don't worry about it either way.


use alloc::collections::HashMap;

#[cfg(feature = "ignore-me")]
fn main() {
    let files = HashMap::<&'static [u8], Vec<&Vec<u8>>>::new();
    
    // `::encode` and `.encode` takes a very generic input type of basically anything that can
    // be iterate over as pairs of values that can each be borrowed as `&[u8]`, which may be
    // different types.

    let encoded_with_defaults: Vec<u8> = zipng::encode(files);
    let encoded_with_config: Vec<u8> = zipng::Encoder::new()
        .with_mode(zipng::EncoderMode::Rgba)
        .with_font(zipng::Font::SixthSlab)
        .encode(files);

    // Decode returns impl FromIterator<(Vec<u8>, Vec<u8>)> or something like that (still generic, but less generic).
    // Generic defaults are allowed in this case we'd default to be BTreeMap but I don't think they are.
    let decoded: BTreeMap<Vec<u8>, Vec<u8>> = zipng::decode(encoded_with_defaults);
    let decoded_with_config: Vec<u8> = zipng::Decoder::new()
        .with_source(zipng::Source::Zip);
}
