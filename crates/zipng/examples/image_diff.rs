use image::GenericImageView;
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

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: image_diff <old.png> <new.png>");
        return ExitCode::from(2);
    }

    let old_img = image::open(&args[1]).unwrap_or_else(|e| {
        eprintln!("failed to open {}: {}", &args[1], e);
        std::process::exit(2);
    });
    let new_img = image::open(&args[2]).unwrap_or_else(|e| {
        eprintln!("failed to open {}: {}", &args[2], e);
        std::process::exit(2);
    });

    let (old_w, old_h) = old_img.dimensions();
    let (new_w, new_h) = new_img.dimensions();

    let (name_a, name_b) = strip_common_prefix(&args[1], &args[2]);

    println!("{}: {}×{}", name_a, old_w, old_h);
    println!("{}: {}×{}", name_b, new_w, new_h);

    let w = old_w.min(new_w);
    let h = old_h.min(new_h);

    if old_w != new_w {
        println!("width: {} vs {}, cropping to {}", old_w, new_w, w);
    }
    if old_h != new_h {
        println!("height: {} vs {}, cropping to {}", old_h, new_h, h);
    }

    let old_rgba = old_img.to_rgba8();
    let new_rgba = new_img.to_rgba8();

    let total = (w as u64) * (h as u64);

    // Per-channel stats
    struct ChannelStats {
        diff_count: u64,
        abs_sum: f64,
        signed_sum: f64,
        // Track min/max across both images to detect uniform identical channels
        min_val: u8,
        max_val: u8,
    }

    let mut ch_stats = [
        ChannelStats { diff_count: 0, abs_sum: 0.0, signed_sum: 0.0, min_val: 255, max_val: 0 },
        ChannelStats { diff_count: 0, abs_sum: 0.0, signed_sum: 0.0, min_val: 255, max_val: 0 },
        ChannelStats { diff_count: 0, abs_sum: 0.0, signed_sum: 0.0, min_val: 255, max_val: 0 },
        ChannelStats { diff_count: 0, abs_sum: 0.0, signed_sum: 0.0, min_val: 255, max_val: 0 },
    ];

    // Bounding box of changed pixels
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x: u32 = 0;
    let mut max_y: u32 = 0;

    // First pass: per-channel stats and detect alpha uniformity
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

    // Both fully opaque = omit alpha from output and whole-pixel calculations
    let both_opaque = ch_stats[3].min_val == 255 && ch_stats[3].max_val == 255;
    let num_channels: usize = if both_opaque { 3 } else { 4 };

    // Any channel differs at all?
    let any_diff = ch_stats[..num_channels].iter().any(|s| s.diff_count > 0);

    if !any_diff {
        println!("\nimages are identical ({} pixels compared)", total);
        return ExitCode::SUCCESS;
    }

    // Second pass: whole-pixel stats (excluding alpha if both opaque)
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
