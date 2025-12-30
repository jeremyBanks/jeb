//! git-zoom: Extract and embed subtrees within git repositories.
//!
//! Usage:
//!   git zoom in [path] [--allow-empty]
//!   git zoom out [target[:path]] [--deny-empty]

mod git;
mod scan;
mod tree;
mod zoom_in;
mod zoom_out;
#[allow(unused)]
mod gits;

use std::env;
use std::process::ExitCode;

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  git-zoom in [path] [--allow-empty]");
    eprintln!("  git-zoom out [target[:path]] [--deny-empty]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  in   Zoom into a subtree, making it the working tree root");
    eprintln!("  out  Zoom out, embedding current tree back into full tree");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --allow-empty  Allow zooming into non-existent path (creates empty tree)");
    eprintln!("  --deny-empty   Error if subtree would be empty when zooming out");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return ExitCode::from(1);
    }

    // Check preconditions
    if let Err(e) = git::check_repo_root() {
        eprintln!("Error: {}", e);
        return ExitCode::from(1);
    }

    if let Err(e) = git::check_clean() {
        eprintln!("Error: {}", e);
        return ExitCode::from(1);
    }

    let command = &args[1];

    match command.as_str() {
        "in" => {
            let mut path: Option<&str> = None;
            let mut allow_empty = false;

            for arg in &args[2..] {
                if arg == "--allow-empty" {
                    allow_empty = true;
                } else if !arg.starts_with('-') {
                    if path.is_some() {
                        eprintln!("Error: multiple paths provided (expected at most one)");
                        return ExitCode::from(1);
                    }
                    path = Some(arg);
                } else {
                    eprintln!("Unknown option: {}", arg);
                    return ExitCode::from(1);
                }
            }

            if let Err(e) = zoom_in::zoom_in(path, allow_empty) {
                eprintln!("Error: {}", e);
                return ExitCode::from(1);
            }
        }
        "out" => {
            let mut target_path: Option<&str> = None;
            let mut deny_empty = false;

            for arg in &args[2..] {
                if arg == "--deny-empty" {
                    deny_empty = true;
                } else if !arg.starts_with('-') {
                    if target_path.is_some() {
                        eprintln!("Error: multiple arguments provided (expected at most one)");
                        return ExitCode::from(1);
                    }
                    target_path = Some(arg);
                } else {
                    eprintln!("Unknown option: {}", arg);
                    return ExitCode::from(1);
                }
            }

            if let Err(e) = zoom_out::zoom_out(target_path, deny_empty) {
                eprintln!("Error: {}", e);
                return ExitCode::from(1);
            }
        }
        "help" | "--help" | "-h" => {
            print_usage();
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            return ExitCode::from(1);
        }
    }

    ExitCode::SUCCESS
}
