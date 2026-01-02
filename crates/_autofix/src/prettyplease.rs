use {
    std::{
        fs,
        path::Path,
    },
    walkdir::WalkDir,
};
pub fn main() -> i32 {
    eprintln!("Running: prettyplease formatting on all .rs files");
    let mut failed_count = 0;
    let mut processed_count = 0;
    let mut modified_count = 0;
    for entry in WalkDir::new(".")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let path = entry.path();
        if path.components().any(|c| c.as_os_str() == "target") {
            continue;
        }
        processed_count += 1;
        match format_file(path) {
            Ok(true) => {
                modified_count += 1;
            }
            Ok(false) => {}
            Err(e) => {
                eprintln!("  Error formatting {}: {}", path.display(), e);
                failed_count += 1;
            }
        }
    }
    eprintln!(
        "  Processed {} files, modified {}, {} errors",
        processed_count, modified_count, failed_count,
    );
    eprintln!();
    if failed_count > 0 { 1 } else { 0 }
}
/// Format a single Rust file. Returns Ok(true) if modified, Ok(false) if
/// unchanged.
fn format_file(path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let original = fs::read_to_string(path)?;
    let syntax_tree = syn::parse_file(&original).map_err(|e| format!("parse error: {}", e))?;
    let formatted = prettyplease::unparse(&syntax_tree);
    if original == formatted {
        return Ok(false);
    }
    fs::write(path, formatted)?;
    Ok(true)
}
