use {
    ::save::hex::{decode_hex_nibbles, hex, MaskedBytes},
    inline::InlineSnapExt,
};

#[test]
fn hex() {
    decode_hex_nibbles("FAE")
        .snap_dbg("MaskedBytes { bytes: [250, 224], mask: [255, 240] }");
    decode_hex_nibbles("0x12345678")
        .snap_dbg("MaskedBytes { bytes: [18, 52, 86, 120], mask: [255, 255, 255, 255] }");
    decode_hex_nibbles("")
        .snap_dbg("MaskedBytes { bytes: [], mask: [] }");
    decode_hex_nibbles("_")
        .snap_dbg("MaskedBytes { bytes: [0], mask: [0] }");
    MaskedBytes::from("\x12 < \x34".to_string())
        .snap_dbg("MaskedBytes { bytes: [18, 32, 60, 32, 52], mask: [255, 255, 255, 255, 255] }");
    decode_hex_nibbles("__01 2 3 4")
        .snap_dbg("MaskedBytes { bytes: [0, 1, 35, 64], mask: [0, 255, 255, 240] }");
    hex![0x12345]
        .snap_dbg("MaskedBytes { bytes: [18, 52, 80], mask: [255, 255, 240] }");
    hex![00__FF]
        .snap_dbg("MaskedBytes { bytes: [0, 0, 255], mask: [255, 0, 255] }");
}
