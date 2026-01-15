use {
    ::save::hex::{decode_hex_nibbles, hex, MaskedBytes},
    inline::InlineSnapExt,
};

#[test]
fn hex() {
    decode_hex_nibbles("FAE").snap_dbg(
        "MaskedBytes {\n    bytes: [\n        250,\n        224,\n    ],\n    mask: [\n        \
         255,\n        240,\n    ],\n}",
    );
    decode_hex_nibbles("0x12345678").snap_dbg(
        "MaskedBytes {\n    bytes: [\n        18,\n        52,\n        86,\n        120,\n    \
         ],\n    mask: [\n        255,\n        255,\n        255,\n        255,\n    ],\n}",
    );
    decode_hex_nibbles("").snap_dbg("MaskedBytes {\n    bytes: [],\n    mask: [],\n}");
    decode_hex_nibbles("_").snap_dbg(
        "MaskedBytes {\n    bytes: [\n        0,\n    ],\n    mask: [\n        0,\n    ],\n}",
    );
    MaskedBytes::from("\x12 < \x34".to_string()).snap_dbg(
        "MaskedBytes {\n    bytes: [\n        18,\n        32,\n        60,\n        32,\n        \
         52,\n    ],\n    mask: [\n        255,\n        255,\n        255,\n        255,\n        \
         255,\n    ],\n}",
    );
    decode_hex_nibbles("__01 2 3 4")
        .snap_dbg("MaskedBytes {\n    bytes: [\n        0,\n        1,\n        35,\n        64,\n    ],\n    mask: [\n        0,\n        255,\n        255,\n        240,\n    ],\n}");
    hex![0x12345].snap_dbg(
        "MaskedBytes {\n    bytes: [\n        18,\n        52,\n        80,\n    ],\n    mask: \
         [\n        255,\n        255,\n        240,\n    ],\n}",
    );
    hex![00__FF]
        .snap_dbg("MaskedBytes {\n    bytes: [\n        0,\n        0,\n        255,\n    ],\n    mask: [\n        255,\n        0,\n        255,\n    ],\n}");
}
