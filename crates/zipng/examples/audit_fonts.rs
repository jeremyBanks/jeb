use std::collections::HashMap;

fn main() {
    let fonts: Vec<(&str, &[u8], &[u8])> = vec![
        ("sixth", include_bytes!("../src/text/sixth.json") as &[u8], include_bytes!("../src/text/sixth.png") as &[u8]),
        ("swiss", include_bytes!("../src/text/swiss.json") as &[u8], include_bytes!("../src/text/swiss.png") as &[u8]),
        ("sky", include_bytes!("../src/text/sky.json") as &[u8], include_bytes!("../src/text/sky.png") as &[u8]),
        ("monte", include_bytes!("../src/text/monte.json") as &[u8], include_bytes!("../src/text/monte.png") as &[u8]),
        ("sugimori", include_bytes!("../src/text/sugimori.json") as &[u8], include_bytes!("../src/text/sugimori.png") as &[u8]),
        ("mini", include_bytes!("../src/text/mini.json") as &[u8], include_bytes!("../src/text/mini.png") as &[u8]),
        ("micro", include_bytes!("../src/text/micro.json") as &[u8], include_bytes!("../src/text/micro.png") as &[u8]),
    ];

    for (name, json_data, png_data) in fonts {
        audit_font(name, json_data, png_data);
    }
}

#[derive(serde::Deserialize)]
struct FontMeta {
    x: usize,
    y: usize,
    dx: usize,
    dy: usize,
    w: usize,
    h: usize,
    #[serde(default)]
    b: usize,
    rows: Vec<String>,
    name: String,
}

fn audit_font(file_name: &str, json_data: &[u8], png_data: &[u8]) {
    let meta: FontMeta = serde_json::from_slice(json_data).unwrap();
    let img = image::load_from_memory(png_data).unwrap().to_luma8();
    let (img_w, img_h) = (img.width() as usize, img.height() as usize);

    println!("=== {} ({}) ===", meta.name, file_name);
    println!("    image: {}x{}, grid: x={} y={} dx={} dy={} w={} h={} b={}",
        img_w, img_h, meta.x, meta.y, meta.dx, meta.dy, meta.w, meta.h, meta.b);

    // Build map of which pixels belong to which glyph cell
    let mut glyph_cells: HashMap<(usize, usize), (usize, usize, char)> = HashMap::new();
    for (row_idx, row_str) in meta.rows.iter().enumerate() {
        for (col_idx, ch) in row_str.chars().enumerate() {
            let gx = meta.x + col_idx * meta.dx;
            let gy = meta.y + row_idx * meta.dy;
            for py in 0..meta.h {
                for px in 0..meta.w {
                    glyph_cells.insert((gx + px, gy + py), (row_idx, col_idx, ch));
                }
            }
        }
    }

    // Find all dark pixels
    let mut dark_outside: Vec<(usize, usize)> = Vec::new();
    let mut glyph_dark_pixels: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

    for y in 0..img_h {
        for x in 0..img_w {
            let pixel = img.get_pixel(x as u32, y as u32).0[0];
            if pixel < 128 {
                if let Some(&(row, col, _)) = glyph_cells.get(&(x, y)) {
                    glyph_dark_pixels.entry((row, col)).or_default().push((x, y));
                } else {
                    dark_outside.push((x, y));
                }
            }
        }
    }

    if !dark_outside.is_empty() {
        println!("  *** {} dark pixels OUTSIDE any glyph cell ***", dark_outside.len());
        // Group by approximate glyph location to identify which glyph is bleeding
        for &(x, y) in dark_outside.iter().take(50) {
            // Find nearest glyph
            let approx_col = if x >= meta.x { (x - meta.x) / meta.dx } else { 0 };
            let approx_row = if y >= meta.y { (y - meta.y) / meta.dy } else { 0 };
            let ch = meta.rows.get(approx_row)
                .and_then(|r| r.chars().nth(approx_col))
                .unwrap_or('?');
            let gx = meta.x + approx_col * meta.dx;
            let gy = meta.y + approx_row * meta.dy;
            println!("    pixel ({},{}) outside cell, nearest glyph '{}' at ({},{}) cell=[{},{}..{},{}]",
                x, y, ch, gx, gy, gx, gy, gx + meta.w - 1, gy + meta.h - 1);
        }
        if dark_outside.len() > 50 {
            println!("    ... and {} more", dark_outside.len() - 50);
        }
    }

    // Check each glyph for edge-touching (clipping)
    let mut clipped = Vec::new();
    for (row_idx, row_str) in meta.rows.iter().enumerate() {
        for (col_idx, ch) in row_str.chars().enumerate() {
            if ch == ' ' { continue; }
            let gx = meta.x + col_idx * meta.dx;
            let gy = meta.y + row_idx * meta.dy;

            if let Some(pixels) = glyph_dark_pixels.get(&(row_idx, col_idx)) {
                let min_x = pixels.iter().map(|p| p.0).min().unwrap();
                let max_x = pixels.iter().map(|p| p.0).max().unwrap();
                let min_y = pixels.iter().map(|p| p.1).min().unwrap();
                let max_y = pixels.iter().map(|p| p.1).max().unwrap();

                let local_min_x = min_x - gx;
                let local_max_x = max_x - gx;
                let local_min_y = min_y - gy;
                let local_max_y = max_y - gy;

                let touches_left = local_min_x == 0;
                let touches_right = local_max_x == meta.w - 1;
                let touches_top = local_min_y == 0;
                let touches_bottom = local_max_y == meta.h - 1;

                // Only flag right/bottom edge touching as potential clipping
                if touches_right || touches_bottom {
                    clipped.push((row_idx, col_idx, ch, local_min_x, local_max_x, local_min_y, local_max_y,
                                  touches_left, touches_right, touches_top, touches_bottom));
                }
            }
        }
    }

    if !clipped.is_empty() {
        println!("  Glyphs touching cell edge (potential clipping):");
        for &(row, col, ch, lx0, lx1, ly0, ly1, tl, tr, tt, tb) in &clipped {
            let edges: Vec<&str> = [
                if tl { Some("L") } else { None },
                if tr { Some("R") } else { None },
                if tt { Some("T") } else { None },
                if tb { Some("B") } else { None },
            ].iter().filter_map(|x| *x).collect();
            println!("    '{}' row={} col={} bbox=[{},{}..{},{}] edges={:?}",
                ch, row, col, lx0, ly0, lx1, ly1, edges);
        }

        // Print ASCII art for clipped glyphs
        println!("  ASCII art of potentially clipped glyphs:");
        for &(row, col, ch, _, _, _, _, _, _, _, _) in &clipped {
            let gx = meta.x + col * meta.dx;
            let gy = meta.y + row * meta.dy;
            // Show expanded area (w+4 x h+4) to see if pixels exist beyond cell
            let expand = 2;
            let sx = if gx >= expand { gx - expand } else { 0 };
            let sy = if gy >= expand { gy - expand } else { 0 };
            let ex = (gx + meta.w + expand).min(img_w);
            let ey = (gy + meta.h + expand).min(img_h);

            println!("    '{}' (expanded view, cell marked with |/-):", ch);
            for py in sy..ey {
                let mut line = String::from("      ");
                for px in sx..ex {
                    let in_cell = px >= gx && px < gx + meta.w && py >= gy && py < gy + meta.h;
                    let pixel = img.get_pixel(px as u32, py as u32).0[0];
                    let dark = pixel < 128;
                    line.push(match (in_cell, dark) {
                        (true, true) => '#',
                        (true, false) => '.',
                        (false, true) => 'X',  // dark pixel OUTSIDE cell!
                        (false, false) => ' ',
                    });
                }
                // Mark cell boundaries
                if py == gy || py == gy + meta.h - 1 {
                    line.push_str(" <--");
                }
                println!("{}", line);
            }
        }
    }

    // Also show any glyph where dark pixels extend into dx-w or dy-h gap
    // by checking expanded area around each glyph
    let mut bleeding = Vec::new();
    for (row_idx, row_str) in meta.rows.iter().enumerate() {
        for (col_idx, ch) in row_str.chars().enumerate() {
            if ch == ' ' { continue; }
            let gx = meta.x + col_idx * meta.dx;
            let gy = meta.y + row_idx * meta.dy;

            // Check gap area to the right (w..dx) and below (h..dy)
            let mut right_bleed = false;
            let mut bottom_bleed = false;
            for py in 0..meta.dy {
                for px in 0..meta.dx {
                    let ax = gx + px;
                    let ay = gy + py;
                    if ax >= img_w || ay >= img_h { continue; }
                    let pixel = img.get_pixel(ax as u32, ay as u32).0[0];
                    if pixel < 128 {
                        if px >= meta.w && py < meta.h { right_bleed = true; }
                        if py >= meta.h && px < meta.w { bottom_bleed = true; }
                    }
                }
            }
            if right_bleed || bottom_bleed {
                bleeding.push((row_idx, col_idx, ch, right_bleed, bottom_bleed));
            }
        }
    }

    if !bleeding.is_empty() {
        println!("  Glyphs bleeding into gap area (dx-w or dy-h space):");
        for &(row, col, ch, rb, bb) in &bleeding {
            let mut bleed_str = String::new();
            if rb { bleed_str.push_str("RIGHT "); }
            if bb { bleed_str.push_str("BOTTOM "); }
            println!("    '{}' row={} col={} bleeds: {}", ch, row, col, bleed_str);

            // Show expanded view
            let gx = meta.x + col * meta.dx;
            let gy = meta.y + row * meta.dy;
            println!("    '{}' full cell area (dx x dy):", ch);
            for py in 0..meta.dy.min(img_h - gy) {
                let mut line = String::from("      ");
                for px in 0..meta.dx.min(img_w - gx) {
                    let in_cell = px < meta.w && py < meta.h;
                    let pixel = img.get_pixel((gx + px) as u32, (gy + py) as u32).0[0];
                    let dark = pixel < 128;
                    line.push(match (in_cell, dark) {
                        (true, true) => '#',
                        (true, false) => '.',
                        (false, true) => 'X',
                        (false, false) => ' ',
                    });
                }
                println!("{}", line);
            }
        }
    }

    if dark_outside.is_empty() && clipped.is_empty() && bleeding.is_empty() {
        println!("  All glyphs OK - no clipping or bleeding detected.");
    }
    println!();
}
