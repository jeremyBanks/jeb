use std::io::BufReader;
use std::process::ExitCode;

fn strip_common_prefix<'a>(a: &'a str, b: &'a str) -> (&'a str, &'a str) {
    let common_len = a
        .bytes()
        .zip(b.bytes())
        .take_while(|(x, y)| x == y)
        .count();
    // Back up to last '/' within the common prefix
    let cut = a[..common_len].rfind('/').map(|i| i + 1).unwrap_or(0);
    (&a[cut..], &b[cut..])
}

struct PngInfo {
    width: u32,
    height: u32,
    color_type: png::ColorType,
    bit_depth: png::BitDepth,
    file_size: u64,
    /// Raw index bytes if indexed color, None otherwise
    raw_indices: Option<Vec<u8>>,
}

fn read_png_info(path: &str) -> PngInfo {
    let file_size = std::fs::metadata(path)
        .unwrap_or_else(|e| {
            eprintln!("failed to stat {}: {}", path, e);
            std::process::exit(2);
        })
        .len();

    let file = std::fs::File::open(path).unwrap_or_else(|e| {
        eprintln!("failed to open {}: {}", path, e);
        std::process::exit(2);
    });

    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder.read_info().unwrap_or_else(|e| {
        eprintln!("failed to read PNG info from {}: {}", path, e);
        std::process::exit(2);
    });

    let info = reader.info();
    let width = info.width;
    let height = info.height;
    let color_type = info.color_type;
    let bit_depth = info.bit_depth;

    let raw_indices = if color_type == png::ColorType::Indexed {
        let mut buf = vec![0u8; reader.output_buffer_size().unwrap()];
        let output_info = reader.next_frame(&mut buf).unwrap_or_else(|e| {
            eprintln!("failed to decode {}: {}", path, e);
            std::process::exit(2);
        });
        buf.truncate(output_info.buffer_size());
        Some(buf)
    } else {
        None
    };

    PngInfo {
        width,
        height,
        color_type,
        bit_depth,
        file_size,
        raw_indices,
    }
}

fn color_type_name(ct: png::ColorType) -> &'static str {
    match ct {
        png::ColorType::Grayscale => "Grayscale",
        png::ColorType::Rgb => "RGB",
        png::ColorType::Indexed => "Indexed",
        png::ColorType::GrayscaleAlpha => "GrayscaleAlpha",
        png::ColorType::Rgba => "RGBA",
    }
}

fn bit_depth_bits(bd: png::BitDepth) -> u8 {
    match bd {
        png::BitDepth::One => 1,
        png::BitDepth::Two => 2,
        png::BitDepth::Four => 4,
        png::BitDepth::Eight => 8,
        png::BitDepth::Sixteen => 16,
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: image_diff <old.png> <new.png>");
        return ExitCode::from(2);
    }

    let old_info = read_png_info(&args[1]);
    let new_info = read_png_info(&args[2]);

    let (name_a, name_b) = strip_common_prefix(&args[1], &args[2]);

    println!(
        "{}: {}×{}, {} {}-bit, {} bytes",
        name_a, old_info.width, old_info.height,
        color_type_name(old_info.color_type),
        bit_depth_bits(old_info.bit_depth),
        old_info.file_size
    );
    println!(
        "{}: {}×{}, {} {}-bit, {} bytes",
        name_b, new_info.width, new_info.height,
        color_type_name(new_info.color_type),
        bit_depth_bits(new_info.bit_depth),
        new_info.file_size
    );

    let w = old_info.width.min(new_info.width);
    let h = old_info.height.min(new_info.height);

    if old_info.width != new_info.width {
        println!("width: {} vs {}, cropping to {}", old_info.width, new_info.width, w);
    }
    if old_info.height != new_info.height {
        println!("height: {} vs {}, cropping to {}", old_info.height, new_info.height, h);
    }

    let both_indexed = old_info.raw_indices.is_some()
        && new_info.raw_indices.is_some()
        && old_info.bit_depth == new_info.bit_depth;

    if old_info.color_type != new_info.color_type {
        println!(
            "note: color type mismatch ({} vs {}), comparing as RGBA",
            color_type_name(old_info.color_type),
            color_type_name(new_info.color_type)
        );
    }

    if both_indexed {
        compare_indexed(&args[1], &args[2], name_a, name_b, &old_info, &new_info, w, h)
    } else {
        compare_rgba(&args[1], &args[2], w, h)
    }
}

fn compare_indexed(
    _old_path: &str,
    _new_path: &str,
    _name_a: &str,
    _name_b: &str,
    old_info: &PngInfo,
    new_info: &PngInfo,
    w: u32,
    h: u32,
) -> ExitCode {
    let old_data = old_info.raw_indices.as_ref().unwrap();
    let new_data = new_info.raw_indices.as_ref().unwrap();

    let old_stride = old_info.width as usize;
    let new_stride = new_info.width as usize;

    let total = (w as u64) * (h as u64);
    let mut diff_count: u64 = 0;
    let mut abs_sum: f64 = 0.0;
    let mut signed_sum: f64 = 0.0;

    let mut min_x = w;
    let mut min_y = h;
    let mut max_x: u32 = 0;
    let mut max_y: u32 = 0;

    for y in 0..h {
        for x in 0..w {
            let ov = old_data[y as usize * old_stride + x as usize];
            let nv = new_data[y as usize * new_stride + x as usize];
            let d = nv as i16 - ov as i16;
            if d != 0 {
                diff_count += 1;
                abs_sum += d.unsigned_abs() as f64;
                signed_sum += d as f64;
                if x < min_x { min_x = x; }
                if x > max_x { max_x = x; }
                if y < min_y { min_y = y; }
                if y > max_y { max_y = y; }
            }
        }
    }

    if diff_count == 0 {
        println!("\nimages are identical ({} pixels compared)", total);
        return ExitCode::SUCCESS;
    }

    println!("\n{} pixels compared:", total);
    println!();

    let pct = 100.0 * diff_count as f64 / total as f64;
    let mean_abs = abs_sum / diff_count as f64;
    let mean_signed = signed_sum / diff_count as f64;
    println!(
        "index: {} differ ({:.2}%), mean |d|={:.2}, mean d={:+.2}",
        diff_count, pct, mean_abs, mean_signed
    );

    println!();
    println!(
        "changed pixel bounds: ({}, {}) to ({}, {}), spanning {}×{}",
        min_x, min_y, max_x, max_y,
        max_x - min_x + 1, max_y - min_y + 1
    );

    ExitCode::from(1)
}

fn compare_rgba(old_path: &str, new_path: &str, w: u32, h: u32) -> ExitCode {
    let old_img = image::open(old_path).unwrap_or_else(|e| {
        eprintln!("failed to open {}: {}", old_path, e);
        std::process::exit(2);
    });
    let new_img = image::open(new_path).unwrap_or_else(|e| {
        eprintln!("failed to open {}: {}", new_path, e);
        std::process::exit(2);
    });

    let old_rgba = old_img.to_rgba8();
    let new_rgba = new_img.to_rgba8();

    let total = (w as u64) * (h as u64);

    struct ChannelStats {
        diff_count: u64,
        abs_sum: f64,
        signed_sum: f64,
        min_val: u8,
        max_val: u8,
    }

    let mut ch_stats = [
        ChannelStats { diff_count: 0, abs_sum: 0.0, signed_sum: 0.0, min_val: 255, max_val: 0 },
        ChannelStats { diff_count: 0, abs_sum: 0.0, signed_sum: 0.0, min_val: 255, max_val: 0 },
        ChannelStats { diff_count: 0, abs_sum: 0.0, signed_sum: 0.0, min_val: 255, max_val: 0 },
        ChannelStats { diff_count: 0, abs_sum: 0.0, signed_sum: 0.0, min_val: 255, max_val: 0 },
    ];

    let mut min_x = w;
    let mut min_y = h;
    let mut max_x: u32 = 0;
    let mut max_y: u32 = 0;

    for y in 0..h {
        for x in 0..w {
            let op = old_rgba.get_pixel(x, y).0;
            let np = new_rgba.get_pixel(x, y).0;

            let mut pixel_differs = false;

            for c in 0..4 {
                let ov = op[c];
                let nv = np[c];
                let s = &mut ch_stats[c];
                if ov < s.min_val { s.min_val = ov; }
                if ov > s.max_val { s.max_val = ov; }
                if nv < s.min_val { s.min_val = nv; }
                if nv > s.max_val { s.max_val = nv; }

                let d = nv as i16 - ov as i16;
                if d != 0 {
                    pixel_differs = true;
                    s.diff_count += 1;
                    s.abs_sum += d.unsigned_abs() as f64;
                    s.signed_sum += d as f64;
                }
            }

            if pixel_differs {
                if x < min_x { min_x = x; }
                if x > max_x { max_x = x; }
                if y < min_y { min_y = y; }
                if y > max_y { max_y = y; }
            }
        }
    }

    let both_opaque = ch_stats[3].min_val == 255 && ch_stats[3].max_val == 255;
    let num_channels: usize = if both_opaque { 3 } else { 4 };

    let any_diff = ch_stats[..num_channels].iter().any(|s| s.diff_count > 0);

    if !any_diff {
        println!("\nimages are identical ({} pixels compared)", total);
        return ExitCode::SUCCESS;
    }

    let mut px_diff_count: u64 = 0;
    let mut px_abs_sum: f64 = 0.0;
    let mut px_signed_sum = vec![0.0f64; num_channels];
    let mut px_euclid_sum: f64 = 0.0;

    for y in 0..h {
        for x in 0..w {
            let op = old_rgba.get_pixel(x, y).0;
            let np = new_rgba.get_pixel(x, y).0;

            let mut pixel_differs = false;
            let mut abs_total = 0.0f64;
            let mut euclid_sq = 0.0f64;

            for c in 0..num_channels {
                let d = np[c] as f64 - op[c] as f64;
                if d != 0.0 {
                    pixel_differs = true;
                }
                abs_total += d.abs();
                px_signed_sum[c] += d;
                euclid_sq += d * d;
            }

            if pixel_differs {
                px_diff_count += 1;
                px_abs_sum += abs_total;
                px_euclid_sum += euclid_sq.sqrt();
            }
        }
    }

    println!("\n{} pixels compared:", total);
    println!();

    let all_ch_names = ["R", "G", "B", "A"];
    for i in 0..num_channels {
        let name = all_ch_names[i];
        let s = &ch_stats[i];
        if s.diff_count == 0 {
            if s.min_val == s.max_val {
                println!("{}: identical (all 0x{:02X})", name, s.min_val);
            } else {
                println!("{}: identical (range 0x{:02X}..0x{:02X})", name, s.min_val, s.max_val);
            }
        } else {
            let pct = 100.0 * s.diff_count as f64 / total as f64;
            let mean_abs = s.abs_sum / s.diff_count as f64;
            let mean_signed = s.signed_sum / s.diff_count as f64;
            println!(
                "{}: {} differ ({:.2}%), mean |d|={:.2}, mean d={:+.2}",
                name, s.diff_count, pct, mean_abs, mean_signed
            );
        }
    }

    println!();
    let pct = 100.0 * px_diff_count as f64 / total as f64;
    let mean_abs = px_abs_sum / px_diff_count as f64;
    let mean_euclid = px_euclid_sum / px_diff_count as f64;
    let mean_signed: Vec<String> = px_signed_sum
        .iter()
        .zip(all_ch_names.iter())
        .map(|(s, n)| format!("{}={:+.2}", n, s / px_diff_count as f64))
        .collect();

    println!(
        "pixel: {} differ ({:.2}%), mean |d|={:.2}, mean euclid={:.2}, mean signed=[{}]",
        px_diff_count,
        pct,
        mean_abs,
        mean_euclid,
        mean_signed.join(", ")
    );

    println!();
    println!(
        "changed pixel bounds: ({}, {}) to ({}, {}), spanning {}×{}",
        min_x, min_y, max_x, max_y,
        max_x - min_x + 1, max_y - min_y + 1
    );

    ExitCode::from(1)
}
