//! Test replacing with a more complex type (Vec)

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
fn test_replace_me_with_vec() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");
    fs::write(
        &path,
        r#"fn main() {
    let x = replace_me(vec![1, 2, 3]);
}
"#,
    )
    .unwrap();

    env::set_var("INLINE_MODE", "write");
    inline::clear_file_state_cache();

    let (line, column) = find_call_position(&path);
    let result = replace_at(vec![1i32, 2, 3], path.to_str().unwrap(), line, column);

    assert_eq!(result, vec![1i32, 2, 3]);

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        !content.contains("replace_me"),
        "replace_me should be replaced. Content:\n{}",
        content
    );

    env::remove_var("INLINE_MODE");
}
