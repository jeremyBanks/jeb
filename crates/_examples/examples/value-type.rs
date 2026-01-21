use std::cmp::Ordering;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueTypes {
    pub null: bool,
    pub boolean: bool,
    pub number: bool,
    pub bytes: bool,
    pub string: bool,
    pub array: bool,
    pub bytes_map: bool,
    pub string_map: bool,
}

impl From<ValueTypes> for u8 {
    fn from(value: ValueTypes) -> Self {
        let mut bits = 0;
        if value.null {
            bits |= 1 << 0;
        }
        if value.boolean {
            bits |= 1 << 1;
        }
        if value.number {
            bits |= 1 << 2;
        }
        if value.bytes {
            bits |= 1 << 3;
        }
        if value.string {
            bits |= 1 << 4;
        }
        if value.array {
            bits |= 1 << 5;
        }
        if value.bytes_map {
            bits |= 1 << 6;
        }
        if value.string_map {
            bits |= 1 << 7;
        }
        bits
    }
}

impl PartialOrd for ValueTypes {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {}
}

impl ValueTypes {
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
