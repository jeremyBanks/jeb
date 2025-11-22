use std::{env, path::PathBuf};

fn main() {
    let mut flags = Vec::<String>::new();

    let home_dir = env::var("HOME").unwrap_or_else(|_| env::var("USERPROFILE").unwrap_or_default());
    flags.push(format!("--remap-path-prefix={home_dir}=~",));

    let crate_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let ancestors = crate_path.ancestors().into_iter().collect::<Vec<_>>();

    for (i, local_path) in ancestors.iter().enumerate().peekable() {
        let local_path = local_path.to_str().unwrap();

        let normalized_path = if i == 0 {
            ".".to_string()
        } else if i + 1 == ancestors.len() {
            "/".to_string()
        } else {
            "../".repeat(i).strip_suffix("/").unwrap().to_string()
        };

        flags.push(format!(
            "--remap-path-prefix={local_path}={normalized_path}",
        ));
    }

    let encoded_rustflags = flags.join("\x1F");

    println!("cargo:rustc-env=CARGO_ENCODED_RUSTFLAGS={encoded_rustflags}");
    println!("cargo:rustc-env=SOURCE_DATE_EPOCH=1608040201");
    println!("cargo:rustc-env=TZ=UTC");
    println!("cargo:rustc-env=LC_ALL=C");
}
