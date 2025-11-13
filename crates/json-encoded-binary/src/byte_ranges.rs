/// Characters used by Z85 encoding.
pub(crate) const Z85: &[u8; 85] =
    b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
pub(crate) const Z85_LUT: [u8; 256] = index_lut(Z85, 0xFF);

/// Characters that are safe as-is in in URLs according to the current
/// [RFC 3986](https://datatracker.ietf.org/doc/html/rfc3986).
pub(crate) static SAFE_IN_URL: &[u8; 66] =
    b"-.0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz~";
pub(crate) static SAFE_IN_URL_LUT: [bool; 256] = presence_lut(SAFE_IN_URL);

/// Characters that were safe as-is in in URLs according to the older
/// now-obsolete [RFC 2396](https://datatracker.ietf.org/doc/html/rfc2396).
pub(crate) static SAFE_IN_URL_RFC_2396: &[u8; 70] =
    b"!'()-.0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz~";
pub(crate) static SAFE_IN_URL_RFC_2396_LUT: [bool; 256] = presence_lut(SAFE_IN_URL_RFC_2396);

/// Characters that are safe as-is in double-quoted strings in JavaScript,
/// JSON, and languages with similar syntax.
pub(crate) static SAFE_IN_STRING_JSON: &[u8; 93] =
        b" !#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~";
pub(crate) static SAFE_IN_STRING_JSON_LUT: [bool; 256] = presence_lut(SAFE_IN_STRING_JSON);

/// Characters that are safe as-is in single-quoted strings in JavaScript
/// and languages with similar syntax.
pub(crate) static SAFE_IN_STRING_SINGLE_QUOTED: &[u8; 93] =
        b" !\"#$%&()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~";
pub(crate) static SAFE_IN_STRING_SINGLE_QUOTED_LUT: [bool; 256] =
    presence_lut(SAFE_IN_STRING_SINGLE_QUOTED);

/// Characters that are safe as-is in backtick-quoted strings in
/// JavaScript and languages with similar syntax. (This excludes `$`
/// unconditionally, but a smarter encoding would only require it excluded
/// when it occurs before a raw `{`.)
pub(crate) static SAFE_IN_STRING_BACKTICKED: &[u8; 94] =
        b" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~";
pub(crate) static SAFE_IN_STRING_BACKTICKED_LUT: [bool; 256] =
    presence_lut(SAFE_IN_STRING_BACKTICKED);

/// ASCII characters/bytes that won't occur in plain text UTF-8 data.
#[rustfmt::skip]
pub(crate) static NOT_PLAIN_UTF8_TEXT: &[u8; 44] = &[
    // Control characters other than `\t`, `\n`, and `\r`.
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08,             0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
    0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F,
                                                0x7F,
    // Invalid UTF-8 bytes: only in overlong encodings.
    0xC0, 0xC1,
    // Invalid UTF-8 bytes: only in out-of-bounds values.
                                  0xF5, 0xF6, 0xF7,
    0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF,
];
pub(crate) static NOT_PLAIN_UTF8_TEXT_LUT: [bool; 256] = presence_lut(NOT_PLAIN_UTF8_TEXT);
pub(crate) static PLAIN_UTF8_TEXT_LUT: [bool; 256] = invert_presence(&NOT_PLAIN_UTF8_TEXT_LUT);

const fn index_lut(values: &[u8], absent: u8) -> [u8; 256] {
    if (values.len() >= 0xFF) {
        panic!("index_lut!(...) requires fewer than 256 values");
    }
    let mut table = [absent; 256];
    let mut index = 0;
    while index < values.len() {
        let value = values[index] as usize;
        if table[value] != absent {
            panic!("duplicate value in index_lut!(...)");
        }
        if (value > 0xFF) {
            panic!("out-of-bounds value in index_lut!(...)");
        }
        table[value] = index as u8;
        index += 1;
    }
    table
}

const fn presence_lut(values: &[u8]) -> [bool; 256] {
    let mut table = [false; 256];
    let mut index = 0;
    while index < values.len() {
        let value = values[index] as usize;
        if (table[value]) {
            panic!("duplicate value in presence_lut!(...)");
        }
        table[value] = true;
        index += 1;
    }
    table
}

const fn invert_presence(lut: &[bool; 256]) -> [bool; 256] {
    let mut table = [false; 256];
    let mut index = 0;
    while index < 256 {
        table[index] = !lut[index];
        index += 1;
    }
    table
}
