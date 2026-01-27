//! Validation utilities for polyglot PNG+ZIP files.
//!
//! Uses the `image` crate for PNG validation and `zip` crate for ZIP validation.

/// Result of validating a polyglot file.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid_png: bool,
    pub is_valid_zip: bool,
    pub png_width: u32,
    pub png_height: u32,
    pub zip_file_count: usize,
    pub zip_total_uncompressed_size: usize,
    pub zip_files: Vec<String>,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.is_valid_png && self.is_valid_zip && self.errors.is_empty()
    }
}

/// Validate a polyglot PNG+ZIP file using external crates.
#[cfg(test)]
pub fn validate_polyglot(data: &[u8]) -> ValidationResult {
    use std::io::Cursor;

    let mut result = ValidationResult {
        is_valid_png: false,
        is_valid_zip: false,
        png_width: 0,
        png_height: 0,
        zip_file_count: 0,
        zip_total_uncompressed_size: 0,
        zip_files: Vec::new(),
        errors: Vec::new(),
    };

    // Validate PNG using image crate
    match image::load_from_memory_with_format(data, image::ImageFormat::Png) {
        Ok(img) => {
            result.is_valid_png = true;
            result.png_width = img.width();
            result.png_height = img.height();
        }
        Err(e) => {
            result.errors.push(format!("PNG validation failed: {}", e));
        }
    }

    // Validate ZIP using zip crate
    let cursor = Cursor::new(data);
    match zip::ZipArchive::new(cursor) {
        Ok(mut archive) => {
            result.is_valid_zip = true;
            result.zip_file_count = archive.len();

            for i in 0..archive.len() {
                match archive.by_index(i) {
                    Ok(file) => {
                        result.zip_files.push(file.name().to_string());
                        result.zip_total_uncompressed_size += file.size() as usize;
                    }
                    Err(e) => {
                        result.errors.push(format!("ZIP file {} error: {}", i, e));
                    }
                }
            }
        }
        Err(e) => {
            result.errors.push(format!("ZIP validation failed: {}", e));
        }
    }

    result
}

/// Assert that data is a valid polyglot PNG+ZIP.
/// Panics with detailed error message if validation fails.
#[cfg(test)]
pub fn assert_valid_polyglot(data: &[u8]) -> ValidationResult {
    let result = validate_polyglot(data);

    if !result.is_valid() {
        panic!(
            "Invalid polyglot!\n\
             PNG valid: {} ({}x{})\n\
             ZIP valid: {} ({} files, {} bytes)\n\
             Errors:\n  {}",
            result.is_valid_png,
            result.png_width,
            result.png_height,
            result.is_valid_zip,
            result.zip_file_count,
            result.zip_total_uncompressed_size,
            result.errors.join("\n  ")
        );
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polyglot::build_polyglot;
    use crate::png::{BitDepth, ColorType};

    #[test]
    fn test_simple_polyglot() {
        let files = vec![(b"test.txt".as_ref(), b"Hello!".as_ref())];
        let polyglot = build_polyglot(&files, 0, BitDepth::EightBit, ColorType::Luminance, None);

        let result = assert_valid_polyglot(&polyglot);
        assert_eq!(result.zip_file_count, 1);
        assert_eq!(result.zip_files, vec!["test.txt"]);
        assert_eq!(result.zip_total_uncompressed_size, 6);
    }

    #[test]
    fn test_multiple_files() {
        let files = vec![
            (b"a.txt".as_ref(), b"File A content".as_ref()),
            (b"b.txt".as_ref(), b"File B content".as_ref()),
            (b"c.txt".as_ref(), b"File C".as_ref()),
        ];
        let polyglot = build_polyglot(&files, 0, BitDepth::EightBit, ColorType::Luminance, None);

        let result = assert_valid_polyglot(&polyglot);
        assert_eq!(result.zip_file_count, 3);
    }

    #[test]
    fn test_indexed_color() {
        use crate::palettes::viridis::VIRIDIS;

        let files = vec![(b"test.txt".as_ref(), b"Hello!".as_ref())];
        let polyglot = build_polyglot(&files, 0, BitDepth::EightBit, ColorType::Indexed, Some(VIRIDIS));

        let result = assert_valid_polyglot(&polyglot);
        assert_eq!(result.zip_file_count, 1);
    }

    #[test]
    fn test_large_content() {
        let large_content = vec![0x42u8; 50_000]; // 50KB of data
        let files = vec![(b"large.bin".as_ref(), large_content.as_ref())];
        let polyglot = build_polyglot(&files, 0, BitDepth::EightBit, ColorType::Luminance, None);

        let result = assert_valid_polyglot(&polyglot);
        assert_eq!(result.zip_file_count, 1);
        assert_eq!(result.zip_total_uncompressed_size, 50_000);
    }

    #[test]
    fn test_real_files() {
        // Test with actual project files like the zipng example uses
        let cargo_toml = include_bytes!("../../Cargo.toml");
        let readme = include_bytes!("../../README.md");

        let files = vec![
            (b"assets/Cargo.toml".as_ref(), cargo_toml.as_ref()),
            (b"assets/README.md".as_ref(), readme.as_ref()),
        ];
        // Use Luminance (grayscale) which doesn't require a palette
        let polyglot = build_polyglot(&files, 0, BitDepth::EightBit, ColorType::Luminance, None);

        let result = assert_valid_polyglot(&polyglot);
        assert_eq!(result.zip_file_count, 2);
        assert_eq!(result.zip_total_uncompressed_size, cargo_toml.len() + readme.len());
    }

    #[test]
    fn test_long_filenames() {
        // Test with filenames longer than the base minimum row width
        // This ensures the row width is increased to accommodate long filenames
        let files = vec![
            (b"this-is-a-very-long-filename-that-exceeds-forty-characters.txt".as_ref(), b"Content A".as_ref()),
            (b"another/deeply/nested/path/with/many/components/file.dat".as_ref(), b"Content B".as_ref()),
            (b"short.txt".as_ref(), b"Content C".as_ref()),
        ];
        let polyglot = build_polyglot(&files, 0, BitDepth::EightBit, ColorType::Luminance, None);

        let result = assert_valid_polyglot(&polyglot);
        assert_eq!(result.zip_file_count, 3);

        // Verify the filenames are intact
        assert!(result.zip_files.contains(&"this-is-a-very-long-filename-that-exceeds-forty-characters.txt".to_string()));
        assert!(result.zip_files.contains(&"another/deeply/nested/path/with/many/components/file.dat".to_string()));
        assert!(result.zip_files.contains(&"short.txt".to_string()));
    }

    #[test]
    fn test_zipng_sorts_files() {
        use indexmap::IndexMap;
        use crate::Files;

        // Create files in non-sorted order
        let mut files = IndexMap::new();
        files.insert(b"z_last.txt".to_vec(), b"Z".to_vec());
        files.insert(b"a_first.txt".to_vec(), b"A".to_vec());
        files.insert(b"m_middle.txt".to_vec(), b"M".to_vec());

        let polyglot = crate::zipng(&Files { files });
        let result = assert_valid_polyglot(&polyglot);

        // Files should be sorted lexicographically
        assert_eq!(result.zip_files, vec!["a_first.txt", "m_middle.txt", "z_last.txt"]);
    }

    #[test]
    fn test_zipng_uses_indexed_color_for_small_files() {
        use indexmap::IndexMap;
        use crate::Files;

        // Small file should use indexed color (with palette)
        let mut files = IndexMap::new();
        files.insert(b"small.txt".to_vec(), b"Hello".to_vec());

        let polyglot = crate::zipng(&Files { files });
        let result = assert_valid_polyglot(&polyglot);
        assert_eq!(result.zip_file_count, 1);

        // Check for PLTE chunk presence (indicates indexed color)
        assert!(polyglot.windows(4).any(|w| w == b"PLTE"), "Small files should use indexed color with PLTE chunk");
    }

    #[test]
    fn test_zipng_deterministic_palette() {
        use indexmap::IndexMap;
        use crate::Files;

        // Same input should produce identical output (deterministic palette selection)
        let mut files = IndexMap::new();
        files.insert(b"test.txt".to_vec(), b"Hello, World!".to_vec());

        let polyglot1 = crate::zipng(&Files { files: files.clone() });
        let polyglot2 = crate::zipng(&Files { files });

        assert_eq!(polyglot1, polyglot2, "Same input should produce identical output");
    }

    #[test]
    fn test_filename_labels_small_file() {
        // Small files should have filename labels rendered
        // The labels are 5 rows each (1 empty + 3 text + 1 empty)
        let files = vec![
            (b"hello.txt".as_ref(), b"Hello World!".as_ref()),
        ];
        let polyglot = build_polyglot(&files, 0, BitDepth::EightBit, ColorType::Luminance, None);

        // Should still be valid
        let result = assert_valid_polyglot(&polyglot);
        assert_eq!(result.zip_file_count, 1);

        // The image should have more rows due to labels
        // Check that we can extract the expected file
        assert_eq!(result.zip_files, vec!["hello.txt"]);
    }

    #[test]
    fn test_filename_labels_disabled_for_large() {
        // Very large files should not have labels (height > 1024 would result)
        // Create content that would make image very tall with labels
        let large_content = vec![0x42u8; 50_000];
        let files: Vec<_> = (0..20)
            .map(|i| {
                let name = format!("file_{:02}.bin", i);
                (name.into_bytes(), large_content.clone())
            })
            .collect();
        let file_refs: Vec<(&[u8], &[u8])> = files
            .iter()
            .map(|(n, b)| (n.as_slice(), b.as_slice()))
            .collect();

        let polyglot = build_polyglot(&file_refs, 0, BitDepth::EightBit, ColorType::Luminance, None);

        // Should still be valid
        let result = assert_valid_polyglot(&polyglot);
        assert_eq!(result.zip_file_count, 20);

        // Check image dimensions - height should be reasonable (< ~2000 or so)
        // If labels were shown, height would be much larger
        assert!(result.png_height < 3000, "Height {} should be limited without labels", result.png_height);
    }
}
