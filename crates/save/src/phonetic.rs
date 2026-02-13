//! NATO phonetic alphabet encoding for hex characters.
//!
//! Converts hexadecimal characters to their NATO phonetic alphabet equivalents
//! for human-readable, easily spoken commit identifiers.

/// Converts hex characters to NATO phonetic alphabet words.
///
/// # Arguments
///
/// * `hex` - A string containing hexadecimal characters (0-9, a-f, A-F)
///
/// # Returns
///
/// A space-separated string of lowercase phonetic words.
/// Non-hex characters are skipped.
///
/// # Examples
///
/// ```
/// use save::phonetic::hex_to_phonetic;
///
/// assert_eq!(hex_to_phonetic("4A92"), "four alfa nine two");
/// assert_eq!(hex_to_phonetic("DE64"), "delta echo six four");
/// assert_eq!(hex_to_phonetic(""), "");
/// ```
pub fn hex_to_phonetic(hex: &str) -> String {
    hex.chars()
        .filter_map(|c| {
            match c.to_ascii_lowercase() {
                '0' => Some("zero"),
                '1' => Some("one"),
                '2' => Some("two"),
                '3' => Some("three"),
                '4' => Some("four"),
                '5' => Some("five"),
                '6' => Some("six"),
                '7' => Some("seven"),
                '8' => Some("eight"),
                '9' => Some("nine"),
                'a' => Some("alfa"),
                'b' => Some("bravo"),
                'c' => Some("charlie"),
                'd' => Some("delta"),
                'e' => Some("echo"),
                'f' => Some("foxtrot"),
                _ => None, // Skip non-hex characters
            }
        })
        .collect::<Vec<&str>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_hex_digits() {
        // Test all digits 0-9
        assert_eq!(hex_to_phonetic("0"), "zero");
        assert_eq!(hex_to_phonetic("1"), "one");
        assert_eq!(hex_to_phonetic("2"), "two");
        assert_eq!(hex_to_phonetic("3"), "three");
        assert_eq!(hex_to_phonetic("4"), "four");
        assert_eq!(hex_to_phonetic("5"), "five");
        assert_eq!(hex_to_phonetic("6"), "six");
        assert_eq!(hex_to_phonetic("7"), "seven");
        assert_eq!(hex_to_phonetic("8"), "eight");
        assert_eq!(hex_to_phonetic("9"), "nine");

        // Test all hex letters a-f
        assert_eq!(hex_to_phonetic("a"), "alfa");
        assert_eq!(hex_to_phonetic("b"), "bravo");
        assert_eq!(hex_to_phonetic("c"), "charlie");
        assert_eq!(hex_to_phonetic("d"), "delta");
        assert_eq!(hex_to_phonetic("e"), "echo");
        assert_eq!(hex_to_phonetic("f"), "foxtrot");
    }

    #[test]
    fn test_case_insensitivity() {
        // Lowercase
        assert_eq!(hex_to_phonetic("abc"), "alfa bravo charlie");
        // Uppercase
        assert_eq!(hex_to_phonetic("ABC"), "alfa bravo charlie");
        // Mixed case
        assert_eq!(hex_to_phonetic("AbC"), "alfa bravo charlie");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(hex_to_phonetic(""), "");
    }

    #[test]
    fn test_historical_examples() {
        // From the historical bash-based commit script
        assert_eq!(hex_to_phonetic("4A92"), "four alfa nine two");
        assert_eq!(hex_to_phonetic("B470"), "bravo four seven zero");
    }

    #[test]
    fn test_non_hex_characters_skipped() {
        // Non-hex characters should be ignored
        assert_eq!(hex_to_phonetic("1-2-3"), "one two three");
        assert_eq!(hex_to_phonetic("A G"), "alfa");
    }

    #[test]
    fn test_all_combinations() {
        // Test a sequence with all hex characters
        assert_eq!(
            hex_to_phonetic("0123456789ABCDEF"),
            "zero one two three four five six seven eight nine alfa bravo charlie delta echo foxtrot"
        );
    }
}
