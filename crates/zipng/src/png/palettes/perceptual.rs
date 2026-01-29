//! Perceptually uniform colormap generator using Oklab color space.
//!
//! Given a sequence of user-specified colors (control points), this module
//! generates a 256-entry RGB palette where the perceptual distance between
//! adjacent entries is uniform throughout.
//!
//! ## Algorithm
//!
//! 1. Optionally reorder control points for global coherence (see
//!    [`sort_colors`]).
//! 2. Convert all control points to Oklab.
//! 3. Compute cumulative perceptual arc-length between consecutive control
//!    points (using many small linear steps in Oklab).
//! 4. Redistribute the 256 output samples so that each step covers the same
//!    perceptual distance along the polyline.
//!
//! If only one color is provided, black is prepended and white is appended so
//! the result is a usable gradient rather than a solid fill.
//!
//! ## Sorting / reordering strategies
//!
//! When users provide unordered colors, [`sort_colors`] can reorder them to
//! produce a more coherent gradient. The current implementation uses a shortest
//! Hamiltonian path (open TSP) via brute-force permutation, which finds the
//! ordering that minimizes total perceptual arc-length in Oklab. This is
//! feasible because users typically provide few control points (brute-force is
//! instant for ≤10 colors, i.e. 10! = 3.6M permutations).
//!
//! ### Future options for alternative sort strategies
//!
//! The TSP distance function currently uses unweighted Oklab Euclidean
//! distance. Several variations could be useful:
//!
//! - **Lightness-weighted distance**: `sqrt(w_L * ΔL² + Δa² + Δb²)` with
//!   `w_L > 1` (e.g. 2–4) biases the path to avoid lightness reversals. This
//!   produces colormaps that are more useful for data visualization (readable
//!   in grayscale, accessible to colorblind viewers) at the cost of
//!   potentially longer chromatic jumps. As `w_L → ∞` this converges to a
//!   pure lightness sort.
//!
//! - **Oklch (L, C) distance**: Use the polar form of Oklab and compute
//!   distance in just the lightness-chroma plane, ignoring hue entirely for
//!   ordering. This produces paths monotonic in lightness/saturation while
//!   allowing hue to vary freely — a principled decomposition that avoids
//!   an arbitrary weight parameter.
//!
//! - **Pure lightness sort**: Simply sort by Oklab L. The simplest option
//!   and guarantees monotonic lightness, but ignores chromatic relationships
//!   entirely.
//!
//! - **Principal component ordering**: Project colors onto their first
//!   principal component in Oklab and sort by that. Finds the "natural axis"
//!   of the color set. Works well when colors roughly form a line or arc, but
//!   can produce odd results for clustered color sets.

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

/// Reorder colors to minimize total perceptual arc-length (shortest
/// Hamiltonian path / open TSP in Oklab space).
///
/// For ≤10 colors this uses brute-force over all permutations. For more than
/// 10 it falls back to a greedy nearest-neighbor heuristic starting from the
/// darkest color.
///
/// The returned ordering produces the smoothest possible gradient through the
/// given colors using unweighted Oklab Euclidean distance. See the module docs
/// for alternative weighting strategies.
pub fn sort_colors(colors: &[RGB8]) -> Vec<RGB8> {
    if colors.len() <= 2 {
        return colors.to_vec();
    }

    let ok: Vec<Oklab> = colors.iter().map(|&c| rgb_to_ok(c)).collect();
    let n = ok.len();

    let best_order = if n <= 10 {
        // Brute-force: try all permutations, keep the shortest path.
        let mut indices: Vec<usize> = (0..n).collect();
        let mut best: Option<(f32, Vec<usize>)> = None;
        permutations(&mut indices, n, &mut |perm| {
            let cost = path_cost(perm, &ok);
            if best.as_ref().map_or(true, |(b, _)| cost < *b) {
                best = Some((cost, perm.to_vec()));
            }
        });
        best.unwrap().1
    } else {
        // Greedy nearest-neighbor from the darkest color.
        let mut remaining: Vec<usize> = (0..n).collect();
        let start = remaining
            .iter()
            .copied()
            .min_by(|&a, &b| ok[a].l.partial_cmp(&ok[b].l).unwrap())
            .unwrap();
        remaining.retain(|&i| i != start);
        let mut order = vec![start];
        while !remaining.is_empty() {
            let last = ok[*order.last().unwrap()];
            let (best_idx, _) = remaining
                .iter()
                .enumerate()
                .min_by(|(_, &a), (_, &b)| {
                    perceptual_dist(last, ok[a])
                        .partial_cmp(&perceptual_dist(last, ok[b]))
                        .unwrap()
                })
                .unwrap();
            order.push(remaining.remove(best_idx));
        }
        order
    };

    best_order.iter().map(|&i| colors[i]).collect()
}

/// Total path cost for a given ordering of indices in Oklab space.
fn path_cost(order: &[usize], colors: &[Oklab]) -> f32 {
    order
        .windows(2)
        .map(|w| perceptual_dist(colors[w[0]], colors[w[1]]))
        .sum()
}

/// Heap's algorithm for generating all permutations, calling `f` on each.
fn permutations(arr: &mut Vec<usize>, k: usize, f: &mut impl FnMut(&[usize])) {
    if k == 1 {
        f(arr);
        return;
    }
    permutations(arr, k - 1, f);
    for i in 0..k - 1 {
        if k % 2 == 0 {
            arr.swap(i, k - 1);
        } else {
            arr.swap(0, k - 1);
        }
        permutations(arr, k - 1, f);
    }
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

    #[test]
    fn sort_produces_shorter_path() {
        // Deliberately scrambled order.
        let scrambled = vec![
            RGB8::new(0xFF, 0xFF, 0xFF),
            RGB8::new(0x00, 0x00, 0x00),
            RGB8::new(0xFF, 0x00, 0x00),
            RGB8::new(0x80, 0x80, 0x80),
        ];
        let sorted = sort_colors(&scrambled);

        let cost = |cs: &[RGB8]| -> f32 {
            cs.windows(2)
                .map(|w| perceptual_dist(rgb_to_ok(w[0]), rgb_to_ok(w[1])))
                .sum()
        };

        assert!(
            cost(&sorted) <= cost(&scrambled),
            "sorted path should be no longer than scrambled"
        );
    }
}
