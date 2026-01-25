//! Integration tests for snap! macro and Snap type with source file updates.

use {
    std::{
        env,
        fs,
    },
    tempfile::TempDir,
};

/// Helper to find snap!() macro positions in a file
fn find_snap_positions(file_path: &std::path::Path) -> Vec<(u32, u32)> {
    let source = fs::read_to_string(file_path).unwrap();
    let ast = syn::parse_file(&source).unwrap();

    use syn::visit::Visit;
    struct MacroCollector {
        positions: Vec<(u32, u32)>,
    }

    impl<'ast> Visit<'ast> for MacroCollector {
        fn visit_expr(&mut self, node: &'ast syn::Expr) {
            if let syn::Expr::Macro(mac) = node {
                if let Some(segment) = mac.mac.path.segments.last() {
                    if segment.ident == "snap" {
                        let span = segment.ident.span();
                        let start = span.start();
                        // proc_macro2 uses 0-indexed columns, but Location::caller()
                        // uses 1-indexed columns. Add 1 to match the runtime API.
                        self.positions
                            .push((start.line as u32, start.column as u32 + 1));
                    }
                }
            }
            syn::visit::visit_expr(self, node);
        }
    }

    let mut collector = MacroCollector { positions: vec![] };
    syn::visit::visit_file(&mut collector, &ast);
    collector.positions
}

#[test]
fn test_snap_updates_source_file() {
    // Create a temp file with snap! macro
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = snap!(42);
}
"#;
    fs::write(&path, source).unwrap();

    // SAFETY: Test-only; no concurrent access to this env var in this test
    unsafe { env::set_var("INLINE_MODE", "write") };
    inline::clear_file_state_cache();

    // Find the snap! macro position
    let positions = find_snap_positions(&path);
    assert_eq!(positions.len(), 1);
    let (line, col) = positions[0];

    // Create a Snap with explicit location and compare against a different value
    // We need to leak the path string to get a 'static lifetime
    let path_str: &'static str = Box::leak(path.to_str().unwrap().to_string().into_boxed_str());

    {
        let snap = inline::Snap::__new(42i32, path_str, line, col);
        let _ = snap == 100i32; // Should update 42 to 100
    }

    // Verify the file was updated (databake adds type suffix like 100i32)
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("snap!(100") && !content.contains("snap!(42"),
        "Expected file to contain snap!(100...), got:\n{}",
        content
    );

    // SAFETY: Test-only; no concurrent access to this env var in this test
    unsafe { env::remove_var("INLINE_MODE") };
}

#[test]
fn test_snap_with_string_updates_to_raw_string() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = snap!("old");
}
"#;
    fs::write(&path, source).unwrap();

    // SAFETY: Test-only; no concurrent access to this env var in this test
    unsafe { env::set_var("INLINE_MODE", "write") };
    inline::clear_file_state_cache();

    let positions = find_snap_positions(&path);
    let (line, col) = positions[0];
    let path_str: &'static str = Box::leak(path.to_str().unwrap().to_string().into_boxed_str());

    {
        let snap = inline::Snap::__new("old", path_str, line, col);
        let actual = "hello\nworld".to_string();
        let _ = snap == actual; // Should update to multi-line raw string
    }

    let content = fs::read_to_string(&path).unwrap();
    // Should contain the raw string with actual newline
    assert!(
        content.contains("snap!(r\"hello\nworld\")")
            || content.contains("snap!(r#\"hello\nworld\"#)"),
        "Expected file to contain raw string, got:\n{}",
        content
    );

    // SAFETY: Test-only; no concurrent access to this env var in this test
    unsafe { env::remove_var("INLINE_MODE") };
}

#[test]
fn test_snap_no_update_when_values_match() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = snap!(42);
}
"#;
    fs::write(&path, source).unwrap();

    // SAFETY: Test-only; no concurrent access to this env var in this test
    unsafe { env::set_var("INLINE_MODE", "write") };
    inline::clear_file_state_cache();

    let positions = find_snap_positions(&path);
    let (line, col) = positions[0];
    let path_str: &'static str = Box::leak(path.to_str().unwrap().to_string().into_boxed_str());

    {
        let snap = inline::Snap::__new(42i32, path_str, line, col);
        let _ = snap == 42i32; // Values match, no update
    }

    // File should be unchanged
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, source);

    // SAFETY: Test-only; no concurrent access to this env var in this test
    unsafe { env::remove_var("INLINE_MODE") };
}
