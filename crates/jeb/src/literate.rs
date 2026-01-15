/// Literate programming-style test macro.
///
/// Transforms doc comments into printed prose and shows the code being executed.
/// Useful for creating self-documenting, narrative-style tests.
///
/// # Example
///
/// ```
/// use jeb::literate;
///
/// literate! {
///     /// # Fibonacci Test
///     ///
///     /// We start with the base cases:
///     let a = 0;
///     let b = 1;
///
///     /// Then compute the next value:
///     let c = a + b;
///
///     /// And verify the result:
///     assert_eq!(c, 1);
/// }
/// ```
///
/// Output:
/// ```text
/// # Fibonacci Test
///
/// We start with the base cases:
///
///     let a = 0
///     let b = 1
///
/// Then compute the next value:
///
///     let c = a + b
///
/// And verify the result:
///
///     assert_eq!(c, 1)
/// ```
#[macro_export]
macro_rules! literate {
    // Empty doc line (bare `///`)
    (#[doc = ""] $($rest:tt)*) => {
        println!();
        $crate::literate!($($rest)*);
    };

    // Doc comment followed by another doc comment
    (#[doc = $doc:literal] #[$($next_attr:tt)*] $($rest:tt)*) => {
        println!("{}", $doc.trim());
        $crate::literate!(#[$($next_attr)*] $($rest)*);
    };

    // Doc comment followed by statement, then another doc (prose -> code -> prose)
    (#[doc = $doc:literal] $stmt:stmt; #[$($next_attr:tt)*] $($rest:tt)*) => {
        println!("{}", $doc.trim());
        println!();
        println!("    {}", stringify!($stmt));
        $stmt
        println!();
        $crate::literate!(#[$($next_attr)*] $($rest)*);
    };

    // Doc comment followed by statement
    (#[doc = $doc:literal] $stmt:stmt; $($rest:tt)*) => {
        println!("{}", $doc.trim());
        println!();
        println!("    {}", stringify!($stmt));
        $stmt
        $crate::literate!($($rest)*);
    };

    // Trailing doc comment (no statement after)
    (#[doc = $doc:literal]) => {
        println!("{}", $doc.trim());
    };

    // Regular statement followed by doc comment (code -> prose transition)
    ($stmt:stmt; #[$($next_attr:tt)*] $($rest:tt)*) => {
        println!("    {}", stringify!($stmt));
        $stmt
        println!();
        $crate::literate!(#[$($next_attr)*] $($rest)*);
    };

    // Regular statement (no doc comment follows)
    ($stmt:stmt; $($rest:tt)*) => {
        println!("    {}", stringify!($stmt));
        $stmt
        $crate::literate!($($rest)*);
    };

    // Base case: empty
    () => {};
}

#[cfg(test)]
mod tests {
    #[test]
    #[allow(unused_doc_comments)]
    fn test_literate_macro() {
        literate! {
            /// # Addition Test
            ///
            /// We create two numbers:
            let a = 2;
            let b = 3;

            /// Add them together:
            let sum = a + b;

            /// Verify the result:
            assert_eq!(sum, 5);
        }
    }
}
