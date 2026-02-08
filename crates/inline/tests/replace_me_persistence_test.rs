//! Test that multiple calls from the same location return the same value

use {
    inline::replace_at,
    std::{
        env,
        fs,
    },
    tempfile::TempDir,
};

fn find_call_position(file_path: &std::path::Path) -> (u32, u32) {
    let source = fs::read_to_string(file_path).unwrap();
    let ast = syn::parse_file(&source).unwrap();

    use syn::visit::Visit;
    struct CallFinder {
        position: Option<(u32, u32)>,
    }

    impl<'ast> Visit<'ast> for CallFinder {
        fn visit_expr(&mut self, node: &'ast syn::Expr) {
            if self.position.is_none() {
                if let syn::Expr::Call(call) = node {
                    use syn::spanned::Spanned;
                    let span = call.func.span();
                    let start = span.start();
                    self.position = Some((start.line as u32, start.column as u32 + 1));
                }
            }
            syn::visit::visit_expr(self, node);
        }
    }

    let mut finder = CallFinder { position: None };
    finder.visit_file(&ast);
    finder.position.expect("Should find a function call")
}

#[test]
fn test_replace_me_persistence() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");
    fs::write(
        &path,
        r#"fn main() {
    let x = replace_me(42u32);
}
"#,
    )
    .unwrap();

    // SAFETY: Test-only; no concurrent access to this env var in this test
    unsafe { env::set_var("INLINE_MODE", "memory") };
    inline::clear_file_state_cache();

    let (line, column) = find_call_position(&path);

    // First call stores the value
    let result1 = replace_at(100u32, path.to_str().unwrap(), line, column);
    assert_eq!(result1, 100u32);

    // Second call from same location returns the stored value, ignoring the new
    // argument
    let result2 = replace_at(999u32, path.to_str().unwrap(), line, column);
    assert_eq!(result2, 100u32); // Should still be 100, not 999

    // SAFETY: Test-only; no concurrent access to this env var in this test
    unsafe { env::remove_var("INLINE_MODE") };
}
