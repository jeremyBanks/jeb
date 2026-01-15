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
    // Add trailing newline to content
    let content = format!("{}\n", code);
    eprintln!();
    ::bat::PrettyPrinter::new()
        .input_from_bytes(content.as_bytes())
        .language("rust")
        .print()
        .unwrap();
}

#[macro_export]
macro_rules! literate {
    // Base case - no more tokens, flush any buffered docs
    (@process [] [$($doc:literal),*] $($output:tt)*) => {
        $($output)*
        $crate::testing::print_doc_block(&[$($doc),*]);
    };

    // Accumulate consecutive doc comments into buffer
    (@process [#[doc = $doc:literal] $($rest:tt)*] [$($buf:literal),*] $($output:tt)*) => {
        literate!(@process [$($rest)*] [$($buf,)* $doc] $($output)*)
    };

    // Hit a let statement - flush buffer first, then process
    (@process [let $p:pat = $e:expr ; $($rest:tt)*] [$($buf:literal),*] $($output:tt)*) => {
        literate!(@process [$($rest)*] [] $($output)* $crate::testing::print_doc_block(&[$($buf),*]); $crate::testing::print_code(concat!(::stringify_verbatim::stringify_verbatim!(let $p = $e), ";")); let $p = $e;)
    };

    // Hit an expression statement - flush buffer first, then process
    (@process [$e:expr ; $($rest:tt)*] [$($buf:literal),*] $($output:tt)*) => {
        literate!(@process [$($rest)*] [] $($output)* $crate::testing::print_doc_block(&[$($buf),*]); $crate::testing::print_code(concat!(::stringify_verbatim::stringify_verbatim!($e), ";")); $e;)
    };

    // Hit a trailing expression - flush buffer first, then process
    (@process [$e:expr] [$($buf:literal),*] $($output:tt)*) => {
        $($output)* $crate::testing::print_doc_block(&[$($buf),*]); $crate::testing::print_code(::stringify_verbatim::stringify_verbatim!($e)); $e
    };

    // Main entry: create the test function with empty buffer
    ($($body:tt)*) => {
        pub fn main() {
            literate!(@process [$($body)*] []);
        }
    };
}
pub use literate;
