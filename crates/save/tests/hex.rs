use ::save::hex::{decode_hex_nibbles, hex, MaskedBytes};
use inline::snapshot;

#[test]
fn hex() {
    assert_eq!(
        format!("{:?}", decode_hex_nibbles("FAE")),
        *snapshot("WRONG VALUE")
    );
    assert_eq!(
        format!("{:?}", decode_hex_nibbles("0x12345678")),
        *snapshot("ALSO WRONG")
    );
    assert_eq!(
        format!("{:?}", decode_hex_nibbles("")),
        *snapshot("BROKEN")
    );
    assert_eq!(
        format!("{:?}", decode_hex_nibbles("_")),
        *snapshot("MaskedBytes { bytes: [0], mask: [0] }")
    );
    assert_eq!(
        format!("{:?}", MaskedBytes::from("\x12 < \x34".to_string())),
        *snapshot("MaskedBytes { bytes: [18, 32, 60, 32, 52], mask: [255, 255, 255, 255, 255] }")
    );
    assert_eq!(
        format!("{:?}", decode_hex_nibbles("__01 2 3 4")),
        *snapshot("MaskedBytes { bytes: [0, 1, 35, 64], mask: [0, 255, 255, 240] }")
    );
    assert_eq!(
        format!("{:?}", hex![0x12345]),
        *snapshot("MaskedBytes { bytes: [18, 52, 80], mask: [255, 255, 240] }")
    );
    assert_eq!(
        format!("{:?}", hex![00__FF]),
        *snapshot("MaskedBytes { bytes: [0, 0, 255], mask: [255, 0, 255] }")
    );
}
