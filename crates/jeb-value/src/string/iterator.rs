use crate::String;

// [impl jeb-value.string.from-char-iterator]
impl FromIterator<char> for String {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        String(iter.into_iter().collect())
    }
}

// [impl jeb-value.string.from-string-iterator]
impl FromIterator<std::string::String> for String {
    fn from_iter<T: IntoIterator<Item = std::string::String>>(iter: T) -> Self {
        String(iter.into_iter().collect())
    }
}

// [impl jeb-value.string.from-str-iterator]
impl<'a> FromIterator<&'a str> for String {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        String(iter.into_iter().collect())
    }
}
