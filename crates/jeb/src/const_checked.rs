#![expect(clippy::must_use_candidate)]

use core::mem::swap;


#[macro_export]
macro_rules! noop {
    ($($x:expr $(;)?)+) => {
        $(
            const _: [(); {
                {
                    $x
                };
                0
            }] = [];
        )+
    };
}

/// const-compatible order-preserving set of unique byte values. The byte in the
/// set will always make up the first `length` items of the `bytes` array, in
/// specified order (if any). Most of the const method implementations are very
/// slow and should not be used at runtime.
#[derive(Debug, Copy, Clone)]
#[must_use]
pub struct OrderedByteSet {
    bytes: [u8; 256],
    len: usize,
}

impl AsRef<[u8]> for OrderedByteSet {
    fn as_ref(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

impl OrderedByteSet {
    pub const ALL: Self = Self {
        bytes: [
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
            0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF, 0xC0, 0xC1, 0xC2, 0xC3,
            0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF, 0xD0, 0xD1,
            0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xDB, 0xDC, 0xDD, 0xDE, 0xDF,
            0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED,
            0xEE, 0xEF, 0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFB,
            0xFC, 0xFD, 0xFE, 0xFF,
        ],
        len: 256,
    };
    pub const NONE: Self = Self {
        len: 0,
        ..Self::ALL
    };

    pub const fn len(self) -> usize {
        self.len
    }

    pub const fn is_empty(self) -> bool {
        self.len > 0
    }

    pub const fn from_bytes(bytes: &[u8]) -> Self {
        let mut set = Self::NONE;

        let mut index = 0;
        loop {
            if (index >= bytes.len()) {
                break;
            }

            set.insert(bytes[index]);

            index += 1;
        }

        set
    }

    const fn index_of(self, byte: u8) -> usize {
        let mut index = 0;
        loop {
            if (self.bytes[index] == byte) {
                return index;
            }

            index += 1;
            if (index > 0xFF) {
                return -1isize as usize;
            }
        }
    }

    pub const fn insert(&mut self, byte: u8) -> Option<u8> {
        let existing_index = self.index_of(byte);

        if (existing_index < self.len()) {
            return Some(byte);
        }

        self.bytes.swap(existing_index, self.len);

        self.len += 1;

        None
    }

    pub const fn remove(&mut self, byte: u8) -> Option<u8> {
        let existing_index = self.index_of(byte);

        if (existing_index >= self.len()) {
            return None;
        }

        self.len -= 1;

        let mut existing_index = existing_index;
        while (existing_index < self.len()) {
            self.bytes.swap(existing_index, existing_index + 1);
            existing_index += 1;
        }

        Some(byte)
    }

    pub const fn add(self, other: Self) -> Self {
        let mut result = self;
        let mut index = 0;
        while (index < other.len()) {
            result.insert(other.bytes[index]);
            index += 1;
        }
        result
    }

    pub const fn sub(self, other: Self) -> Self {
        let mut result = self;
        let mut index = 0;
        while (index < other.len()) {
            result.remove(other.bytes[index]);
            index += 1;
        }
        result
    }

    pub const fn and(self, other: Self) -> Self {
        let mut result = Self::NONE;
        let mut index = 0;
        while (index < self.len()) {
            let byte = self.bytes[index];
            if (other.contains(byte)) {
                result.insert(byte);
            }
            index += 1;
        }
        result
    }

    /// # Panics
    ///
    /// Panics if the length of `self` does not match `LENGTH`.
    pub const fn to_array<const LENGTH: usize>(self) -> [u8; LENGTH] {
        let mut array = [0u8; LENGTH];

        if (self.len() > LENGTH) {
            panic!("computed array had larger than expected size")
        } else if (self.len() < LENGTH) {
            panic!("computed array had smaller than expected size")
        }

        let mut index: usize = 0;
        loop {
            array[index] = self.bytes[index];

            index += 1;
            if (index >= LENGTH) {
                break;
            }
        }

        array
    }

    pub const fn contains(&self, byte: u8) -> bool {
        self.index_of(byte) < self.len()
    }

    pub const fn invert(self) -> Self {
        Self::ALL.sub(self)
    }

    pub const fn presence_lut(self) -> [bool; 256] {
        let mut lut = [false; 256];

        let mut index = 0;
        while (index < self.len()) {
            let byte = self.bytes[index];
            lut[byte as usize] = true;

            index += 1;
        }

        lut
    }

    pub const fn index_lut(self) -> [u8; 256] {
        let mut lut = [0xFF; 256];

        let mut index = 0;
        while (index < self.len()) {
            let byte = self.bytes[index];
            lut[byte as usize] = index as u8;

            index += 1;
        }

        lut
    }

    pub const fn sort(self) -> Self {
        Self::ALL.and(self)
    }

    /// # Panics
    ///
    /// Panics if the length or contents of `expected` do not match `self`.
    pub const fn eq(self, expected: &[u8]) -> Self {
        if expected.len() < self.len() {
            panic!("calculated value had lower length than expected value");
        } else if expected.len() > self.len() {
            panic!("calculated value had greater length than expected value");
        }

        let mut index = 0;
        while index < expected.len() {
            if expected[index] != self.bytes[index] {
                panic!("calculated value did not match expected value");
            }
            index += 1;
        }

        self
    }
}

#[expect(non_snake_case)]
pub const fn OBS(b: &[u8]) -> OrderedByteSet {
    OrderedByteSet::from_bytes(b)
}

/// Asserts that both `usize` arguments are equal, then returns that value.
///
/// # Panics
///
/// Panics if `expected` and `calculation` are not equal.
pub const fn usize_eq(expected: usize, calculation: usize) -> usize {
    if expected != calculation {
        panic!("calculated value did not match expected value");
    }

    expected
}

/// Asserts that both `&[u8]` arguments are equal, then returns that value.
///
/// # Panics
///
/// Panics if the length or contents of `expected` and `calculated` do not
/// match.
pub const fn bytes_eq<'a>(expected: &'a [u8], calculated: &'a [u8]) -> &'a [u8] {
    if expected.len() < calculated.len() {
        panic!("calculated value had lower length than expected value");
    } else if expected.len() > calculated.len() {
        panic!("calculated value had greater length than expected value");
    }

    let mut index = 0;
    while index < expected.len() {
        if expected[index] != calculated[index] {
            panic!("calculated value did not match expected value");
        }
        index += 1;
    }

    expected
}

/// Performs exact division, ensuring there is no remainder.
///
/// # Panics
///
/// Panics if `dividend` is not evenly divisible by `divisor`.
pub const fn div_exact(dividend: usize, divisor: usize) -> usize {
    if !dividend.is_multiple_of(divisor) {
        panic!("remainder in div_exact");
    }

    dividend / divisor
}

pub const fn pow(base: usize, exponent: usize) -> usize {
    let mut result: usize = 1;

    let mut iterations: usize = 0;
    while iterations < exponent {
        result *= base;
        iterations += 1;
    }

    result
}
