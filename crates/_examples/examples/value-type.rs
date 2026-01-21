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

impl ValueTypes {
    pub const JSON: Self = Self {
        null: true,
        boolean: true,
        number: true,
        string: true,
        array: true,
        string_map: true,

        ..Self::default()
    };
    pub const SCALAR: Self = Self {
        null: true,
        boolean: true,
        number: true,
        string: true,
    };
}

fn main() {}
