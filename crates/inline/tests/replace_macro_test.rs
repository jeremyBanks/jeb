//! Test that replace!() replaces the entire macro invocation

use {
    std::{
        env,
        fs,
    },
    tempfile::TempDir,
};

fn find_macro_positions(path: &std::path::Path) -> Vec<(u32, u32)> {
    let source = fs::read_to_string(path).unwrap();
    let ast = syn::parse_file(&source).unwrap();

    use syn::visit::Visit;
    struct MacroFinder {
        positions: Vec<(u32, u32)>,
    }

    impl<'ast> Visit<'ast> for MacroFinder {
        fn visit_expr(&mut self, node: &'ast syn::Expr) {
            if let syn::Expr::Macro(mac) = node {
                if let Some(seg) = mac.mac.path.segments.last() {
                    let name = seg.ident.to_string();
                    if name == "cell" || name == "replace" {
                        use syn::spanned::Spanned;
                        let span = mac.mac.path.span();
                        let start = span.start();
                        self.positions
                            .push((start.line as u32, start.column as u32 + 1));
                    }
                }
            }
            syn::visit::visit_expr(self, node);
        }
    }

    let mut finder = MacroFinder {
        positions: Vec::new(),
    };
    finder.visit_file(&ast);
    finder.positions
}

#[test]
fn test_replace_macro_basic() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = replace!(42u32);
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("INLINE_MODE", "write");
    inline::clear_file_state_cache();

    let positions = find_macro_positions(&path);
    assert_eq!(positions.len(), 1, "Should find one macro");
    let (line, col) = positions[0];

    let result = inline::replace_at(100u32, path.to_str().unwrap(), line, col);
    assert_eq!(result, 100u32);

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("100u32"),
        "File should contain the baked value. Actual:\n{}",
        content
    );
    assert!(
        !content.contains("replace!"),
        "Macro invocation should be replaced. Actual:\n{}",
        content
    );

    env::remove_var("INLINE_MODE");
}
