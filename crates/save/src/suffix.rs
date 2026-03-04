//! Suffix requirement for commit hash brute-forcing.

/// Requirement for the nibble (hex digit) following the target prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuffixRequirement {
    /// No requirement - any nibble is acceptable.
    None,
    /// Requires a letter (a-f, nibble value 0xA-0xF).
    Letter,
    /// Requires a digit (0-9, nibble value 0x0-0x9).
    Digit,
}

impl SuffixRequirement {
    /// Check if a nibble value (0x0-0xF) satisfies this requirement.
    pub fn check(self, nibble: u8) -> bool {
        match self {
            SuffixRequirement::None => true,
            SuffixRequirement::Letter => nibble >= 0xA,
            SuffixRequirement::Digit => nibble <= 0x9,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none() {
        for nibble in 0x0..=0xF {
            assert!(SuffixRequirement::None.check(nibble));
        }
    }

    #[test]
    fn test_letter() {
        // Digits should fail
        for nibble in 0x0..=0x9 {
            assert!(!SuffixRequirement::Letter.check(nibble));
        }
        // Letters should pass
        for nibble in 0xA..=0xF {
            assert!(SuffixRequirement::Letter.check(nibble));
        }
    }

    #[test]
    fn test_digit() {
        // Digits should pass
        for nibble in 0x0..=0x9 {
            assert!(SuffixRequirement::Digit.check(nibble));
        }
        // Letters should fail
        for nibble in 0xA..=0xF {
            assert!(!SuffixRequirement::Digit.check(nibble));
        }
    }
}
