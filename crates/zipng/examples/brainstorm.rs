// Sketch of the v2 public API usage.

use std::collections::HashMap;

fn main() {
    let files = HashMap::<&'static [u8], Vec<u8>>::new();

    // `::encode` and `.encode` take a very generic input type of basically anything that can
    // be iterated over as pairs of values that can each be borrowed as `&[u8]`, which may be
    // different types.

    let _encoded_with_defaults: Vec<u8> = zipng::v2::encode(&files);
    let _encoded_with_config: Vec<u8> = zipng::v2::Encoder::new()
        .with_mode(zipng::v2::EncoderMode::Rgba)
        .with_font(zipng::v2::FontChoice::Sixth)
        .encode(&files);

    // Decode is not yet implemented but the API shape exists.
    // let decoded: Vec<(Vec<u8>, Vec<u8>)> = zipng::v2::decode(&encoded_with_defaults);
    // let decoded_with_config = zipng::v2::Decoder::new()
    //     .with_source(zipng::v2::Source::Zip)
    //     .decode(&encoded_with_config);
}
