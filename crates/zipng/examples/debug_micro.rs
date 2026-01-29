//! Debug: for each file, show the pixel content of rows around its label.

use std::process::Command;
use zipng::Files;
use indexmap::IndexMap;

fn git_blob(hash: &str) -> Vec<u8> {
    let output = Command::new("git")
        .args(["cat-file", "-p", hash])
        .output()
        .unwrap();
    assert!(output.status.success());
    output.stdout
}

fn main() {
    let files: Vec<(&str, Vec<u8>)> = vec![
        ("history/tts.rs", git_blob("417a417289e71d71159c54cd8a1bf3f761b2e8ff")),
        ("history/ChapterPlayer.tsx", git_blob("6775460dc8205391fcfe8e2fcba1376fb4585d71")),
        ("history/rss.ts", git_blob("b4bc50599277271eb20ab3ebaba00b5580078b7d")),
        ("history/oklab.ts", git_blob("edc33782a6fcb821b3146fb8cad50fb72282b0c6")),
        ("history/RYL0051925.json", git_blob("6429e34ed36a6cc563167626d89973d64ab538d9")),
        ("history/RYL0021220.json", git_blob("8568558f3115ccd2859741b0b7ea1822beb169b5")),
        ("history/poc.htm", git_blob("5e2c1c3f950b74db76c34df838363d6a6be2c55a")),
        ("history/zip.xml", git_blob("9be3db2625f4ebc85ed17078eb22ecacf687071f")),
        ("history/polyglot_v1.rs", git_blob("105696cda2fa45fa53c98b403dc681f9d3ffd2d9")),
        ("history/polyglot_v2.rs", git_blob("77e6918a8f3b0128ea404045c323398f464914d5")),
        ("history/polyglot_v3.rs", git_blob("da359b39faf049612742a212af282893e096bfe9")),
        ("history/write_zipng_v1.rs", git_blob("580135c539bc0138087100c1dabcb0f7b7577592")),
        ("history/write_zipng_v2.rs", git_blob("fc2ebd0b541bf25d0459c57dfc9392e26442418b")),
        ("history/png_v1.rs", git_blob("01e49c4108c13a34a3fed61037916ac4dd98b401")),
        ("history/png_v2.rs", git_blob("ec87a5b3261fd6faf07c94929ee1f8cad5d386b5")),
        ("history/zip_v1.rs", git_blob("649a65c68e685b194c03bbd97d942bae2914bd6f")),
        ("assets/hybrid.png", git_blob("4f77dc8bc5201e9b4ae092b4c569b1cbac0aedb8")),
        ("assets/revolution.png", git_blob("cb4d4bb40c47238570b7fe0ce8ae5980ef7b7c6b")),
        ("assets/icon.png", git_blob("518ab5baa722247ee4b9311c43f251abdf7f908e")),
        ("assets/cover.png", git_blob("b3fc3f510f4f6c0b832bdc04c31daeab28602412")),
        ("assets/poc-gray.png", git_blob("ef554fe90423bdd8212bf1c1b7fb31470c612ba9")),
        ("assets/poc-turbo.png", git_blob("7d17ae102570957cfd7a9acfdee22088efa50219")),
        ("assets/sizes-2048.png", git_blob("cda9599db284f7bc413a8e0990f6e896e83ef829")),
        ("assets/sans701.woff2", git_blob("43f253e52c22ddc961267a7422f4f0a3fbdad3ff")),
        ("assets/sans401.woff2", git_blob("d35d3a78dc5df918f102b566115f22845bfcce31")),
        ("assets/sans700.woff2", git_blob("19a58eace2f37f645b2e724046288a979efafd84")),
        ("assets/sans400.woff2", git_blob("99b3c6f5e440bc94bb70b1727928066cadfd1bbf")),
        ("assets/favicon.ico", git_blob("b99cca8acdeaab2246318b3489fdda5c9544f425")),
        ("assets/poc.png.zip", git_blob("7d5e5c629b918f8dcbec25164b12484e949253dd")),
        ("snapshot-a/lib.rs", include_bytes!("../src/lib.rs").to_vec()),
        ("snapshot-a/checksums.rs", include_bytes!("../src/checksums.rs").to_vec()),
        ("snapshot-a/deflate.rs", include_bytes!("../src/deflate.rs").to_vec()),
        ("snapshot-a/zlib.rs", include_bytes!("../src/zlib.rs").to_vec()),
        ("snapshot-a/png/mod.rs", include_bytes!("../src/png.rs").to_vec()),
        ("snapshot-a/png/data.rs", include_bytes!("../src/png/data.rs").to_vec()),
        ("snapshot-a/png/to_png.rs", include_bytes!("../src/png/to_png.rs").to_vec()),
        ("snapshot-a/png/write_png.rs", include_bytes!("../src/png/write_png.rs").to_vec()),
        ("snapshot-a/png/sizes.rs", include_bytes!("../src/png/sizes.rs").to_vec()),
        ("snapshot-a/zip/mod.rs", include_bytes!("../src/zip.rs").to_vec()),
        ("snapshot-a/zip/data.rs", include_bytes!("../src/zip/data.rs").to_vec()),
        ("snapshot-a/zip/to_zip.rs", include_bytes!("../src/zip/to_zip.rs").to_vec()),
        ("snapshot-a/zip/write_zip.rs", include_bytes!("../src/zip/write_zip.rs").to_vec()),
        ("snapshot-a/zip/configuration.rs", include_bytes!("../src/zip/configuration.rs").to_vec()),
        ("snapshot-a/polyglot/fonts.rs", include_bytes!("../src/polyglot/fonts.rs").to_vec()),
        ("snapshot-a/polyglot/validate.rs", include_bytes!("../src/polyglot/validate.rs").to_vec()),
        ("snapshot-a/io/mod.rs", include_bytes!("../src/io.rs").to_vec()),
        ("snapshot-a/io/alignment.rs", include_bytes!("../src/io/alignment.rs").to_vec()),
        ("snapshot-a/palettes/mod.rs", include_bytes!("../src/png/palettes.rs").to_vec()),
        ("snapshot-a/palettes/viridis.rs", include_bytes!("../src/png/palettes/viridis.rs").to_vec()),
        ("snapshot-a/palettes/singles.rs", include_bytes!("../src/png/palettes/singles.rs").to_vec()),
        ("snapshot-a/palettes/diagnostic.rs", include_bytes!("../src/png/palettes/diagnostic.rs").to_vec()),
        ("snapshot-a/palettes/mappings.rs", include_bytes!("../src/png/palettes/mappings.rs").to_vec()),
        ("snapshot-a2/lib.rs", include_bytes!("../src/lib.rs").to_vec()),
        ("snapshot-a2/checksums.rs", include_bytes!("../src/checksums.rs").to_vec()),
        ("snapshot-a2/deflate.rs", include_bytes!("../src/deflate.rs").to_vec()),
        ("snapshot-a2/zlib.rs", include_bytes!("../src/zlib.rs").to_vec()),
        ("snapshot-a2/png/mod.rs", include_bytes!("../src/png.rs").to_vec()),
        ("snapshot-a2/png/data.rs", include_bytes!("../src/png/data.rs").to_vec()),
        ("snapshot-a2/png/to_png.rs", include_bytes!("../src/png/to_png.rs").to_vec()),
        ("snapshot-a2/png/write_png.rs", include_bytes!("../src/png/write_png.rs").to_vec()),
        ("snapshot-a2/png/sizes.rs", include_bytes!("../src/png/sizes.rs").to_vec()),
        ("snapshot-a2/zip/mod.rs", include_bytes!("../src/zip.rs").to_vec()),
        ("snapshot-a2/zip/data.rs", include_bytes!("../src/zip/data.rs").to_vec()),
        ("snapshot-a2/zip/to_zip.rs", include_bytes!("../src/zip/to_zip.rs").to_vec()),
        ("snapshot-a2/zip/write_zip.rs", include_bytes!("../src/zip/write_zip.rs").to_vec()),
        ("snapshot-a2/zip/configuration.rs", include_bytes!("../src/zip/configuration.rs").to_vec()),
        ("snapshot-a2/polyglot/fonts.rs", include_bytes!("../src/polyglot/fonts.rs").to_vec()),
        ("snapshot-a2/polyglot/validate.rs", include_bytes!("../src/polyglot/validate.rs").to_vec()),
        ("snapshot-a2/io/mod.rs", include_bytes!("../src/io.rs").to_vec()),
        ("snapshot-a2/io/alignment.rs", include_bytes!("../src/io/alignment.rs").to_vec()),
        ("snapshot-a2/palettes/mod.rs", include_bytes!("../src/png/palettes.rs").to_vec()),
        ("snapshot-a2/palettes/viridis.rs", include_bytes!("../src/png/palettes/viridis.rs").to_vec()),
        ("snapshot-a2/palettes/singles.rs", include_bytes!("../src/png/palettes/singles.rs").to_vec()),
        ("snapshot-a2/palettes/diagnostic.rs", include_bytes!("../src/png/palettes/diagnostic.rs").to_vec()),
        ("snapshot-a2/palettes/mappings.rs", include_bytes!("../src/png/palettes/mappings.rs").to_vec()),
        ("snapshot-b/fic/lib.rs", include_bytes!("../../../src/lib.rs").to_vec()),
        ("snapshot-b/fic/backend.rs", include_bytes!("../../../src/backend.rs").to_vec()),
        ("snapshot-b/fic/engine.rs", include_bytes!("../../../src/engine.rs").to_vec()),
        ("snapshot-b/fic/query.rs", include_bytes!("../../../src/query.rs").to_vec()),
        ("snapshot-b/Cargo.toml", include_bytes!("../Cargo.toml").to_vec()),
        ("snapshot-b/Cargo.lock", include_bytes!("../Cargo.lock").to_vec()),
        ("snapshot-b/README.md", include_bytes!("../README.md").to_vec()),
        ("snapshot-b/CLAUDE.md", include_bytes!("../CLAUDE.md").to_vec()),
        ("snapshot-b/examples/zipng.rs", include_bytes!("zipng.rs").to_vec()),
        ("snapshot-b/examples/rgb.rs", include_bytes!("rgb.rs").to_vec()),
        ("snapshot-b/examples/indexed8bit.rs", include_bytes!("indexed8bit.rs").to_vec()),
        ("snapshot-b/examples/greyscale8bit.rs", include_bytes!("greyscale8bit.rs").to_vec()),
        ("snapshot-b/scripts/debug_polyglot.rs", include_bytes!("../scripts/debug_polyglot.rs").to_vec()),
        ("snapshot-b/scripts/check_font.rs", include_bytes!("../scripts/check_font.rs").to_vec()),
        ("snapshot-b/scripts/test_kerning.rs", include_bytes!("../scripts/test_kerning.rs").to_vec()),
        ("snapshot-c/polyglot-architecture.md", include_bytes!("../docs/polyglot-architecture.md").to_vec()),
        ("snapshot-c/polyglot-constraints.md", include_bytes!("../docs/polyglot-constraints.md").to_vec()),
        ("snapshot-c/idat-boundary-analysis.md", include_bytes!("../docs/exploration/idat-boundary-analysis.md").to_vec()),
        ("snapshot-c/variable-width-math.md", include_bytes!("../docs/exploration/variable-width-math.md").to_vec()),
        ("snapshot-c/variable-width-plan.md", include_bytes!("../docs/exploration/variable-width-plan.md").to_vec()),
        ("snapshot-c/wider-images-analysis.md", include_bytes!("../docs/exploration/wider-images-analysis.md").to_vec()),
        ("archive/poc.htm", git_blob("5e2c1c3f950b74db76c34df838363d6a6be2c55a")),
        ("archive/zip.xml", git_blob("9be3db2625f4ebc85ed17078eb22ecacf687071f")),
        ("archive/RYL0051925.json", git_blob("6429e34ed36a6cc563167626d89973d64ab538d9")),
        ("archive/RYL0021220.json", git_blob("8568558f3115ccd2859741b0b7ea1822beb169b5")),
        ("archive/polyglot_v2.rs", git_blob("77e6918a8f3b0128ea404045c323398f464914d5")),
        ("archive/polyglot_v3.rs", git_blob("da359b39faf049612742a212af282893e096bfe9")),
    ];

    let index_map: IndexMap<Vec<u8>, Vec<u8>> = files
        .into_iter()
        .map(|(k, v)| (k.as_bytes().to_vec(), v))
        .collect();
    let files_struct: Files = index_map.into();
    let polyglot = zipng::zipng(&files_struct);
    let png_data = &polyglot;

    let width = u32::from_be_bytes([png_data[16], png_data[17], png_data[18], png_data[19]]) as usize;
    let height = u32::from_be_bytes([png_data[20], png_data[21], png_data[22], png_data[23]]) as usize;

    // Collect IDAT data
    let mut pos = 8;
    let mut idat_data = Vec::new();
    while pos < png_data.len() {
        let chunk_len = u32::from_be_bytes([png_data[pos], png_data[pos+1], png_data[pos+2], png_data[pos+3]]) as usize;
        let chunk_type = &png_data[pos+4..pos+8];
        if chunk_type == b"IDAT" {
            idat_data.extend_from_slice(&png_data[pos+8..pos+8+chunk_len]);
        }
        pos += 4 + 4 + chunk_len + 4;
    }

    // Decompress stored deflate
    let compressed = &idat_data[2..idat_data.len()-4];
    let mut filtered_data = Vec::new();
    let mut dpos = 0;
    while dpos < compressed.len() {
        let header = compressed[dpos];
        let is_final = header & 1 != 0;
        dpos += 1;
        let len = u16::from_le_bytes([compressed[dpos], compressed[dpos+1]]) as usize;
        dpos += 4;
        if dpos + len <= compressed.len() {
            filtered_data.extend_from_slice(&compressed[dpos..dpos+len]);
        }
        dpos += len;
        if is_final { break; }
    }

    // Get filter bytes and reconstruct pixels
    let stride = width + 1;
    let mut pixels = vec![vec![0u8; width]; height];
    let mut filter_bytes = vec![0u8; height];
    for row in 0..height {
        let row_start = row * stride;
        if row_start >= filtered_data.len() { break; }
        filter_bytes[row] = filtered_data[row_start];
        let row_data = &filtered_data[row_start+1..row_start+stride.min(filtered_data.len()-row_start)];
        match filter_bytes[row] {
            0 => {
                for (i, &b) in row_data.iter().enumerate() {
                    if i < width { pixels[row][i] = b; }
                }
            }
            1 => {
                let mut prev = 0u8;
                for (i, &b) in row_data.iter().enumerate() {
                    let actual = b.wrapping_add(prev);
                    if i < width { pixels[row][i] = actual; }
                    prev = actual;
                }
            }
            _ => {
                for (i, &b) in row_data.iter().enumerate() {
                    if i < width { pixels[row][i] = b; }
                }
            }
        }
    }

    // For each row, compute: number of distinct pixel values, count of index-0 pixels,
    // count of 0xFF pixels, and whether it looks like a "background" row.
    // A label text row has 0xFF pixels in the first ~200 columns (small font text).
    // A background/padding row is mostly index 0.
    // A file content row has varied pixel values across the full width.

    // Characterize each row
    let row_chars: Vec<String> = (0..height).map(|r| {
        let zero_count = pixels[r].iter().filter(|&&b| b == 0).count();
        let ff_count = pixels[r].iter().filter(|&&b| b == 0xFF).count();
        let ff_in_first_200 = pixels[r][..200.min(width)].iter().filter(|&&b| b == 0xFF).count();
        let nonzero = width - zero_count;
        let filter = filter_bytes[r];

        // Classify
        if nonzero == 0 {
            format!("BG      f={} z={} nz=0", filter, zero_count)
        } else if ff_in_first_200 > 3 && ff_count < 200 && nonzero < 200 {
            format!("LABEL   f={} nz={:<4} ff={:<4} ff200={}", filter, nonzero, ff_count, ff_in_first_200)
        } else if nonzero > width / 2 {
            format!("CONTENT f={} nz={:<4} ff={:<4}", filter, nonzero, ff_count)
        } else if nonzero <= 20 {
            format!("SPARSE  f={} nz={:<4} ff={:<4}", filter, nonzero, ff_count)
        } else {
            format!("MIXED   f={} nz={:<4} ff={:<4}", filter, nonzero, ff_count)
        }
    }).collect();

    // Find PK headers
    let mut pk_locations: Vec<(String, usize)> = Vec::new();
    for row in 0..height {
        for col in 0..width.saturating_sub(30) {
            if pixels[row][col] == b'P' && pixels[row][col+1] == b'K'
                && pixels[row][col+2] == 0x03 && pixels[row][col+3] == 0x04 {
                let name_len = if col + 27 < width {
                    u16::from_le_bytes([pixels[row][col+26], pixels[row][col+27]]) as usize
                } else { 0 };
                let name_start = col + 30;
                if name_start + name_len <= width {
                    let name = String::from_utf8_lossy(&pixels[row][name_start..name_start+name_len]).to_string();
                    pk_locations.push((name, row));
                }
            }
        }
    }
    pk_locations.sort_by_key(|e| e.1);

    let rows_per_idat = 65535 / stride;
    println!("PNG: {}x{}, rows_per_idat={}", width, height, rows_per_idat);

    // For each PK header, show the 8 rows above it and the PK row itself.
    // Look for labels where the top padding row doesn't look like background.
    println!("\nChecking each file's label area (8 rows above PK header):");
    println!("Looking for labels where top padding row is NOT background.\n");

    for (name, pk_row) in &pk_locations {
        let pk_row = *pk_row;
        // Show rows pk_row-7 through pk_row
        let start = pk_row.saturating_sub(7);

        // Quick check: does this look like a label with bad top padding?
        // Normal label (micro): row-5=toppad(BG), row-4..row-2=text(LABEL), row-1=botpad(mixed), row=PK(CONTENT)
        // Check if row pk_row-5 is NOT background
        let label_top = pk_row.saturating_sub(5);
        let has_label_text = (pk_row.saturating_sub(4)..=pk_row.saturating_sub(2)).any(|r| {
            r < height && row_chars[r].starts_with("LABEL")
        });
        let top_pad_ok = label_top < height && row_chars[label_top].starts_with("BG");

        // Also check if label is split (no LABEL rows in pk_row-4..pk_row-2)
        let is_split = !has_label_text;

        // Flag anomalies
        let anomaly = if has_label_text && !top_pad_ok {
            " <<< TOP PAD NOT BACKGROUND"
        } else {
            ""
        };

        // Only print files with anomalies, or snapshot/deflate files for reference
        if !anomaly.is_empty() || name.contains("deflate") {
            println!("=== {} (PK at row {}) {}{}", name, pk_row,
                if is_split { "[SPLIT]" } else { "" }, anomaly);
            for r in start..=pk_row.min(height - 1) {
                let marker = if r == label_top && has_label_text { " <-- top pad" }
                    else if (pk_row.saturating_sub(4)..=pk_row.saturating_sub(2)).contains(&r) && has_label_text { " <-- text" }
                    else if r == pk_row.saturating_sub(1) && has_label_text { " <-- bot pad" }
                    else if r == pk_row { " <-- PK header" }
                    else { "" };
                println!("  row {:>5}: {}{}", r, row_chars[r], marker);
            }
            println!();
        }
    }
}
