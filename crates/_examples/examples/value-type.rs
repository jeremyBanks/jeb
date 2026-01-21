use std::cmp::Ordering;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueType {
    pub null: bool,
    pub boolean: bool,
    pub number: bool,
    pub bytes: bool,
    pub string: bool,
    pub array: bool,
    pub bytes_map: bool,
    pub string_map: bool,
}

impl ValueType {
    fn bitmask(&self) -> u8 {
        let mut bits = 0;
        if self.null {
            bits |= 1 << 0;
        }
        if self.boolean {
            bits |= 1 << 1;
        }
        if self.number {
            bits |= 1 << 2;
        }
        if self.bytes {
            bits |= 1 << 3;
        }
        if self.string {
            bits |= 1 << 4;
        }
        if self.array {
            bits |= 1 << 5;
        }
        if self.bytes_map {
            bits |= 1 << 6;
        }
        if self.string_map {
            bits |= 1 << 7;
        }
        bits
    }

    pub fn is_a(&self, other: ValueType) -> bool {
        let self_bitmask = self.bitmask();
        let other_bitmask = other.bitmask();
        let union = self_bitmask | other_bitmask;
        union == self_bitmask
    }
}

impl PartialOrd for ValueType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_bitmask = self.bitmask();
        let other_bitmask = other.bitmask();
        let union = self_bitmask | other_bitmask;

        if self == other {
            Some(Ordering::Equal)
        } else if union == self_bitmask {
            Some(Ordering::Greater)
        } else if union == other_bitmask {
            Some(Ordering::Less)
        } else {
            None
        }
    }
}

impl ValueType {
    pub const NONE: Self = Self {
        null: false,
        boolean: false,
        number: false,
        bytes: false,
        string: false,
        array: false,
        bytes_map: false,
        string_map: false,
    };

    pub const ANY: Self = Self {
        null: true,
        boolean: true,
        number: true,
        bytes: true,
        string: true,
        array: true,
        bytes_map: true,
        string_map: true,
    };

    pub const NULL: Self = Self {
        null: true,
        ..Self::NONE
    };

    pub const BOOLEAN: Self = Self {
        boolean: true,
        ..Self::NONE
    };

    pub const NUMBER: Self = Self {
        number: true,
        ..Self::NONE
    };

    pub const STRING: Self = Self {
        string: true,
        ..Self::NONE
    };

    pub const ARRAY: Self = Self {
        array: true,
        ..Self::NONE
    };

    pub const BYTES_MAP: Self = Self {
        bytes_map: true,
        ..Self::NONE
    };

    pub const STRING_MAP: Self = Self {
        string_map: true,
        ..Self::NONE
    };

    pub const SCALAR: Self = Self {
        null: true,
        boolean: true,
        number: true,
        string: true,
        ..Self::NONE
    };

    pub const JSON: Self = Self {
        null: true,
        boolean: true,
        number: true,
        string: true,
        array: true,
        string_map: true,
        ..Self::NONE
    };
}

fn main() {}
