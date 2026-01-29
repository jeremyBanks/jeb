//! Perceptually uniform colormap generator using Oklab color space.
//!
//! Given a sequence of user-specified colors (control points), this module
//! generates a 256-entry RGB palette where the perceptual distance between
//! adjacent entries is uniform throughout.
//!
//! The algorithm:
//! 1. Convert all control points to Oklab.
//! 2. Compute cumulative perceptual arc-length between consecutive control
//!    points (using many small linear steps in Oklab).
//! 3. Redistribute the 256 output samples so that each step covers the same
//!    perceptual distance along the polyline.
//!
//! If only one color is provided, black is prepended and white is appended so
//! the result is a usable gradient rather than a solid fill.

use {
    oklab::{self, Oklab},
    rgb::RGB8,
};

/// Number of sub-steps used when measuring the perceptual arc-length of each
/// segment. More steps ≈ more accurate, but the cost is trivial for 256
/// outputs.
const ARC_STEPS: usize = 1024;

fn rgb_to_ok(c: RGB8) -> Oklab {
    oklab::srgb_to_oklab(c)
}

fn ok_to_rgb(c: Oklab) -> RGB8 {
    oklab::oklab_to_srgb(c)
}

fn lerp(a: Oklab, b: Oklab, t: f32) -> Oklab {
    let inv = 1.0 - t;
    Oklab {
        l: a.l * inv + b.l * t,
        a: a.a * inv + b.a * t,
        b: a.b * inv + b.b * t,
    }
}

fn perceptual_dist(a: Oklab, b: Oklab) -> f32 {
    let dl = a.l - b.l;
    let da = a.a - b.a;
    let db = a.b - b.b;
    (dl * dl + da * da + db * db).sqrt()
}

/// Generate a perceptually uniform 256-color palette from the given control
/// points.
///
/// Returns a `Vec<u8>` of length 768 (256 × 3 bytes, R G B) compatible with
/// the existing `EightBit` palette format.
///
/// # Panics
///
/// Panics if `colors` is empty.
pub fn generate(colors: &[RGB8]) -> Vec<u8> {
    assert!(!colors.is_empty(), "at least one color is required");

    let points: Vec<Oklab> = if colors.len() == 1 {
        vec![
            rgb_to_ok(RGB8::new(0x00, 0x00, 0x00)),
            rgb_to_ok(colors[0]),
            rgb_to_ok(RGB8::new(0xFF, 0xFF, 0xFF)),
        ]
    } else {
        colors.iter().map(|&c| rgb_to_ok(c)).collect()
    };

    let n_segments = points.len() - 1;
    let total_samples = n_segments * ARC_STEPS + 1;

    let mut samples: Vec<Oklab> = Vec::with_capacity(total_samples);
    let mut arc_lengths: Vec<f32> = Vec::with_capacity(total_samples);

    let mut cumulative: f32 = 0.0;
    samples.push(points[0]);
    arc_lengths.push(0.0);

    for seg in 0..n_segments {
        let a = points[seg];
        let b = points[seg + 1];
        for step in 1..=ARC_STEPS {
            let t = step as f32 / ARC_STEPS as f32;
            let interp = lerp(a, b, t);
            let prev = *samples.last().unwrap();
            cumulative += perceptual_dist(prev, interp);
            samples.push(interp);
            arc_lengths.push(cumulative);
        }
    }

    let total_length = cumulative;

    let mut palette = Vec::with_capacity(768);
    let mut cursor: usize = 0;

    for i in 0..256u32 {
        let target = if total_length == 0.0 {
            0.0
        } else {
            (i as f32 / 255.0) * total_length
        };

        while cursor + 1 < samples.len() && arc_lengths[cursor + 1] < target {
            cursor += 1;
        }

        let color = if cursor + 1 >= samples.len() {
            *samples.last().unwrap()
        } else {
            let seg_start = arc_lengths[cursor];
            let seg_end = arc_lengths[cursor + 1];
            let seg_len = seg_end - seg_start;
            if seg_len < 1e-12 {
                samples[cursor]
            } else {
                let t = (target - seg_start) / seg_len;
                lerp(samples[cursor], samples[cursor + 1], t)
            }
        };

        let rgb = ok_to_rgb(color);
        palette.push(rgb.r);
        palette.push(rgb.g);
        palette.push(rgb.b);
    }

    palette
}

/// Generate a palette and return it as a fixed-size array suitable for use as
/// a `static` palette constant.
pub fn generate_array(colors: &[RGB8]) -> [u8; 768] {
    let v = generate(colors);
    let mut arr = [0u8; 768];
    arr.copy_from_slice(&v);
    arr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_color_produces_gradient() {
        let palette = generate(&[RGB8::new(0xFF, 0x00, 0x00)]);
        assert_eq!(palette.len(), 768);
        assert_eq!(&palette[0..3], &[0, 0, 0]);
        assert_eq!(&palette[765..768], &[255, 255, 255]);
    }

    #[test]
    fn two_colors_endpoints_match() {
        let palette = generate(&[RGB8::new(0x00, 0x00, 0xFF), RGB8::new(0xFF, 0xFF, 0x00)]);
        assert_eq!(palette.len(), 768);
        assert_eq!(&palette[0..3], &[0x00, 0x00, 0xFF]);
        assert_eq!(&palette[765..768], &[0xFF, 0xFF, 0x00]);
    }

    #[test]
    fn perceptual_uniformity() {
        let palette = generate(&[
            RGB8::new(0x00, 0x00, 0x00),
            RGB8::new(0xFF, 0x00, 0x00),
            RGB8::new(0xFF, 0xFF, 0xFF),
        ]);

        let mut diffs = Vec::new();
        for i in 0..255 {
            let a = rgb_to_ok(RGB8::new(palette[i * 3], palette[i * 3 + 1], palette[i * 3 + 2]));
            let b = rgb_to_ok(RGB8::new(
                palette[(i + 1) * 3],
                palette[(i + 1) * 3 + 1],
                palette[(i + 1) * 3 + 2],
            ));
            diffs.push(perceptual_dist(a, b));
        }

        let mean: f32 = diffs.iter().sum::<f32>() / diffs.len() as f32;

        // Check that the 90th percentile step size is within 2x of the mean.
        // sRGB 8-bit quantization causes occasional outlier steps at color
        // boundaries, so we measure bulk uniformity rather than worst-case.
        let mut sorted = diffs.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p90 = sorted[(sorted.len() as f32 * 0.9) as usize];
        assert!(
            p90 < mean * 2.0,
            "perceptual uniformity violated: mean={mean}, p90={p90}"
        );
    }
}
