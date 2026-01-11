#!/usr/bin/env rust
//! Cargo subcommand wrapper that sets INLINE_MODE=write.
//!
//! Usage:
//!   cargo inline-write <COMMAND> [ARGS...]
//!
//! This is equivalent to:
//!   INLINE_MODE=write cargo <COMMAND> [ARGS...]
//!
//! Examples:
//!   cargo inline-write test
//!   cargo inline-write test -- --nocapture
//!   cargo inline-write test test_name
//!   cargo inline-write run --example demo

use std::{
    env,
    process::{
        Command,
        exit,
    },
};

fn main() {
    let mut args: Vec<String> = env::args().collect();

    // Remove the binary name
    args.remove(0);

    // If the first arg is "inline-write", remove it (cargo passes the subcommand
    // name)
    if args.first().map(|s| s.as_str()) == Some("inline-write") {
        args.remove(0);
    }

    let status = Command::new("cargo")
        .args(&args)
        .env("INLINE_MODE", "write")
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Failed to execute cargo: {}", e);
            exit(1);
        });

    exit(status.code().unwrap_or(1));
}
