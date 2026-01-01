use crate::const_checked::OBS;
/// Digits of hex/base16 encoding as defined by
/// [RFC-3548](https://datatracker.ietf.org/doc/html/rfc3548#section-6).
pub const HEX: &[u8; 16] = b"0123456789ABCDEF";
pub const HEX_LUT: [u8; 256] = OBS(HEX).index_lut();
/// Digits of base32 encoding as defined by
/// [RFC-3548](https://datatracker.ietf.org/doc/html/rfc3548#section-5).
pub const BASE32: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
pub const BASE32_LUT: [u8; 256] = OBS(BASE32).index_lut();
pub const BASE32_PADDING: u8 = b'=';
/// Digits of base64 encoding as defined by
/// [RFC-3548](https://datatracker.ietf.org/doc/html/rfc3548#section-3).
pub const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
pub const BASE64_LUT: [u8; 256] = OBS(BASE64).index_lut();
pub const BASE64_PADDING: u8 = b'=';
/// Digits of base64url encoding as defined by
/// [RFC-3548](https://datatracker.ietf.org/doc/html/rfc3548#section-4).
pub const BASE64URL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
pub const BASE64URL_LUT: [u8; 256] = OBS(BASE64).index_lut();
pub const BASE64URL_PADDING: u8 = b'=';
/// Digits of Z85 encoding as defined by
/// [ZeroMQ RFC 32](https://rfc.zeromq.org/spec/32/).
pub const Z85: &[u8; 85] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
pub const Z85_LUT: [u8; 256] = OBS(Z85).index_lut();
/// Digits of Ascii85 encoding.
pub const ASCII85: &[u8; 85] = b"!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstu";
pub const ASCII85_LUT: [u8; 256] = OBS(ASCII85).index_lut();
pub const ASCII85_ZERO: u8 = b'z';
/// Byte sequences that can potentially occur in UTF-8 data, although not every
/// arrangement of these bytes will constitute value UTF-8 data.
pub const UTF8: &[u8; 243] = &[
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B,
    0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29,
    0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
    0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45,
    0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50, 0x51, 0x52, 0x53,
    0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F, 0x60, 0x61,
    0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
    0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D,
    0x7E, 0x7F, 0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B,
    0x8C, 0x8D, 0x8E, 0x8F, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99,
    0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7,
    0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF, 0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5,
    0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF, 0xC2, 0xC3, 0xC4, 0xC5,
    0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF, 0xD0, 0xD1, 0xD2, 0xD3,
    0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xDB, 0xDC, 0xDD, 0xDE, 0xDF, 0xE0, 0xE1,
    0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED, 0xEE, 0xEF,
    0xF0, 0xF1, 0xF2, 0xF3, 0xF4,
];
pub const UTF8_LUT: [bool; 256] = OBS(UTF8).presence_lut();
/// Bytes that can occur in ASCII data.
pub const ASCII: &[u8; 128] = &[
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B,
    0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29,
    0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
    0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45,
    0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50, 0x51, 0x52, 0x53,
    0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F, 0x60, 0x61,
    0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
    0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D,
    0x7E, 0x7F,
];
pub const ASCII_LUT: [bool; 256] = OBS(ASCII).presence_lut();
/// ASCII control characters except for the common whitespace (`\t`, `\n`,
/// `\r`).
pub const ASCII_NON_WHITESPACE_CONTROL_CHARACTERS: &[u8; 30] = &[
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x0B, 0x0C, 0x0E, 0x0F, 0x10,
    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
    0x1F, 0x7F,
];
/// Bytes that can occur in ASCII text data, excluding control characters other
/// than `\t`, `\n`, and `\r`.
pub const ASCII_TEXT: &[u8; 98] = &OBS(ASCII)
    .sub(OBS(ASCII_NON_WHITESPACE_CONTROL_CHARACTERS))
    .sort()
    .eq(
        b"\t\n\r !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~",
    )
    .to_array();
pub const ASCII_TEXT_LUT: [bool; 256] = OBS(ASCII_TEXT).presence_lut();
/// Bytes that may occur in UTF-8 text data, excluding ASCII control characters
/// other than `\t`, `\n`, and `\r`.
pub const UTF8_TEXT: &[u8; 213] = &OBS(UTF8)
    .sub(OBS(ASCII_NON_WHITESPACE_CONTROL_CHARACTERS))
    .to_array();
pub const UTF8_TEXT_LUT: [bool; 256] = OBS(UTF8_TEXT).presence_lut();
/// Bytes that can occur in ASCII text data, excluding control characters.
pub const ASCII_INLINE_TEXT: &[u8; 95] = &OBS(ASCII_TEXT)
    .sub(OBS(b"\t\n\r"))
    .eq(
        b" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~",
    )
    .to_array();
pub const ASCII_INLINE_TEXT_LUT: [bool; 256] = OBS(ASCII_INLINE_TEXT).presence_lut();
/// ASCII characters that are safe as-is as non-leading characters in
/// identifiers in practically all languages.
pub const SAFE_IN_IDENTIFIER: &[u8; 63] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz";
pub const SAFE_IN_IDENTIFIER_LUT: [bool; 256] = OBS(SAFE_IN_IDENTIFIER).presence_lut();
/// Characters that are safe as-is in in URLs according to the current
/// [RFC 3986](https://datatracker.ietf.org/doc/html/rfc3986).
pub const SAFE_IN_URL: &[u8; 66] = &OBS(SAFE_IN_IDENTIFIER)
    .add(OBS(b"-.~"))
    .sort()
    .eq(b"-.0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz~")
    .to_array();
pub const SAFE_IN_URL_LUT: [bool; 256] = OBS(SAFE_IN_URL).presence_lut();
/// Characters that were safe as-is in in URLs according to the older
/// now-obsolete [RFC 2396](https://datatracker.ietf.org/doc/html/rfc2396).
pub const SAFE_IN_URL_RFC_2396: &[u8; 70] = &OBS(SAFE_IN_URL)
    .add(OBS(b"!'()"))
    .sort()
    .eq(b"!'()-.0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz~")
    .to_array();
pub const SAFE_IN_URL_RFC_2396_LUT: [bool; 256] = OBS(SAFE_IN_URL_RFC_2396)
    .presence_lut();
/// Monospace ASCII characters that are safe as-is in all types of strings in
/// JavaScript, JSON, and languages with similar syntax.
pub const SAFE_IN_STRING: &[u8; 91] = &OBS(ASCII_INLINE_TEXT)
    .sub(OBS(b"\"$'\\"))
    .eq(
        b" !#%&()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~",
    )
    .to_array();
/// Monospace ASCII characters that are safe as-is in double-quoted strings in
/// JavaScript, JSON, and languages with similar syntax.
pub const SAFE_IN_STRING_JSON: &[u8; 93] = &OBS(SAFE_IN_STRING)
    .add(OBS(b"$'"))
    .sort()
    .eq(
        b" !#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~",
    )
    .to_array();
pub const SAFE_IN_STRING_JSON_LUT: [bool; 256] = OBS(SAFE_IN_STRING_JSON).presence_lut();
/// Monospace ASCII characters that are safe as-is in single-quoted strings in
/// JavaScript and languages with similar syntax.
pub const SAFE_IN_STRING_SINGLE_QUOTED: &[u8; 93] = &OBS(ASCII_INLINE_TEXT)
    .sub(OBS(b"'\\"))
    .eq(
        b" !\"#$%&()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~",
    )
    .to_array();
pub const SAFE_IN_STRING_SINGLE_QUOTED_LUT: [bool; 256] = OBS(
        SAFE_IN_STRING_SINGLE_QUOTED,
    )
    .presence_lut();
/// Monospace ASCII characters that are safe as-is in backtick-quoted strings in
/// JavaScript and languages with similar syntax.
///
/// This excludes `$` unconditionally, but a smarter encoding would only require
/// it excluded when it occurs before a raw `{`.
pub const SAFE_IN_STRING_BACKTICKED: &[u8; 92] = &OBS(ASCII_INLINE_TEXT)
    .sub(OBS(b"$`\\"))
    .eq(
        b" !\"#%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_abcdefghijklmnopqrstuvwxyz{|}~",
    )
    .to_array();
pub const SAFE_IN_STRING_BACKTICKED_LUT: [bool; 256] = OBS(SAFE_IN_STRING_BACKTICKED)
    .presence_lut();
