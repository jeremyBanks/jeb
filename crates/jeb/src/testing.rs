pub fn print_doc_block(doc_strings: &[&str]) {
    if doc_strings.is_empty() {
        return;
    }

    // Group doc strings: multi-line strings (/** */) are processed alone,
    // consecutive single-line strings (///) are grouped together
    let mut i = 0;
    while i < doc_strings.len() {
        let s = doc_strings[i];
        if s.contains('\n') {
            // Multi-line block comment - process alone
            print_single_doc_group(&[s]);
            i += 1;
        } else {
            // Single-line - gather consecutive single-line strings
            let start = i;
            while i < doc_strings.len() && !doc_strings[i].contains('\n') {
                i += 1;
            }
            print_single_doc_group(&doc_strings[start..i]);
        }
    }
}

pub fn print_single_doc_group(doc_strings: &[&str]) {
    if doc_strings.is_empty() {
        return;
    }

    // Join all doc strings with newlines, then split into lines
    let combined = doc_strings.join("\n");
    let lines: Vec<&str> = combined
        .lines()
        .map(|line| if line.trim().is_empty() { "" } else { line })
        .collect();
    let mut lines: &[&str] = &lines;

    // Strip one leading and one trailing empty line if present
    if lines.first().map(|s| s.is_empty()).unwrap_or(false) {
        lines = &lines[1..];
    }
    if lines.last().map(|s| s.is_empty()).unwrap_or(false) {
        lines = &lines[..lines.len() - 1];
    }

    if lines.is_empty() {
        return;
    }

    // Find minimum leading whitespace among non-empty lines
    let min_indent = lines
        .iter()
        .filter(|line| !line.is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    // Build dedented text
    let dedented: String = lines
        .iter()
        .map(|line| {
            if line.is_empty() {
                ""
            } else {
                &line[min_indent..]
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Print with bat markdown highlighting (add trailing newline to content)
    let content = format!("{}\n", dedented);
    eprintln!();
    ::bat::PrettyPrinter::new()
        .input_from_bytes(content.as_bytes())
        .language("markdown")
        .print()
        .unwrap();
}

pub fn print_code(code: &str) {
    eprintln!(); // Blank line before code

    // Normalize indentation: strip common leading whitespace, re-indent with 4 spaces
    let lines: Vec<&str> = code.lines().collect();

    // Find minimum leading whitespace among non-empty lines
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    // Build normalized text: strip common indent, add 4-space indent
    let normalized: String = lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("    {}", &line[min_indent..])
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let content = format!("{}\n", normalized);
    ::bat::PrettyPrinter::new()
        .input_from_bytes(content.as_bytes())
        .language("rust")
        .print()
        .unwrap();
}

/// Internal macro to collect consecutive doc comments
#[macro_export]
macro_rules! literate_docs {
    // Done collecting docs, hit fn item
    ([$($doc:literal),*] fn $($item:tt)*) => {
        $crate::testing::print_doc_block(&[$($doc),*]);
        $crate::literate_fn!([fn] $($item)*);
    };
    // Done collecting docs, hit static item
    ([$($doc:literal),*] static $($item:tt)*) => {
        $crate::testing::print_doc_block(&[$($doc),*]);
        $crate::literate_static_const!([static] $($item)*);
    };
    // Done collecting docs, hit const item
    ([$($doc:literal),*] const $($item:tt)*) => {
        $crate::testing::print_doc_block(&[$($doc),*]);
        $crate::literate_static_const!([const] $($item)*);
    };
    // No more input - just emit the docs
    ([$($doc:literal),*]) => {
        $crate::testing::print_doc_block(&[$($doc),*]);
    };
    // Accumulate another doc comment
    ([$($doc:literal),*] #[doc = $next:literal] $($rest:tt)*) => {
        $crate::literate_docs!([$($doc,)* $next] $($rest)*);
    };
    // Done collecting docs, hit anything else - print docs, then process statement
    ([$($doc:literal),*] $first:tt $($rest:tt)*) => {
        $crate::testing::print_doc_block(&[$($doc),*]);
        $crate::literate_stmt!([$first] $($rest)*);
    };
}

/// Internal macro for fn items - accumulate until body, then stringify & emit
#[macro_export]
macro_rules! literate_fn {
    // Found the body (a brace group) - stringify accumulated + body, emit
    ([$($acc:tt)*] { $($body:tt)* } $($rest:tt)*) => {
        $crate::testing::print_code(::stringify_verbatim::stringify_verbatim!($($acc)* { $($body)* }));
        $($acc)* { $($body)* }
        $crate::literate_inner!($($rest)*);
    };
    // Accumulate next token
    ([$($acc:tt)*] $next:tt $($rest:tt)*) => {
        $crate::literate_fn!([$($acc)* $next] $($rest)*);
    };
}

/// Internal macro for static/const items - accumulate until semicolon, then
/// stringify & emit
#[macro_export]
macro_rules! literate_static_const {
    // Found the semicolon - stringify accumulated tokens, emit
    ([$($acc:tt)*] ; $($rest:tt)*) => {
        $crate::testing::print_code(concat!(::stringify_verbatim::stringify_verbatim!($($acc)*), ";"));
        $($acc)*;
        $crate::literate_inner!($($rest)*);
    };
    // Accumulate next token
    ([$($acc:tt)*] $next:tt $($rest:tt)*) => {
        $crate::literate_static_const!([$($acc)* $next] $($rest)*);
    };
}

/// Internal macro for TT-munching statements until we hit a semicolon.
/// When we find a semicolon, we stringify THAT statement and emit it, then continue.
/// This captures verbatim at the right time - before further macro processing.
#[macro_export]
macro_rules! literate_stmt {
    // Hit a semicolon - stringify the accumulated tokens now (before more processing)
    ([$($acc:tt)*] ; $($rest:tt)*) => {
        $crate::testing::print_code(concat!(::stringify_verbatim::stringify_verbatim!($($acc)*), ";"));
        $($acc)*;
        $crate::literate_inner!($($rest)*);
    };

    // Accumulate a token
    ([$($acc:tt)*] $next:tt $($rest:tt)*) => {
        $crate::literate_stmt!([$($acc)* $next] $($rest)*);
    };
}

/// Internal macro for processing literate body
#[macro_export]
macro_rules! literate_inner {
    // Empty - done
    () => {};

    // Doc comment - start collecting
    (#[doc = $doc:literal] $($rest:tt)*) => {
        $crate::literate_docs!([$doc] $($rest)*);
    };

    // fn item - start accumulating with fn as first token
    (fn $($item:tt)*) => {
        $crate::literate_fn!([fn] $($item)*);
    };

    // static item - start accumulating with static as first token
    (static $($item:tt)*) => {
        $crate::literate_static_const!([static] $($item)*);
    };

    // const item - start accumulating with const as first token
    (const $($item:tt)*) => {
        $crate::literate_static_const!([const] $($item)*);
    };

    // Anything else - start TT-munching for a statement
    ($first:tt $($rest:tt)*) => {
        $crate::literate_stmt!([$first] $($rest)*);
    };
}

/// Main entry point macro - wraps body in a main function
#[macro_export]
macro_rules! literate {
    ($($body:tt)*) => {
        pub fn main() {
            $crate::literate_inner!($($body)*);
        }
    };
}
pub use {
    literate,
    literate_docs,
    literate_fn,
    literate_inner,
    literate_static_const,
    literate_stmt,
};
