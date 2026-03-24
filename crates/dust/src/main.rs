use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: dust <file.rs> [-o output.wasm]");
        process::exit(1);
    }

    let input_path = &args[1];
    let output_path = if args.len() >= 4 && args[2] == "-o" {
        args[3].clone()
    } else {
        let stem = input_path
            .strip_suffix(".rs")
            .unwrap_or(input_path);
        format!("{}.wasm", stem)
    };

    let source = match fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read '{}': {}", input_path, e);
            process::exit(1);
        }
    };

    let mut lexer = dust::lexer::Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            let span = dust::span::Span::new(e.pos, e.pos);
            let (line, col) = span.line_col(&source);
            eprintln!("{}:{}:{}: error: {}", input_path, line, col, e.msg);
            process::exit(1);
        }
    };

    let trees = match dust::token_tree::build_token_tree(tokens) {
        Ok(t) => t,
        Err(e) => {
            let span = dust::span::Span::new(e.pos, e.pos);
            let (line, col) = span.line_col(&source);
            eprintln!("{}:{}:{}: error: {}", input_path, line, col, e.msg);
            process::exit(1);
        }
    };

    let ast = match dust::parser::Parser::new(&trees).parse_file() {
        Ok(a) => a,
        Err(e) => {
            let (line, col) = e.span.line_col(&source);
            eprintln!("{}:{}:{}: error: {}", input_path, line, col, e.msg);
            process::exit(1);
        }
    };

    let wasm_bytes = match dust::compiler::compile(&ast) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = fs::write(&output_path, &wasm_bytes) {
        eprintln!("error: cannot write '{}': {}", output_path, e);
        process::exit(1);
    }

    eprintln!("wrote {} bytes to {}", wasm_bytes.len(), output_path);
}
