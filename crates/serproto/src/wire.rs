use std::io::Write;

use crate::error::Error;
use crate::error::Result;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireType {
    /// A variable-length integer, with the 7 low bits being data and the high bit indicating continuation.
    ///
    /// ## Protocol Buffers Compatibility Considerations
    ///
    /// Protocol buffers only supports up to 64-bit integers.
    VarInt = 0b_000,
    /// Variable-length data, with a VarInt prefix indicating its length in bytes.
    ///
    /// ## Protocol Buffers Compatibility Considerations
    ///
    /// Protocol buffers only supports variable-lengths that fit in u31 (up to 2GiB),
    /// and some implementations enforce much lower limits.
    VarLength = 0b_010,
    /// 32-bit fixed-length data (typically u32, i32, or f32).
    Fixed32 = 0b_101,
    /// 64-bit fixed-length data (typically u64, i64, or f64).
    Fixed64 = 0b_001,
    /// Obsolete Protocol Buffers 2 "start group" tag (not used or supported).
    #[doc(hidden)]
    _Obsolete3 = 0b_011,
    /// Obsolete Protocol Buffers 2 "end group" tag (not used or supported).
    #[doc(hidden)]
    _Obsolete4 = 0b_100,
    #[doc(hidden)]
    _Undefined6 = 0b_110,
    #[doc(hidden)]
    _Undefined7 = 0b_111,
}

#[inline]
pub fn read_wire_type(tag_byte: u8) -> WireType {
    match 0b_111 & tag_byte {
        0b_000 => WireType::VarInt,
        0b_010 => WireType::VarLength,
        0b_001 => WireType::Fixed64,
        0b_101 => WireType::Fixed32,
        0b_011 => WireType::_Obsolete3,
        0b_100 => WireType::_Obsolete4,
        0b_110 => WireType::_Undefined6,
        0b_111 => WireType::_Undefined7,
        _ => unreachable!(),
    }
}

// write a varint together with the read_wire_type tag
#[inline]
pub fn write_varint(writer: &mut impl Write, tag: WireType, mut value: u64) -> Result<()> {
    let tag = tag as u8;
    let partial = ((value & 15) << 3) as u8;
    value >>= 4;
    if value == 0 {
        writer.write(&[tag | partial])?;
        return Ok(());
    }
    // 10 bytes supports 4 + 9 * 7 = 67 bits of data
    let mut b = [0u8; 10];
    b[0] = tag | partial | 0x80;
    let mut len = 1;
    loop {
        let partial = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            b[len] = partial;
            len += 1;
            break;
        }
        b[len] = partial | 0x80;
        len += 1;
    }
    writer.write_all(&b[..len])?;
    Ok(())
}

// read a varint, given a tag byte and remaining data; returns the value and
// the size consumed from data
#[inline]
pub fn read_varint(tag_byte: u8, data: &[u8]) -> Result<(u64, usize)> {
    if tag_byte & 0x80 == 0 {
        let value = tag_byte >> 3;
        return Ok((value as u64, 0));
    }
    let mut value = ((tag_byte & 0x7f) >> 3) as u64;
    let mut shift = 4;
    // note I've tested with a fast/slow variant, where the fast variant doesn't
    // need to check for end of input, but it doesn't make it faster; the test
    // is negligible
    for (i, b) in data.iter().copied().enumerate() {
        if shift >= 64 {
            return Err(Error::ValueOverflow);
        }
        if b & 0x80 == 0 {
            value |= (b as u64) << shift;
            return Ok((value, i + 1));
        }
        value |= ((b & 0x7f) as u64) << shift;
        shift += 7;
    }
    Err(Error::UnexpectedEndOfInput)
}

#[inline]
pub fn skip_varint(tag_byte: u8, data: &[u8]) -> Result<usize> {
    if tag_byte & 0x80 == 0 {
        return Ok(0);
    }
    for (i, b) in data.iter().copied().enumerate() {
        // if we reach byte 18, we've consumed 19 bytes including tag byte, exceeding
        // max encoding of a 128-bit varint
        if i == 18 {
            return Err(Error::ValueOverflow);
        }
        if b & 0x80 == 0 {
            return Ok(i + 1);
        }
    }
    Err(Error::UnexpectedEndOfInput)
}

#[test]
fn test_varint() {
    let mut buf = vec![];

    write_varint(&mut buf, WireType::VarInt, 15).unwrap();
    assert_eq!(buf.len(), 1);
    assert_eq!(read_varint(buf[0], &buf[1..]).unwrap(), (15, 0));

    buf.clear();
    write_varint(&mut buf, WireType::VarInt, u64::MAX).unwrap();
    assert_eq!(buf.len(), 10);
    assert_eq!(read_varint(buf[0], &buf[1..]).unwrap(), (u64::MAX, 9));
}

serde::serde_if_integer128! {
    #[inline]
    pub fn write_varint_128(writer: &mut impl Write, tag: WireType, mut value: u128) -> Result<()> {
        let tag = tag as u8;
        let partial = ((value & 15) << 3) as u8;
        value >>= 4;
        if value == 0 {
            writer.write(&[tag | partial])?;
            return Ok(());
        }
        // 19 bytes supports 4 + 18 x 7 = 130 bits of data
        let mut b = [0u8; 19];
        b[0] = tag | partial | 0x80;
        let mut len = 1;
        loop {
            let partial = (value & 0x7f) as u8;
            value >>= 7;
            if value == 0 {
                b[len] = partial;
                len += 1;
                break;
            }
            b[len] = partial | 0x80;
            len += 1;
        }
        writer.write_all(&b[..len])?;
        Ok(())
    }

    #[inline]
    pub fn read_varint_128(tag_byte: u8, data: &[u8]) -> Result<(u128, usize)> {
        if tag_byte & 0x80 == 0 {
            let value = tag_byte >> 3;
            return Ok((value as u128, 0));
        }
        let mut value = ((tag_byte & 0x7f) >> 3) as u128;
        let mut shift = 4;
        for (i,b) in data.iter().copied().enumerate() {
            if shift >= 128 {
                return Err(Error::ValueOverflow);
            }
            if b & 0x80 == 0 {
                value |= (b as u128) << shift;
                return Ok((value, i + 1));
            }
            value |= ((b & 0x7f) as u128) << shift;
            shift += 7;
        }
        Err(Error::UnexpectedEndOfInput)
    }

    #[test]
    fn test_varint_128() {
        let mut buf = vec![];

        write_varint_128(&mut buf, WireType::VarInt, u128::MAX).unwrap();
        assert_eq!(buf.len(), 19);
        assert_eq!(read_varint_128(buf[0], &buf[1..]).unwrap(), (u128::MAX, 18));
    }
}

// signed varints use google's zig-zag method

#[inline]
pub fn zigzag_encode(value: i64) -> u64 {
    let encoded = (value << 1) ^ (value >> 63);
    encoded as u64
}

#[inline]
pub fn zigzag_decode(encoded: u64) -> i64 {
    (encoded >> 1) as i64 ^ -(encoded as i64 & 1)
}

#[test]
fn test_zigzag() {
    assert_eq!(zigzag_decode(zigzag_encode(42)), 42);
    assert_eq!(zigzag_decode(zigzag_encode(-42)), -42);
    assert_eq!(zigzag_decode(zigzag_encode(0)), 0);
    assert_eq!(zigzag_decode(zigzag_encode(i64::MAX)), i64::MAX);
    assert_eq!(zigzag_decode(zigzag_encode(i64::MIN)), i64::MIN);

    assert!(zigzag_encode(-10) < zigzag_encode(100));
    assert!(zigzag_encode(10) < zigzag_encode(-100));
}

serde::serde_if_integer128! {
    #[inline]
    pub fn zigzag_encode_128(value: i128) -> u128 {
        let encoded = (value << 1) ^ (value >> 127);
        encoded as u128
    }


    #[inline]
    pub fn zigzag_decode_128(encoded: u128) -> i128 {
        (encoded >> 1) as i128 ^ -(encoded as i128 & 1)
    }

    #[test]
    fn test_zigzag_128() {
        assert_eq!(zigzag_decode_128(zigzag_encode_128(42)), 42);
        assert_eq!(zigzag_decode_128(zigzag_encode_128(-42)), -42);
        assert_eq!(zigzag_decode_128(zigzag_encode_128(i128::MAX)), i128::MAX);
        assert_eq!(zigzag_decode_128(zigzag_encode_128(i128::MIN)), i128::MIN);

        assert_eq!(zigzag_decode_128(zigzag_encode(42) as u128), 42);
        assert_eq!(zigzag_decode_128(zigzag_encode(-42) as u128), -42);

        assert_eq!(zigzag_decode(zigzag_encode_128(42) as u64), 42);
        assert_eq!(zigzag_decode(zigzag_encode_128(-42) as u64), -42);
    }
}
