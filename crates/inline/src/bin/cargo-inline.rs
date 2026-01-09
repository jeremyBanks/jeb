#!/usr/bin/env rust
//! Cargo subcommand wrapper that sets INLINE_MODE=write.
//!
//! Usage:
//!   cargo inline <COMMAND> [ARGS...]
//!
//! This is equivalent to:
//!   INLINE_MODE=write cargo <COMMAND> [ARGS...]
//!
//! Examples:
//!   cargo inline test
//!   cargo inline test -- --nocapture
//!   cargo inline test test_name
//!   cargo inline run --example demo

use std::{
    env,
    process::{
        Command,
        exit,
    },
};

fn main() {
    // Cargo invokes this as: cargo-inline inline [args...]
    // We need to skip the first argument if it's the subcommand name
    let mut args: Vec<String> = env::args().collect();

    // Remove the binary name
    args.remove(0);

    // If the first arg is "inline", remove it (cargo passes the subcommand name)
    if args.first().map(|s| s.as_str()) == Some("inline") {
        args.remove(0);
    }

    // Set the environment variable to enable write mode
    env::set_var("INLINE_MODE", "write");

    // Execute cargo with all the provided arguments
    let mut cmd = Command::new("cargo");
    cmd.args(&args);

    // Preserve the current environment (including our INLINE_MODE=write)
    cmd.envs(env::vars());

    // Execute and forward the exit code
    let status = cmd.status().unwrap_or_else(|e| {
        eprintln!("Failed to execute cargo: {}", e);
        exit(1);
    });

    exit(status.code().unwrap_or(1));
}
