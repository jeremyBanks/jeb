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
//! 3. Interpolate between consecutive control points in Oklch (polar Oklab)
//!    space, which preserves chroma through hue transitions.
//! 4. Compute cumulative perceptual arc-length along the interpolated path.
//! 5. Redistribute the 256 output samples so that each step covers the same
//!    perceptual distance along the path.
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
//! The TSP distance function uses lightness-weighted Oklab distance with
//! `w_L = 4`, biasing the path to avoid lightness reversals for better
//! grayscale readability. Several alternative weightings could also be
//! useful:
//!
//! - **Heavier lightness weighting**: Increasing `w_L` (e.g. 3–4) further
//!   biases toward monotonic lightness at the cost of potentially longer
//!   chromatic jumps. As `w_L → ∞` this converges to a pure lightness sort.
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

/// Interpolate between two Oklab colors in Oklch (polar) space.
///
/// Oklch is the polar form of Oklab: L (lightness), C (chroma = sqrt(a²+b²)),
/// h (hue = atan2(b, a)). Interpolating in polar coordinates keeps chroma high
/// through hue transitions, avoiding the desaturation dip that Cartesian lerp
/// produces when crossing between distant hues.
///
/// Hue is interpolated along the shortest arc. For achromatic colors (C ≈ 0),
/// hue is undefined; we use the other color's hue to avoid discontinuities.
fn lerp(a: Oklab, b: Oklab, t: f32) -> Oklab {
    use std::f32::consts::TAU;

    let inv = 1.0 - t;

    let c_a = (a.a * a.a + a.b * a.b).sqrt();
    let c_b = (b.a * b.a + b.b * b.b).sqrt();

    let h_a = a.b.atan2(a.a);
    let h_b = b.b.atan2(b.a);

    // Interpolate hue along the shortest arc.
    let mut dh = h_b - h_a;
    if dh > std::f32::consts::PI {
        dh -= TAU;
    } else if dh < -std::f32::consts::PI {
        dh += TAU;
    }

    // For achromatic colors, adopt the other color's hue.
    let (h_a, dh) = if c_a < 1e-6 && c_b < 1e-6 {
        (0.0, 0.0)
    } else if c_a < 1e-6 {
        (h_b, 0.0)
    } else if c_b < 1e-6 {
        (h_a, 0.0)
    } else {
        (h_a, dh)
    };

    let l = a.l * inv + b.l * t;
    let c = c_a * inv + c_b * t;
    let h = h_a + dh * t;

    Oklab {
        l,
        a: c * h.cos(),
        b: c * h.sin(),
    }
}

fn perceptual_dist(a: Oklab, b: Oklab) -> f32 {
    let dl = a.l - b.l;
    let da = a.a - b.a;
    let db = a.b - b.b;
    (dl * dl + da * da + db * db).sqrt()
}

/// Like `perceptual_dist` but weights lightness 4× to bias sorting toward
/// monotonic lightness, producing orderings more readable in grayscale.
fn sorting_dist(a: Oklab, b: Oklab) -> f32 {
    let dl = a.l - b.l;
    let da = a.a - b.a;
    let db = a.b - b.b;
    (4.0 * dl * dl + da * da + db * db).sqrt()
}

/// Reorder colors to minimize total perceptual arc-length (shortest
/// Hamiltonian path / open TSP in Oklab space).
///
/// For ≤10 colors this uses brute-force over all permutations. For more than
/// 10 it falls back to a greedy nearest-neighbor heuristic starting from the
/// darkest color.
///
/// The returned ordering produces the smoothest possible gradient through the
/// given colors using lightness-weighted Oklab distance (`w_L = 4`), which
/// biases toward monotonic lightness for better grayscale readability.
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
            if best.as_ref().is_none_or(|(b, _)| cost < *b) {
                best = Some((cost, perm.to_vec()));
            }
        });
        best.unwrap().1
    } else {
        // Greedy nearest-neighbor from the darkest color (suboptimal).
        eprintln!(
            "warning: sort_colors: {n} colors exceeds brute-force limit (10), \
             falling back to greedy nearest-neighbor heuristic"
        );
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
                .min_by(|(_, a), (_, b)| {
                    sorting_dist(last, ok[**a])
                        .partial_cmp(&sorting_dist(last, ok[**b]))
                        .unwrap()
                })
                .unwrap();
            order.push(remaining.remove(best_idx));
        }
        order
    };

    let sorted: Vec<RGB8> = best_order.iter().map(|&i| colors[i]).collect();

    // Preserve the user's intended direction: compare inversions for forward
    // vs reversed result and return whichever is closer to the input order.
    let reversed: Vec<RGB8> = sorted.iter().copied().rev().collect();
    let fwd_inv = count_inversions(colors, &sorted);
    let rev_inv = count_inversions(colors, &reversed);
    if rev_inv < fwd_inv {
        reversed
    } else {
        sorted
    }
}

/// Count the number of pairwise inversions between `reference` order and
/// `candidate` order. Both slices must contain the same elements.
fn count_inversions(reference: &[RGB8], candidate: &[RGB8]) -> usize {
    // Build a position map: for each color in the reference, what index is it at?
    let mut ref_pos: std::collections::HashMap<(u8, u8, u8), usize> =
        std::collections::HashMap::new();
    for (i, c) in reference.iter().enumerate() {
        ref_pos.entry((c.r, c.g, c.b)).or_insert(i);
    }

    let positions: Vec<usize> = candidate
        .iter()
        .map(|c| *ref_pos.get(&(c.r, c.g, c.b)).unwrap_or(&0))
        .collect();

    let mut inversions = 0usize;
    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            if positions[i] > positions[j] {
                inversions += 1;
            }
        }
    }
    inversions
}

/// Total path cost for a given ordering of indices in Oklab space.
fn path_cost(order: &[usize], colors: &[Oklab]) -> f32 {
    order
        .windows(2)
        .map(|w| sorting_dist(colors[w[0]], colors[w[1]]))
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
        if k.is_multiple_of(2) {
            arr.swap(i, k - 1);
        } else {
            arr.swap(0, k - 1);
        }
        permutations(arr, k - 1, f);
    }
}

/// Reassign duplicate RGB8 entries to the nearest unused RGB8 color.
///
/// Processes entries in order: the first occurrence of each RGB8 keeps it,
/// subsequent duplicates are replaced with the perceptually closest unused
/// RGB8 (searching outward from the original in a ±radius cube).
fn deduplicate_palette(entries: &mut [(Oklab, RGB8)]) {
    use std::collections::HashSet;

    let mut used: HashSet<(u8, u8, u8)> = HashSet::new();

    // First pass: mark which entries are unique (first occurrence wins).
    let mut is_dupe = vec![false; entries.len()];
    for (i, &(_, rgb)) in entries.iter().enumerate() {
        let key = (rgb.r, rgb.g, rgb.b);
        if !used.insert(key) {
            is_dupe[i] = true;
        }
    }

    // Second pass: for each duplicate, find the nearest unused RGB8.
    for i in 0..entries.len() {
        if !is_dupe[i] {
            continue;
        }

        let target_ok = entries[i].0;
        let orig = entries[i].1;

        let mut best: Option<(f32, RGB8)> = None;

        // Search outward in expanding radius shells.
        for radius in 1i16..=255 {
            let r_lo = (orig.r as i16 - radius).max(0) as u8;
            let r_hi = (orig.r as i16 + radius).min(255) as u8;
            let g_lo = (orig.g as i16 - radius).max(0) as u8;
            let g_hi = (orig.g as i16 + radius).min(255) as u8;
            let b_lo = (orig.b as i16 - radius).max(0) as u8;
            let b_hi = (orig.b as i16 + radius).min(255) as u8;

            // Only check the shell surface (values at exactly ±radius on at
            // least one axis) to avoid re-checking the interior.
            for r in r_lo..=r_hi {
                for g in g_lo..=g_hi {
                    for b in b_lo..=b_hi {
                        // Skip interior points (already checked at smaller radius).
                        let on_shell = (r as i16 - orig.r as i16).abs() == radius
                            || (g as i16 - orig.g as i16).abs() == radius
                            || (b as i16 - orig.b as i16).abs() == radius;
                        if !on_shell {
                            continue;
                        }

                        let key = (r, g, b);
                        if used.contains(&key) {
                            continue;
                        }

                        let candidate = RGB8::new(r, g, b);
                        let dist = perceptual_dist(target_ok, rgb_to_ok(candidate));
                        if best.as_ref().is_none_or(|(d, _)| dist < *d) {
                            best = Some((dist, candidate));
                        }
                    }
                }
            }

            // If we found any candidate at this radius, it's guaranteed to be
            // closer than anything at a larger radius (in RGB space at least),
            // so we can stop. The perceptual distance might not perfectly
            // correlate with RGB radius, but the error is negligible for small
            // radius values where duplicates occur.
            if best.is_some() {
                break;
            }
        }

        if let Some((_, replacement)) = best {
            let key = (replacement.r, replacement.g, replacement.b);
            used.insert(key);
            entries[i].1 = replacement;
        }
    }
}

/// Deduplicate a raw 768-byte (256 × RGB) palette, ensuring every entry is
/// unique. The Oklab target for each entry is derived from its RGB value.
///
/// Returns a new `Vec<u8>` of length 768 with duplicates replaced by the
/// perceptually nearest unused RGB color.
///
/// # Panics
///
/// Panics if `palette` length is not 768.
pub fn deduplicate_rgb_palette(palette: &[u8]) -> Vec<u8> {
    assert_eq!(palette.len(), 768);

    let mut entries: Vec<(Oklab, RGB8)> = (0..256)
        .map(|i| {
            let rgb = RGB8::new(palette[i * 3], palette[i * 3 + 1], palette[i * 3 + 2]);
            (rgb_to_ok(rgb), rgb)
        })
        .collect();

    deduplicate_palette(&mut entries);

    let mut out = Vec::with_capacity(768);
    for &(_, rgb) in &entries {
        out.push(rgb.r);
        out.push(rgb.g);
        out.push(rgb.b);
    }
    out
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

    // Pass 1: generate ideal (Oklab target, RGB8) pairs allowing duplicates.
    let mut entries: Vec<(Oklab, RGB8)> = Vec::with_capacity(256);
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

        entries.push((color, ok_to_rgb(color)));
    }

    // Pass 2: deduplicate by reassigning duplicate RGB8 values to the nearest
    // unused neighbor. We process in order so earlier entries keep priority.
    deduplicate_palette(&mut entries);

    let mut palette = Vec::with_capacity(768);
    for &(_, rgb) in &entries {
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

/// Parse a hex color string (e.g. "F00" or "FF0000") into RGB8.
/// Supports both 3-digit and 6-digit formats; 3-digit colors are expanded
/// (e.g. "F00" becomes "FF0000").
fn hex_to_rgb(hex: &str) -> RGB8 {
    let hex = hex.trim_start_matches('#');
    let expanded = if hex.len() == 3 {
        // Expand 3-digit hex to 6-digit by doubling each digit
        format!(
            "{}{}{}{}{}{}",
            &hex[0..1],
            &hex[0..1],
            &hex[1..2],
            &hex[1..2],
            &hex[2..3],
            &hex[2..3]
        )
    } else {
        hex.to_string()
    };

    let r = u8::from_str_radix(&expanded[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&expanded[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&expanded[4..6], 16).unwrap_or(0);

    RGB8::new(r, g, b)
}

/// Generate the "frozen" color scheme from hard-coded hex color codes.
/// Returns a 256-color perceptually uniform palette.
/// The control points are automatically reordered using sort_colors for better coherence.
pub fn frozen() -> Vec<u8> {
    let colors = [
        // purple navy gray pale white more purple
        "061B31", "533AFD", /*"50617A",*/ "F6F9FC", "635BFF", "FFFFFF"
        // // yellow orange pink red
        // "FFC01F", "FF6118", "F44BCC", "EA2261",
    ]
    .iter()
    .map(|hex| hex_to_rgb(hex))
    .collect::<Vec<_>>();
    let sorted = sort_colors(&colors);
    let colors = generate(&sorted);
    colors
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
            let a = rgb_to_ok(RGB8::new(
                palette[i * 3],
                palette[i * 3 + 1],
                palette[i * 3 + 2],
            ));
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
    fn sort_preserves_input_direction() {
        // Dark-to-light input
        let dark_to_light = vec![
            RGB8::new(0x00, 0x00, 0x00),
            RGB8::new(0x80, 0x80, 0x80),
            RGB8::new(0xFF, 0xFF, 0xFF),
        ];
        let sorted_dtl = sort_colors(&dark_to_light);
        // Should remain dark-to-light (first element darker than last)
        let first_l = rgb_to_ok(sorted_dtl[0]).l;
        let last_l = rgb_to_ok(sorted_dtl[sorted_dtl.len() - 1]).l;
        assert!(
            first_l <= last_l,
            "dark-to-light input should produce dark-to-light output, got L={first_l} -> L={last_l}"
        );

        // Light-to-dark input
        let light_to_dark = vec![
            RGB8::new(0xFF, 0xFF, 0xFF),
            RGB8::new(0x80, 0x80, 0x80),
            RGB8::new(0x00, 0x00, 0x00),
        ];
        let sorted_ltd = sort_colors(&light_to_dark);
        let first_l2 = rgb_to_ok(sorted_ltd[0]).l;
        let last_l2 = rgb_to_ok(sorted_ltd[sorted_ltd.len() - 1]).l;
        assert!(
            first_l2 >= last_l2,
            "light-to-dark input should produce light-to-dark output, got L={first_l2} -> L={last_l2}"
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
