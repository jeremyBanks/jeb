use proc_macro2::{TokenStream, TokenTree};
use quote::quote;

fn print_spans(tokens: TokenStream, indent: usize) {
    for tt in tokens {
        let span = tt.span();
        let start = span.start();
        let prefix = " ".repeat(indent);
        match &tt {
            TokenTree::Group(g) => {
                println!("{}Group {:?} @ {}:{}-{}:{}", prefix, g.delimiter(),
                    start.line, start.column, span.end().line, span.end().column);
                print_spans(g.stream(), indent + 2);
            }
            TokenTree::Ident(i) => {
                println!("{}Ident '{}' @ {}:{}", prefix, i, start.line, start.column);
            }
            TokenTree::Punct(p) => {
                println!("{}Punct '{}' @ {}:{}", prefix, p, start.line, start.column);
            }
            TokenTree::Literal(l) => {
                println!("{}Literal {} @ {}:{}", prefix, l, start.line, start.column);
            }
        }
    }
}

fn main() {
    // Simulate what the macro sees
    let tokens = quote! {
        fn hex_encode(bytes: &[u8]) -> String {
            let mut result = String::new();
            for byte in bytes {
                let high = (byte & 0xF0) >> 4;
            }
            result
        }
    };

    println!("Token spans from quote!:");
    print_spans(tokens, 0);
}
