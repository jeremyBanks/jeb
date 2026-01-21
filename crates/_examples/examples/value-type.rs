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

impl PartialOrd for ValueTypes {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
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
