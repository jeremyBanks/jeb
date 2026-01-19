#![recursion_limit = "1024"]
#![allow(
    non_snake_case,
    unused
)]
use {
    inline::*,
    jeb::testing::literate,
};

literate! {
/*
    # Encoding bytes as text

    There are a lot of different ways to encode bytes/binary data as text, with
    different trade-offs and use cases. We're going to describe several options
    and discuss the decisions and trade-offs in their designs. Some properties
    we may look at include:

    - **Context compatibility:** What characters does the encoding use in its
      output? Where can the encoded data be used as-is without further encoding?
    - **Offset stability:** Will a given byte offset in the input data always
      map to the same character offset in the output data, or can this differ
      based on preceding bytes? Do those characters themselves have variable
      byte lengths when encoded as UTF-8?
    - **Overhead:** How many more characters than bytes do we need to encode a
      given input? How many bytes do those characters themselves require when
      encoded in UTF-8? Can this differ depending on the input data? If we ever
      describe this with a single number, then we're assuming randomly uniformly
      distributed input data for simplicity.
    - **Transparency:** How meaningful is the encoded data when viewed as text?
      Are any patterns in the binary data visible in the encoded text? Are any
      values passed through in a way that is meaningful to a reader?
    - **Ordering:** will two encoded values retain the same relative
      lexicographic ordering as the original bytes, will the sort order be
      preserved?

    We'll see that different encodings make different trade-offs in this space,
    particularly around block alignment (how the encoding groups bytes) and
    numeric transparency (whether small integers remain recognizable).

    ## Latin-1 passthrough

    The simplest possible way to encode binary data as text is just to covert
    byte values directly to Unicode code points. This is sometimes referred to
    Latin-1 passthrough, because this is equivalent to the legacy Latin-1
    (ISO-8859-1) text encoding whose characters now make up the first 256
    Unicode code points (the Basic Latin (ASCII) and Latin-1 Supplement blocks).

    https://en.wikipedia.org/wiki/ISO/IEC_8859-1

    This has the great virtue of simplicity; it's a natural, almost canonical
    choice. That makes is a good option when you just need to store bytes as
    text due to data format limitations, and there are no other considerations.
    However, it may not be a good choice if you need to embed or operate on the
    encoded data as text.

    - **Context compatibility:** nonexistent. This encoding includes control
      characters which may not be expected or well-supported (e.g. `\x00` `NUL`,
      `\x07` `BEL`, `\x7F` `DEL`). It also includes essentially every character
      with special meaning in other encodings or syntaxes (e.g. `\` backslash,
      `"` double quote). These encoded values will practically always require an
      additional layer of escaping or framing to be used in text contexts.
    - **Offset stability:** stable as characters, unstable as UTF-8. Each input
      byte maps exactly to one output character, but half of those characters
      require two bytes when encoded as UTF-8.
    - **Overhead::** 0 as characters, +50% as UTF-8. The consistent 1 byte:1
      character ratio is good, but half of those characters requiring two bytes
      when encoded as UTF-8 is not good.
    - **Text transparency:** this provides provides great transparency in theory
      because all ASCII characters in the input data are passed through
      unchanged. However, the broad context compatibility issues limit the cases
      where this can actually be taken advantage of.
    - **Ordering:** preserved.

    ## Binary

    Binary encoding—where each byte becomes eight `0` or `1` characters—isn't
    practical for production, but it's useful as a pedagogical tool and for
    human input/output in contexts like this document.
*/
/*
    Because we're speaking in terms of byte-oriented encoding (not
    bit-oriented), we need to specify whether the most-significant-bits/
    higher-order-bits (`128` and down) come first (called "big-endian") or the
    least-significant-bits/lower-order-bits (`1` and up) come first (called
    "little-endian"). We follow the standard choice: big-endian, for consistency
    with the way normal decimal numbers are written in code and math.
*/
    fn to_binary(bytes: impl AsRef<[u8]>) -> String {
        let alphabet = b"01";
        let bytes = bytes.as_ref();
        let len = bytes.len() * 8;
        let mut result = String::with_capacity(len);
        for byte in bytes {
            for bit in 0..8 {
                result.push(alphabet[((*byte as usize) >> (7 - bit)) & 0x1] as char);
            }
        }
        assert!(bytes == from_binary(&result));
        result
    }

    to_binary([  0_u8]).is("00000000");
    to_binary([  1_u8]).is("00000001");
    to_binary([  2_u8]).is("00000010");
    to_binary([  3_u8]).is("00000011");
    to_binary([128_u8]).is("10000000");
    to_binary([255_u8]).is("11111111");
    255_u16.to_be_bytes().is([0_u8, 255_u8]);
    256_u16.to_be_bytes().is([1_u8,   0_u8]);
    to_binary(255_u16.to_be_bytes()).is("0000000011111111");
    to_binary(256_u16.to_be_bytes()).is("0000000100000000");
    to_binary(256_u32.to_be_bytes()).is("00000000000000000000000100000000");

/*
    ## Hexadecimal

    One of the most common ways to encode binary data as text is hexadecimal
    ("hex", base 16). Each byte (8 bits, a value from 0 to 255) is represented
    by two hexadecimal digits (4 bits, each representing a value from 0 to 15),
    extending the decimal digits (`0` to `9`) with the first six letters of the
    alphabet (`A` to `F`).

    https://datatracker.ietf.org/doc/html/rfc4648#section-8

    This is a common choice for binary values that may be directly manually
    edited by humans.

    Cleanly splitting each byte in half keeps this encoding quite simple, with
    only one significant design question: which half comes first in the text
    output, the high 4 bits (representing 16, 32, 64, and 128) or the low 4
    bits (representing 1, 2, 4, and 8)? This property is a form of endianness,
    and for hex the answer is always "big endian" (the high 4 bits come first),
    aligning with how decimal numbers are typically written.

    Encoding is quite simple: just pull out the bits, and use them to index into
    the alphabet.
*/
    fn to_hex(bytes: impl AsRef<[u8]>) -> String {
        let alphabet = b"0123456789ABCDEF";
        let bytes = bytes.as_ref();
        let mut result = String::new();
        for byte in bytes {
            let high = byte >> 4; // == byte / 16
            let low = byte & 0xF; // == byte % 16
            result.push(alphabet[high as usize] as char);
            result.push(alphabet[low as usize] as char);
        }
        assert!(bytes == from_hex(&result));
        result
    }

    to_hex(Vec::<u8>::from_iter(0x00..=0x20)).is(
        "000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F20");

    to_hex(from_binary("00000000"        )).is("00"  );
    to_hex(from_binary("00000001"        )).is("01"  );
    to_hex(from_binary("10000000"        )).is("80"  );
    to_hex(from_binary("11111111"        )).is("FF"  );
    to_hex(from_binary("0000000000000000")).is("0000");
    to_hex(from_binary("0000000000000001")).is("0001");
    to_hex(from_binary("1000000000000000")).is("8000");
    to_hex(from_binary("1111111111111111")).is("FFFF");
/*
    - **Context compatibility:** as good as it gets. It only uses digits and a
      handful of letters, and typically not case-sensitive.
    - **Offset stability:** fully stable.
    - **Overhead:** +100%, as bad as it gets. You wouldn't pick hex for its
      efficiency.
    - **Transparency:** pretty good for numeric/binary data. Zeros are `00` and
      it's not that difficult to interpret positive integers. Text is of course
      unrecognizable.
    - **Ordering:** preserved. The hex alphabet is in ascending order (`0-9`,
      then `A-F`), so lexicographic comparison of hex strings matches numeric
      comparison of the underlying bytes.

    ## Beyond byte boundaries

    Hexadecimal has a special property: 4 bits per character divides evenly into
    8 bits per byte. This means each byte maps to exactly two characters, with
    no leftover bits and no need to group multiple bytes together.

    Other bases don't divide so cleanly. Base 64 uses 6 bits per character, and
    base 85 uses log₂(85) ≈ 6.4 bits per character. Since these don't divide 8
    evenly, these encodings must work on multi-byte blocks. Base 64 works on
    3-byte (24-bit) blocks, and base 85 works on 4-byte (32-bit) blocks.

    This raises a question: what happens when the input length isn't a multiple
    of the block size? Different encodings handle this differently, and we'll
    see the details as we go.
*/

/*
    ## Base 64

    One of the most common choices for encoding binary data as text in
    production use cases where human-readability is not a priority is a base 64
    encoding. We're specifically considering base64url, the standard version
    which uses a URL-safe alphabet and no padding.

    https://datatracker.ietf.org/doc/html/rfc4648#section-5

    Base 64 uses 6 bits per character (2⁶ = 64). Since 6 doesn't divide 8
    evenly, we work on 3-byte (24-bit) blocks, which produce exactly 4
    characters (4 × 6 = 24 bits).

    The encoding algorithm is fundamentally the same as hex: repeated division
    and modulo to extract digits. For hex, we use `/ 16` and `% 16`. For base
    64, we'd use `/ 64` and `% 64`. But here's the key insight: because 64 is a
    power of 2 (2⁶), these operations are equivalent to bit shifts and masks:
    `>> 6` is `/ 64`, and `& 0x3F` is `% 64`. This is the same optimization hex
    uses, just with different constants.

    The encoding processes 3 bytes at a time, treating them as a 24-bit big-
    endian integer, then extracting four 6-bit values from high to low:
*/
    fn to_base64(bytes: impl AsRef<[u8]>) -> String {
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let bytes = bytes.as_ref();
        let mut result = String::new();

        // Process complete 3-byte blocks
        let mut i = 0;
        while i + 3 <= bytes.len() {
            let b0 = bytes[i] as u32;
            let b1 = bytes[i + 1] as u32;
            let b2 = bytes[i + 2] as u32;
            let block = (b0 << 16) | (b1 << 8) | b2; // 24-bit big-endian

            result.push(alphabet[(block >> 18) as usize & 0x3F] as char);
            result.push(alphabet[(block >> 12) as usize & 0x3F] as char);
            result.push(alphabet[(block >>  6) as usize & 0x3F] as char);
            result.push(alphabet[(block      ) as usize & 0x3F] as char);
            i += 3;
        }

        // Handle partial blocks (1 or 2 remaining bytes)
        let remaining = bytes.len() - i;
        if remaining == 1 {
            let b0 = bytes[i] as u32;
            // Pad with zeros on the right, extract 2 characters
            result.push(alphabet[(b0 >> 2) as usize] as char);
            result.push(alphabet[((b0 << 4) & 0x3F) as usize] as char);
        } else if remaining == 2 {
            let b0 = bytes[i] as u32;
            let b1 = bytes[i + 1] as u32;
            let block = (b0 << 8) | b1; // 16 bits
            // Pad with zeros on the right, extract 3 characters
            result.push(alphabet[(block >> 10) as usize & 0x3F] as char);
            result.push(alphabet[(block >>  4) as usize & 0x3F] as char);
            result.push(alphabet[((block << 2) & 0x3F) as usize] as char);
        }

        // assert!(bytes == from_base64(&result));

        result
    }

    // Full 3-byte blocks
    to_base64([0x00, 0x00, 0x00]).is("AAAA");
    to_base64([0xFF, 0xFF, 0xFF]).is("____");

    // Visualize the bit grouping: 3 bytes = 24 bits = 4 × 6-bit values
    to_base64(from_binary("000000 000000 000000 000000".replace(" ", ""))).is("AAAA");
    to_base64(from_binary("111111 111111 111111 111111".replace(" ", ""))).is("____");
    to_base64(from_binary("000001 000010 000011 000100".replace(" ", ""))).is("BCDE");

    // Partial blocks
    to_base64([0x00]).is("AA");           // 1 byte → 2 chars
    to_base64([0x00, 0x00]).is("AAA");    // 2 bytes → 3 chars
    to_base64([0xFF]).is("_w");           // 1 byte → 2 chars
    to_base64([0xFF, 0xFF]).is("__8");    // 2 bytes → 3 chars

/*
    Notice that 24 bits (3 bytes) doesn't align nicely with common data
    structures. A 32-bit integer spans 1⅓ blocks; a 64-bit integer spans 2⅔
    blocks. This makes base64 awkward for inspecting structured binary data.

    - **Context compatibility:** very good with the base64url alphabet. It
      doesn't require escaping in standard string contexts. The only minor
      conflict is that `-` is sometimes used as a delimiter.
    - **Offset stability:** fully stable.
    - **Overhead:** +33% (4 characters per 3 bytes).
    - **Transparency:** poor. The alphabet doesn't start with digits, so even
      small integers are unrecognizable. The value 0 encodes as `A`, not `0`.
    - **Ordering:** not preserved. I'm not sure of the reason for the chosen
      alphabet order.

    ## Z85

    Base 64 encodes 3 bytes (24 bits) into 4 characters. A less-common
    alternative are base-85 encodings, which are more efficient but more
    complicated. 85^5 = 4,437,053,125, which is greater than 2^32 =
    4,294,967,296, but _not equal_ to it; 85 is not a power of two. We're
    specifically considering a variation of Z85, a base-85 encoding designed for
    ZeroMQ. It works on 4-byte (32-bit) blocks, producing 5 characters per
    block.

    https://rfc.zeromq.org/spec/32/
 */
    static Z85: &[u8; 85] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

    85_u64.pow(5).is(4_437_053_125_u64);
    2_u64.pow(32).is(4_294_967_296_u64);
    (85_u64.pow(5) > 2_u64.pow(32)).is(true);

/*
    Because 85 is NOT a power of 2, we can't use bit shifts. We have to do
    actual division. The algorithm is the same conceptually—repeated divide and
    modulo to extract digits—but without the bitwise fast path.

    Z85 uses starts its alphabet with `0-9`, which means small integers look
    like decimal numbers. The value 6 encodes to `00006`, not some
    unrecognizable letter.

    The encoding treats each 4-byte block as a big-endian 32-bit integer, then
    extracts 5 base-85 digits from low to high (we reverse at the end to get
    big-endian output):
*/
    fn to_z85(bytes: impl AsRef<[u8]>) -> String {
        let bytes = bytes.as_ref();
        let mut result = String::new();

        // Process complete 4-byte blocks
        let mut i = 0;
        while i + 4 <= bytes.len() {
            let block = u32::from_be_bytes([bytes[i], bytes[i+1], bytes[i+2], bytes[i+3]]);
            let mut value = block as u64;

            // Extract 5 base-85 digits (low to high)
            let mut chars = [0u8; 5];
            for j in (0..5).rev() {
                chars[j] = Z85[(value % 85) as usize];
                value /= 85;
            }
            for c in chars {
                result.push(c as char);
            }
            i += 4;
        }

        // Handle partial blocks (1, 2, or 3 remaining bytes)
        let remaining = bytes.len() - i;
        if remaining > 0 {
            // Pad with zeros to make a full block, encode, then truncate output
            let mut padded = [0u8; 4];
            for j in 0..remaining {
                padded[j] = bytes[i + j];
            }
            let block = u32::from_be_bytes(padded);
            let mut value = block as u64;

            let mut chars = [0u8; 5];
            for j in (0..5).rev() {
                chars[j] = Z85[(value % 85) as usize];
                value /= 85;
            }
            // Output chars proportional to input bytes: 1 byte → 2 chars, etc.
            let out_chars = remaining + 1;
            for c in &chars[..out_chars] {
                result.push(*c as char);
            }
        }

        // assert!(bytes == from_z85(&result));
        result
    }

    // Small integers are recognizable!
    to_z85(0_u32.to_be_bytes()).is("00000");
    to_z85(1_u32.to_be_bytes()).is("00001");
    to_z85(6_u32.to_be_bytes()).is("00006");
    to_z85(9_u32.to_be_bytes()).is("00009");
    to_z85(10_u32.to_be_bytes()).is("0000a");  // 10 is 'a' in base 85
    to_z85(84_u32.to_be_bytes()).is("0000#");  // 84 is '#', the last character
    to_z85(85_u32.to_be_bytes()).is("00010");  // 85 rolls over to "10"

    // Maximum value
    to_z85(0xFFFFFFFF_u32.to_be_bytes()).is("%nSc0");

    // Visualize with binary: a 32-bit block
    to_z85(from_binary("00000000 00000000 00000000 00000000".replace(" ", ""))).is("00000");
    to_z85(from_binary("00000000 00000000 00000000 00000110".replace(" ", ""))).is("00006");

/*
    Partial blocks need exactly `n+1` characters for `n` bytes. This works because
    sqrt(256) < 85 < 256, which means 85^n < 256^n < 85^(n+1) for any n >= 1: n
    characters can't represent n bytes (85^n is too small), but n+1 characters
    can (85^(n+1) is enough). The same property holds for base 64, and any base
    between 16 and 256.
*/
    to_z85([0x00]).is("00");              // 1 byte → 2 chars
    to_z85([0x00, 0x00]).is("000");       // 2 bytes → 3 chars
    to_z85([0x00, 0x00, 0x00]).is("0000"); // 3 bytes → 4 chars


    to_z85(from_binary("00000000"                        )).is("00"   );
    to_z85(from_binary("00000000000000000000000000000000")).is("00000");
    to_z85(from_binary("00000000000000000000000000000001")).is("00001");
    to_z85(from_binary("11111111111111111111111111111111")).is("%nSc0");

    to_z85(from_binary("00000000000000000000000000000011")).is("00003");
    to_z85(from_binary("00000000000000000000000000000010")).is("00002");
    to_z85(from_binary("11111111111111111111111111111110")).is("%nSb#");

    to_z85(from_binary("00000000000000000000000000000111")).is("00007");
    to_z85(from_binary("00000000000000000000000000000100")).is("00004");
    to_z85(from_binary("11111111111111111111111111111100")).is("%nSb%");

    to_z85(from_binary("00000000000000000000000000001111")).is("0000f");
    to_z85(from_binary("00000000000000000000000000001000")).is("00008");
    to_z85(from_binary("11111111111111111111111111111000")).is("%nSb]");

    to_z85(from_binary("00000000000000000000000000011111")).is("0000v");
    to_z85(from_binary("00000000000000000000000000010000")).is("0000g");
    to_z85(from_binary("11111111111111111111111111110000")).is("%nSb*");

    to_z85(from_binary("00000000000000000000000000111111")).is("0000-");
    to_z85(from_binary("00000000000000000000000000100000")).is("0000w");
    to_z85(from_binary("11111111111111111111111111100000")).is("%nSbS");

    to_z85(from_binary("00000000000000000000000001111111")).is("0001G");
    to_z85(from_binary("00000000000000000000000001000000")).is("0000:");
    to_z85(from_binary("11111111111111111111111111000000")).is("%nSbm");

    to_z85(from_binary("00000000000000000000000011111111")).is("00030");
    to_z85(from_binary("00000000000000000000000010000000")).is("0001H");
    to_z85(from_binary("11111111111111111111111110000000")).is("%nSaH");

    to_z85(from_binary("00000000000000000000000111111111")).is("00061");
    to_z85(from_binary("00000000000000000000000100000000")).is("00031");
    to_z85(from_binary("000000000000000000000001"        )).is("0003" );
    to_z85(from_binary("11111111111111111111111100000000")).is("%nS90");

    to_z85(from_binary("00000000000000000000001111111111")).is("000c3");
    to_z85(from_binary("00000000000000000000001000000000")).is("00062");
    to_z85(from_binary("000000000000000000000010"        )).is("0006" );
    to_z85(from_binary("11111111111111111111111000000000")).is("%nS5#");

    to_z85(from_binary("00000000000000000000011111111111")).is("000o7");
    to_z85(from_binary("00000000000000000000010000000000")).is("000c4");
    to_z85(from_binary("000000000000000000000100"        )).is("000c" );
    to_z85(from_binary("11111111111111111111110000000000")).is("%nR#%");

    to_z85(from_binary("00000000000000000000111111111111")).is("000Mf");
    to_z85(from_binary("00000000000000000000100000000000")).is("000o8");
    to_z85(from_binary("000000000000000000001000"        )).is("000o" );
    to_z85(from_binary("11111111111111111111100000000000")).is("%nR&]");

    to_z85(from_binary("00000000000000000001111111111111")).is("001bv");
    to_z85(from_binary("00000000000000000001000000000000")).is("000Mg");
    to_z85(from_binary("000000000000000000010000"        )).is("000M" );
    to_z85(from_binary("11111111111111111111000000000000")).is("%nRM*");

    to_z85(from_binary("00000000000000000011111111111111")).is("002m-");
    to_z85(from_binary("00000000000000000010000000000000")).is("001bw");
    to_z85(from_binary("000000000000000000100000"        )).is("001b" );
    to_z85(from_binary("11111111111111111110000000000000")).is("%nR0S");

    to_z85(from_binary("00000000000000000111111111111111")).is("004JG");
    to_z85(from_binary("00000000000000000100000000000000")).is("002m:");
    to_z85(from_binary("000000000000000001000000"        )).is("002m" );
    to_z85(from_binary("11111111111111111100000000000000")).is("%nP>m");

    to_z85(from_binary("00000000000000001111111111111111")).is("00960");
    to_z85(from_binary("00000000000000001000000000000000")).is("004JH");
    to_z85(from_binary("000000000000000010000000"        )).is("004J" );
    to_z85(from_binary("11111111111111111000000000000000")).is("%nNPH");

    to_z85(from_binary("00000000000000011111111111111111")).is("00ic1");
    to_z85(from_binary("00000000000000010000000000000000")).is("00961");
    to_z85(from_binary("0000000000000001"                )).is("009"  );
    to_z85(from_binary("11111111111111110000000000000000")).is("%nJ60");

    to_z85(from_binary("00000000000000111111111111111111")).is("00Ao3");
    to_z85(from_binary("00000000000000100000000000000000")).is("00ic2");
    to_z85(from_binary("0000000000000010"                )).is("00i"  );
    to_z85(from_binary("11111111111111100000000000000000")).is("%nz##");

    to_z85(from_binary("00000000000001111111111111111111")).is("00&M7");
    to_z85(from_binary("00000000000001000000000000000000")).is("00Ao4");
    to_z85(from_binary("0000000000000100"                )).is("00A"  );
    to_z85(from_binary("11111111111111000000000000000000")).is("%nh&%");

    to_z85(from_binary("00000000000011111111111111111111")).is("01Ybf");
    to_z85(from_binary("00000000000010000000000000000000")).is("00&M8");
    to_z85(from_binary("0000000000001000"                )).is("00&"  );
    to_z85(from_binary("11111111111110000000000000000000")).is("%m=M]");

    to_z85(from_binary("00000000000111111111111111111111")).is("03zmv");
    to_z85(from_binary("00000000000100000000000000000000")).is("01Ybg");
    to_z85(from_binary("0000000000010000"                )).is("01Y"  );
    to_z85(from_binary("11111111111100000000000000000000")).is("%l{0*");

    to_z85(from_binary("00000000001111111111111111111111")).is("06*I-");
    to_z85(from_binary("00000000001000000000000000000000")).is("03zmw");
    to_z85(from_binary("0000000000100000"                )).is("03z"  );
    to_z85(from_binary("11111111111000000000000000000000")).is("%ki>S");

    to_z85(from_binary("00000000011111111111111111111111")).is("0dU4G");
    to_z85(from_binary("00000000010000000000000000000000")).is("06*I:");
    to_z85(from_binary("0000000001000000"                )).is("06*"  );
    to_z85(from_binary("11111111110000000000000000000000")).is("%g!Qm");

    to_z85(from_binary("00000000111111111111111111111111")).is("0rr90");
    to_z85(from_binary("00000000100000000000000000000000")).is("0dU4H");
    to_z85(from_binary("0000000010000000"                )).is("0dU"  );
    to_z85(from_binary("11111111100000000000000000000000")).is("%9$7H");

    to_z85(from_binary("00000001111111111111111111111111")).is("0SSi1");
    to_z85(from_binary("00000001000000000000000000000000")).is("0rr91");
    to_z85(from_binary("00000001"                        )).is("0r"   );
    to_z85(from_binary("11111111000000000000000000000000")).is("@@r30");

    to_z85(from_binary("00000011111111111111111111111111")).is("1onA3");
    to_z85(from_binary("00000010000000000000000000000000")).is("0SSi2");
    to_z85(from_binary("00000010"                        )).is("0S"   );
    to_z85(from_binary("11111110000000000000000000000000")).is("@R#]#");

    to_z85(from_binary("00000111111111111111111111111111")).is("2MK&7");
    to_z85(from_binary("00000100000000000000000000000000")).is("1onA4");
    to_z85(from_binary("00000100"                        )).is("1o"   );
    to_z85(from_binary("11111100000000000000000000000000")).is("}#uY%");

    to_z85(from_binary("00001111111111111111111111111111")).is("5c8Xf");
    to_z85(from_binary("00001000000000000000000000000000")).is("2MK&8");
    to_z85(from_binary("00001000"                        )).is("2M"   );
    to_z85(from_binary("11111000000000000000000000000000")).is("{Y7o]");

    to_z85(from_binary("00011111111111111111111111111111")).is("aohxv");
    to_z85(from_binary("00010000000000000000000000000000")).is("5c8Xg");
    to_z85(from_binary("00010000"                        )).is("5c"   );
    to_z85(from_binary("11110000000000000000000000000000")).is("[bJB*");

    to_z85(from_binary("00111111111111111111111111111111")).is("kMy=-");
    to_z85(from_binary("00100000000000000000000000000000")).is("aohxw");
    to_z85(from_binary("00100000"                        )).is("ao"   );
    to_z85(from_binary("11100000000000000000000000000000")).is("?#A-S");

    to_z85(from_binary("01111111111111111111111111111111")).is("Fb/MG");
    to_z85(from_binary("01000000000000000000000000000000")).is("kMy=:");
    to_z85(from_binary("01000000"                        )).is("kM"   );
    to_z85(from_binary("11000000000000000000000000000000")).is("ZYjum");

    to_z85(from_binary("11111111111111111111111111111111")).is("%nSc0");
    to_z85(from_binary("10000000"                        )).is("Fb"   );

/*
    Z85's 32-bit blocks align perfectly with common data structures. A u32 is
    exactly one block; a u64 is exactly two blocks. When inspecting binary data,
    you can often see meaningful structure: pointers, sizes, flags.

    - **Context compatibility:** decent: the alphabet was chosen to avoid the
      most common string delimiters, so it won't include single- or
      double-quotes or backslashes `'"\\`, but it does include `$` the dollar
      sign, `&` ampersand, and other characters that can have special meaning in
      text in some languages.
    - **Offset stability:** fully stable.
    - **Overhead:** +25% (5 characters per 4 bytes), better than base64's +33%.
    - **Transparency:** excellent for numeric data. Small integers look like
      integers. Zeros are `00000`. Structure in 32-bit aligned data is visible.
    - **Ordering:** NOT preserved. This is the trade-off for numeric
      transparency. Having `0` encode to `0` (instead of the first character
      in ASCII order) means the alphabet can't be in ascending order (there
      aren't enough suitable ASCII characters after `0`), so lexicographic
      comparison of encoded strings doesn't match byte comparison.

    This is a deliberate design choice: Z85 prioritizes numeric transparency
    over ordering. A different encoding could make the opposite choice.
*/

}

// The decoding implementations are mostly down here, outside of the literate
// block because that's too much code.

fn from_binary(text: impl AsRef<str>) -> Vec<u8> {
    assert!(text.as_ref().len() % 8 == 0);
    let text = text.as_ref();
    let len = text.len() / 8;
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let index = i * 8;
        let byte = u8::from_str_radix(&text[index..index + 8], 2).unwrap();
        result.push(byte);
    }
    result
}

fn from_hex(text: impl AsRef<str>) -> Vec<u8> {
    assert!(text.as_ref().len() % 2 == 0);
    let text = text.as_ref();
    let len = text.len() / 2;
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let index = i * 2;
        let byte = u8::from_str_radix(&text[index..index + 2], 16).unwrap();
        result.push(byte);
    }
    result
}
