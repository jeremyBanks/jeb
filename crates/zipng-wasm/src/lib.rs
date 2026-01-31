//! WASM bindings for zipng polyglot encoder.
//!
//! Provides a JavaScript-friendly API with JSON serialization and concrete types.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// Set up better panic messages for debugging
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single file to be encoded into the polyglot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInput {
    /// File path (used as ZIP entry name).
    pub path: String,
    /// File content as byte array.
    pub content: Vec<u8>,
}

/// Encoding options for the polyglot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodeOptions {
    /// Color mode: "auto" (default), "indexed", or "rgba".
    #[serde(default = "default_mode")]
    pub mode: String,

    /// Font choice: "swiss", "sixth", "sky", "monte", "sugimori", "mini", "micro".
    /// None uses automatic font selection.
    #[serde(default)]
    pub font: Option<String>,

    /// Sort mode: "lexicographic" (default), "reverse", "by_size", "by_extension", "none".
    #[serde(default = "default_sort_mode")]
    pub sort_mode: String,
}

fn default_mode() -> String {
    "auto".to_string()
}

fn default_sort_mode() -> String {
    "lexicographic".to_string()
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self {
            mode: default_mode(),
            font: None,
            sort_mode: default_sort_mode(),
        }
    }
}

/// Input structure for the encode function.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncodeInput {
    files: Vec<FileInput>,
    #[serde(default)]
    options: EncodeOptions,
}

// ---------------------------------------------------------------------------
// WASM API
// ---------------------------------------------------------------------------

/// Encode files into a polyglot PNG+ZIP.
///
/// # Arguments
/// * `files_json` - JSON string containing files and options:
///   ```json
///   {
///     "files": [
///       {"path": "hello.txt", "content": [72, 101, 108, 108, 111]},
///       {"path": "data.bin", "content": [0, 1, 2, 3]}
///     ],
///     "options": {
///       "mode": "auto",
///       "font": "swiss",
///       "sort_mode": "lexicographic"
///     }
///   }
///   ```
///
/// # Returns
/// PNG+ZIP polyglot as Uint8Array.
///
/// # Errors
/// - Invalid JSON
/// - File >60KB (IDAT boundary limitation)
/// - Invalid mode/font/sort_mode values
#[wasm_bindgen]
pub fn encode(files_json: &str) -> Result<Vec<u8>, JsValue> {
    // Parse input
    let input: EncodeInput = serde_json::from_str(files_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid JSON: {}", e)))?;

    // Validate file sizes
    const MAX_FILE_SIZE: usize = 60 * 1024; // 60KB limit
    for file in &input.files {
        if file.content.len() > MAX_FILE_SIZE {
            return Err(JsValue::from_str(&format!(
                "File '{}' exceeds 60KB limit ({} bytes)",
                file.path,
                file.content.len()
            )));
        }
    }

    // Convert files to tuples
    let mut file_tuples: Vec<(Vec<u8>, Vec<u8>)> = input
        .files
        .into_iter()
        .map(|f| (f.path.into_bytes(), f.content))
        .collect();

    // Apply sorting
    match input.options.sort_mode.as_str() {
        "lexicographic" => {
            file_tuples.sort_by(|a, b| a.0.cmp(&b.0));
        }
        "reverse" => {
            file_tuples.sort_by(|a, b| b.0.cmp(&a.0));
        }
        "by_size" => {
            file_tuples.sort_by(|a, b| a.1.len().cmp(&b.1.len()));
        }
        "by_extension" => {
            file_tuples.sort_by(|a, b| {
                let ext_a = extract_extension(&a.0);
                let ext_b = extract_extension(&b.0);
                ext_a.cmp(&ext_b).then_with(|| a.0.cmp(&b.0))
            });
        }
        "none" => {
            // Keep insertion order
        }
        other => {
            return Err(JsValue::from_str(&format!(
                "Invalid sort_mode: '{}'. Must be one of: lexicographic, reverse, by_size, by_extension, none",
                other
            )));
        }
    }

    // Build encoder
    let mut encoder = zipng::v2::Encoder::new();

    // Set mode
    encoder = match input.options.mode.as_str() {
        "auto" => encoder.with_mode(zipng::v2::EncoderMode::Auto),
        "indexed" => encoder.with_mode(zipng::v2::EncoderMode::Indexed),
        "rgba" => encoder.with_mode(zipng::v2::EncoderMode::Rgba),
        other => {
            return Err(JsValue::from_str(&format!(
                "Invalid mode: '{}'. Must be one of: auto, indexed, rgba",
                other
            )));
        }
    };

    // Set font if specified
    if let Some(font_name) = input.options.font {
        encoder = match font_name.as_str() {
            "swiss" => encoder.with_font(zipng::v2::FontChoice::Swiss),
            "sixth" => encoder.with_font(zipng::v2::FontChoice::Sixth),
            "sky" => encoder.with_font(zipng::v2::FontChoice::Sky),
            "monte" => encoder.with_font(zipng::v2::FontChoice::Monte),
            "sugimori" => encoder.with_font(zipng::v2::FontChoice::Sugimori),
            "mini" => encoder.with_font(zipng::v2::FontChoice::Mini),
            "micro" => encoder.with_font(zipng::v2::FontChoice::Micro),
            other => {
                return Err(JsValue::from_str(&format!(
                    "Invalid font: '{}'. Must be one of: swiss, sixth, sky, monte, sugimori, mini, micro",
                    other
                )));
            }
        };
    }

    // Disable default sorting (we handled it above)
    encoder = encoder.with_sorted(false);

    // Encode
    let output = encoder.encode(file_tuples);

    Ok(output)
}

/// Extract file extension from path bytes.
fn extract_extension(path: &[u8]) -> &[u8] {
    if let Some(dot_pos) = path.iter().rposition(|&b| b == b'.') {
        &path[dot_pos + 1..]
    } else {
        b""
    }
}

// ---------------------------------------------------------------------------
// Convenience wrapper for simpler API
// ---------------------------------------------------------------------------

/// Encode files with default options (auto mode, lexicographic sort).
///
/// # Arguments
/// * `files_json` - JSON array of files:
///   ```json
///   [
///     {"path": "hello.txt", "content": [72, 101, 108, 108, 111]},
///     {"path": "data.bin", "content": [0, 1, 2, 3]}
///   ]
///   ```
#[wasm_bindgen]
pub fn encode_simple(files_json: &str) -> Result<Vec<u8>, JsValue> {
    // Parse as array of files
    let files: Vec<FileInput> = serde_json::from_str(files_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid JSON: {}", e)))?;

    // Wrap in EncodeInput with defaults
    let input = EncodeInput {
        files,
        options: EncodeOptions::default(),
    };

    // Serialize and call main encode
    let input_json = serde_json::to_string(&input)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;

    encode(&input_json)
}

// ---------------------------------------------------------------------------
// Version info
// ---------------------------------------------------------------------------

/// Get the version of zipng-wasm.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_simple() {
        let files_json = r#"[
            {"path": "hello.txt", "content": [72, 101, 108, 108, 111]},
            {"path": "world.txt", "content": [119, 111, 114, 108, 100]}
        ]"#;

        let result = encode_simple(files_json);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.is_empty());
        assert_eq!(&output[..4], b"\x89PNG");
    }

    #[test]
    fn test_encode_with_options() {
        let input_json = r#"{
            "files": [
                {"path": "a.txt", "content": [97, 97, 97]},
                {"path": "b.txt", "content": [98, 98, 98]}
            ],
            "options": {
                "mode": "indexed",
                "font": "swiss",
                "sort_mode": "reverse"
            }
        }"#;

        let result = encode(input_json);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.is_empty());
        assert_eq!(&output[..4], b"\x89PNG");
    }

    #[test]
    fn test_file_size_validation() {
        let large_file = FileInput {
            path: "large.bin".to_string(),
            content: vec![0; 61 * 1024], // 61KB
        };

        let input = EncodeInput {
            files: vec![large_file],
            options: EncodeOptions::default(),
        };

        let input_json = serde_json::to_string(&input).unwrap();
        let result = encode(&input_json);

        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("exceeds 60KB limit"));
    }

    #[test]
    fn test_sort_modes() {
        let files = vec![
            FileInput {
                path: "z.txt".to_string(),
                content: vec![1],
            },
            FileInput {
                path: "a.txt".to_string(),
                content: vec![2],
            },
        ];

        // Test lexicographic (should sort a before z)
        let input = EncodeInput {
            files: files.clone(),
            options: EncodeOptions {
                sort_mode: "lexicographic".to_string(),
                ..Default::default()
            },
        };
        let result = encode(&serde_json::to_string(&input).unwrap());
        assert!(result.is_ok());

        // Test reverse (should sort z before a)
        let input = EncodeInput {
            files: files.clone(),
            options: EncodeOptions {
                sort_mode: "reverse".to_string(),
                ..Default::default()
            },
        };
        let result = encode(&serde_json::to_string(&input).unwrap());
        assert!(result.is_ok());

        // Test none (preserve order)
        let input = EncodeInput {
            files: files.clone(),
            options: EncodeOptions {
                sort_mode: "none".to_string(),
                ..Default::default()
            },
        };
        let result = encode(&serde_json::to_string(&input).unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_mode() {
        let input_json = r#"{
            "files": [{"path": "test.txt", "content": [116, 101, 115, 116]}],
            "options": {"mode": "invalid"}
        }"#;

        let result = encode(input_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_extension() {
        assert_eq!(extract_extension(b"file.txt"), b"txt");
        assert_eq!(extract_extension(b"path/to/file.rs"), b"rs");
        assert_eq!(extract_extension(b"no_extension"), b"");
        assert_eq!(extract_extension(b".hidden"), b"hidden");
    }
}
