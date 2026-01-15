//! Test cell!() with a more complex expression

use {
    inline::InlineCellPrivate,
    std::{env, fs},
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
fn test_cell_macro_with_expression() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = cell!(1 + 2);
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("INLINE_MODE", "write");
    inline::clear_file_state_cache();

    let positions = find_macro_positions(&path);
    assert_eq!(positions.len(), 1, "Should find one macro");
    let (line, col) = positions[0];

    {
        let mut cell = inline::InlineCell::__new(3i32, path.to_str().unwrap(), line, col);
        cell.value = 10i32;
    }

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("10i32"),
        "File should contain updated value. Actual:\n{}",
        content
    );
    assert!(
        content.contains("cell!("),
        "Macro syntax should be preserved. Actual:\n{}",
        content
    );

    env::remove_var("INLINE_MODE");
}
