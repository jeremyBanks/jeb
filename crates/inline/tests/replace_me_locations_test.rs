//! Test that different locations get different values

use {
    inline::replace_at,
    std::{env, fs},
    tempfile::TempDir,
};

#[test]
fn test_replace_me_different_locations() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");
    fs::write(
        &path,
        r#"fn main() {
    let x = replace_me(10u32);
    let y = replace_me(20u32);
}
"#,
    )
    .unwrap();

    env::set_var("INLINE_MODE", "memory");
    inline::clear_file_state_cache();

    // Find both positions
    let source = fs::read_to_string(&path).unwrap();
    let ast = syn::parse_file(&source).unwrap();

    use syn::visit::Visit;
    struct CallFinder {
        positions: Vec<(u32, u32)>,
    }

    impl<'ast> Visit<'ast> for CallFinder {
        fn visit_expr(&mut self, node: &'ast syn::Expr) {
            if let syn::Expr::Call(call) = node {
                use syn::spanned::Spanned;
                let span = call.func.span();
                let start = span.start();
                self.positions
                    .push((start.line as u32, start.column as u32 + 1));
            }
            syn::visit::visit_expr(self, node);
        }
    }

    let mut finder = CallFinder {
        positions: Vec::new(),
    };
    finder.visit_file(&ast);

    assert_eq!(finder.positions.len(), 2, "Should find two calls");

    let (line1, col1) = finder.positions[0];
    let (line2, col2) = finder.positions[1];

    // Each location gets its own value
    let result1 = replace_at(10u32, path.to_str().unwrap(), line1, col1);
    let result2 = replace_at(20u32, path.to_str().unwrap(), line2, col2);

    assert_eq!(result1, 10u32);
    assert_eq!(result2, 20u32);

    env::remove_var("INLINE_MODE");
}
