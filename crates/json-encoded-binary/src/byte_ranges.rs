/// ASCII characters/bytes that won't occur in plain text UTF-8 data.
#[rustfmt::skip]
pub const NOT_PLAIN_UTF8_TEXT: &[u8; 44] = &[
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
pub const NOT_PLAIN_UTF8_TEXT_LUT: [bool; 256] = presence_lut(NOT_PLAIN_UTF8_TEXT);
pub const PLAIN_UTF8_TEXT_LUT: [bool; 256] = invert_presence(&NOT_PLAIN_UTF8_TEXT_LUT);



/// Digits of hex/base16 encoding as defined by
/// [RFC-3548](https://datatracker.ietf.org/doc/html/rfc3548#section-6).
pub const HEX: &[u8; 16] = b"0123456789ABCDEF";
pub const HEX_LUT: [u8; 256] = index_lut(HEX, -1 as _);

/// Digits of base32 encoding as defined by
/// [RFC-3548](https://datatracker.ietf.org/doc/html/rfc3548#section-5).
pub const BASE32: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
pub const BASE32_LUT: [u8; 256] = index_lut(BASE32, -1 as _);
pub const BASE32_PADDING: u8 = b'=';

/// Digits of base64 encoding as defined by
/// [RFC-3548](https://datatracker.ietf.org/doc/html/rfc3548#section-3).
pub const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
pub const BASE64_LUT: [u8; 256] = index_lut(BASE64, -1 as _);
pub const BASE64_PADDING: u8 = b'=';

/// Digits of base64url encoding as defined by
/// [RFC-3548](https://datatracker.ietf.org/doc/html/rfc3548#section-4).
pub const BASE64URL: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
pub const BASE64URL_LUT: [u8; 256] = index_lut(BASE64, -1 as _);
pub const BASE64URL_PADDING: u8 = b'=';

/// Digits of Z85 encoding as defined by
/// [ZeroMQ RFC 32](https://rfc.zeromq.org/spec/32/).
pub const Z85: &[u8; 85] =
    b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
pub const Z85_LUT: [u8; 256] = index_lut(Z85, -1 as _);

/// Digits of Ascii85 encoding.
pub const ASCII85: &[u8; 85] =
    b"!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstu";
pub const ASCII85_LUT: [u8; 256] = index_lut(ASCII85, -1 as _);
pub const ASCII85_ZERO: u8 = b'z';



/// Characters that are safe as-is in in URLs according to the current
/// [RFC 3986](https://datatracker.ietf.org/doc/html/rfc3986).
pub const SAFE_IN_URL: &[u8; 66] =
    b"-.0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz~";
pub const SAFE_IN_URL_LUT: [bool; 256] = presence_lut(SAFE_IN_URL);

/// Characters that were safe as-is in in URLs according to the older
/// now-obsolete [RFC 2396](https://datatracker.ietf.org/doc/html/rfc2396).
pub const SAFE_IN_URL_RFC_2396: &[u8; 70] =
    b"!'()-.0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz~";
pub const SAFE_IN_URL_RFC_2396_LUT: [bool; 256] = presence_lut(SAFE_IN_URL_RFC_2396);

/// Characters that are safe as-is in double-quoted strings in JavaScript,
/// JSON, and languages with similar syntax.
pub const SAFE_IN_STRING_JSON: &[u8; 93] =
        b" !#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~";
pub const SAFE_IN_STRING_JSON_LUT: [bool; 256] = presence_lut(SAFE_IN_STRING_JSON);

/// Characters that are safe as-is in single-quoted strings in JavaScript
/// and languages with similar syntax.
pub const SAFE_IN_STRING_SINGLE_QUOTED: &[u8; 93] =
        b" !\"#$%&()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~";
pub const SAFE_IN_STRING_SINGLE_QUOTED_LUT: [bool; 256] =
    presence_lut(SAFE_IN_STRING_SINGLE_QUOTED);

/// Characters that are safe as-is in backtick-quoted strings in
/// JavaScript and languages with similar syntax.
///
/// This excludes `$` unconditionally, but a smarter encoding would only require
/// it excluded when it occurs before a raw `{`.)
pub const SAFE_IN_STRING_BACKTICKED: &[u8; 94] =
        b" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~";
pub const SAFE_IN_STRING_BACKTICKED_LUT: [bool; 256] = presence_lut(SAFE_IN_STRING_BACKTICKED);


const fn index_lut(values: &[u8], absent: u8) -> [u8; 256] {
    if (values.len() >= 0xFF) {
        panic!("index_lut(...) accepts no more than 256 values");
    } // MARK: test

    let mut table = [absent; 256];
    let mut index = 0;
    while index < values.len() {
        let value = values[index] as usize;
        if table[value] != absent {
            panic!("duplicate value in index_lut!(...)");
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
