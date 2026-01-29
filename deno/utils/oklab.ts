/**
 * Perceptually uniform color map generation using the Oklab color space.
 *
 * Given an ordered list of color stops, this module generates a color map
 * where the perceptual distance between adjacent output colors is uniform.
 */

// ---- Color types ----

export interface RGB {
  r: number; // 0-1
  g: number; // 0-1
  b: number; // 0-1
}

export interface Lab {
  L: number; // 0-1 (perceptual lightness)
  a: number; // roughly -0.4 to 0.4
  b: number; // roughly -0.4 to 0.4
}

// ---- sRGB <-> Linear RGB ----

function srgbToLinear(c: number): number {
  return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
}

function linearToSrgb(c: number): number {
  return c <= 0.0031308
    ? 12.92 * c
    : 1.055 * Math.pow(c, 1 / 2.4) - 0.055;
}

// ---- Linear RGB <-> Oklab ----
// Reference: https://bottosson.github.io/posts/oklab/

export function rgbToOklab(rgb: RGB): Lab {
  const r = srgbToLinear(rgb.r);
  const g = srgbToLinear(rgb.g);
  const b = srgbToLinear(rgb.b);

  const l_ = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
  const m_ = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
  const s_ = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

  const l = Math.cbrt(l_);
  const m = Math.cbrt(m_);
  const s = Math.cbrt(s_);

  return {
    L: 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
    a: 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
    b: 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
  };
}

export function oklabToRgb(lab: Lab): RGB {
  const l_ = lab.L + 0.3963377774 * lab.a + 0.2158037573 * lab.b;
  const m_ = lab.L - 0.1055613458 * lab.a - 0.0638541728 * lab.b;
  const s_ = lab.L - 0.0894841775 * lab.a - 1.2914855480 * lab.b;

  const l = l_ * l_ * l_;
  const m = m_ * m_ * m_;
  const s = s_ * s_ * s_;

  return {
    r: clamp01(
      linearToSrgb(+4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s),
    ),
    g: clamp01(
      linearToSrgb(-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s),
    ),
    b: clamp01(
      linearToSrgb(-0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s),
    ),
  };
}

// ---- Perceptual distance in Oklab ----

export function oklabDistance(a: Lab, b: Lab): number {
  const dL = a.L - b.L;
  const da = a.a - b.a;
  const db = a.b - b.b;
  return Math.sqrt(dL * dL + da * da + db * db);
}

// ---- Interpolation in Oklab ----

function lerpLab(a: Lab, b: Lab, t: number): Lab {
  return {
    L: a.L + (b.L - a.L) * t,
    a: a.a + (b.a - a.a) * t,
    b: a.b + (b.b - a.b) * t,
  };
}

// ---- Color map generation ----

/**
 * Given ordered color stops, generates a perceptually uniform color map
 * with `steps` total colors.
 *
 * The input colors define the path through color space. The output
 * resamples that path at perceptually equal intervals.
 *
 * @param stops - Ordered array of at least 2 RGB colors (values 0-1)
 * @param steps - Number of output colors (default 256)
 * @returns Array of RGB colors with perceptually uniform spacing
 */
export function perceptualColorMap(stops: RGB[], steps = 256): RGB[] {
  if (stops.length < 2) {
    throw new Error("Need at least 2 color stops");
  }

  const labStops = stops.map(rgbToOklab);

  // Sample the piecewise-linear path at high resolution to measure arc length
  const sampleCount = 4096;
  const samples: Lab[] = [];
  const cumDist: number[] = [0];

  for (let i = 0; i <= sampleCount; i++) {
    const t = i / sampleCount;
    // Map t to the piecewise-linear path through stops
    const lab = samplePath(labStops, t);
    samples.push(lab);
    if (i > 0) {
      cumDist.push(cumDist[i - 1] + oklabDistance(samples[i - 1], lab));
    }
  }

  const totalDist = cumDist[cumDist.length - 1];

  // Resample at perceptually equal intervals
  const result: RGB[] = [];
  for (let i = 0; i < steps; i++) {
    const targetDist = (i / (steps - 1)) * totalDist;
    // Binary search for the sample index
    let lo = 0, hi = sampleCount;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (cumDist[mid] < targetDist) lo = mid + 1;
      else hi = mid;
    }
    if (lo === 0) {
      result.push(oklabToRgb(samples[0]));
    } else {
      // Linearly interpolate between samples[lo-1] and samples[lo]
      const segLen = cumDist[lo] - cumDist[lo - 1];
      const frac = segLen > 0 ? (targetDist - cumDist[lo - 1]) / segLen : 0;
      result.push(oklabToRgb(lerpLab(samples[lo - 1], samples[lo], frac)));
    }
  }

  return result;
}

/** Sample the piecewise-linear path at parameter t in [0,1]. */
function samplePath(stops: Lab[], t: number): Lab {
  const n = stops.length - 1;
  const scaled = t * n;
  const i = Math.min(Math.floor(scaled), n - 1);
  const frac = scaled - i;
  return lerpLab(stops[i], stops[i + 1], frac);
}

// ---- Hex color parsing/formatting ----

/** Parse a CSS hex color (#rgb, #rrggbb, #rrggbbaa) to RGB. */
export function parseHex(hex: string): RGB {
  hex = hex.replace(/^#/, "");
  if (hex.length === 3) {
    hex = hex[0] + hex[0] + hex[1] + hex[1] + hex[2] + hex[2];
  } else if (hex.length === 4) {
    hex = hex[0] + hex[0] + hex[1] + hex[1] + hex[2] + hex[2]; // ignore alpha
  }
  const n = parseInt(hex.slice(0, 6), 16);
  return {
    r: ((n >> 16) & 0xff) / 255,
    g: ((n >> 8) & 0xff) / 255,
    b: (n & 0xff) / 255,
  };
}

/** Format RGB (0-1) as a CSS hex color. */
export function toHex(rgb: RGB): string {
  const r = Math.round(rgb.r * 255);
  const g = Math.round(rgb.g * 255);
  const b = Math.round(rgb.b * 255);
  return (
    "#" +
    r.toString(16).padStart(2, "0") +
    g.toString(16).padStart(2, "0") +
    b.toString(16).padStart(2, "0")
  );
}

/** Named CSS colors (subset of common ones). */
const NAMED_COLORS: Record<string, string> = {
  white: "#ffffff",
  black: "#000000",
  red: "#ff0000",
  green: "#008000",
  blue: "#0000ff",
  yellow: "#ffff00",
  cyan: "#00ffff",
  magenta: "#ff00ff",
  orange: "#ffa500",
  purple: "#800080",
  pink: "#ffc0cb",
  gray: "#808080",
  grey: "#808080",
  lime: "#00ff00",
  navy: "#000080",
  teal: "#008080",
  maroon: "#800000",
  olive: "#808000",
  aqua: "#00ffff",
  silver: "#c0c0c0",
  coral: "#ff7f50",
  salmon: "#fa8072",
  gold: "#ffd700",
  indigo: "#4b0082",
  violet: "#ee82ee",
  turquoise: "#40e0d0",
  tan: "#d2b48c",
  khaki: "#f0e68c",
  crimson: "#dc143c",
  tomato: "#ff6347",
  chocolate: "#d2691e",
  sienna: "#a0522d",
  peru: "#cd853f",
  lavender: "#e6e6fa",
  plum: "#dda0dd",
  orchid: "#da70d6",
  beige: "#f5f5dc",
  ivory: "#fffff0",
  linen: "#faf0e6",
  snow: "#fffafa",
  honeydew: "#f0fff0",
  azure: "#f0ffff",
  mintcream: "#f5fffa",
  cornflowerblue: "#6495ed",
  dodgerblue: "#1e90ff",
  royalblue: "#4169e1",
  steelblue: "#4682b4",
  skyblue: "#87ceeb",
  midnightblue: "#191970",
  darkblue: "#00008b",
  darkgreen: "#006400",
  darkred: "#8b0000",
  darkorange: "#ff8c00",
  darkviolet: "#9400d3",
  deeppink: "#ff1493",
  firebrick: "#b22222",
  forestgreen: "#228b22",
  hotpink: "#ff69b4",
  lemonchiffon: "#fffacd",
  lightblue: "#add8e6",
  lightcoral: "#f08080",
  lightgreen: "#90ee90",
  lightyellow: "#ffffe0",
  mediumblue: "#0000cd",
  seagreen: "#2e8b57",
  slateblue: "#6a5acd",
  springgreen: "#00ff7f",
};

/** Parse a color string: hex (#rgb/#rrggbb) or named CSS color. */
export function parseColor(input: string): RGB {
  input = input.trim().toLowerCase();
  if (input.startsWith("#")) return parseHex(input);
  const named = NAMED_COLORS[input];
  if (named) return parseHex(named);
  // Try rgb(r, g, b) format
  const rgbMatch = input.match(
    /^rgb\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)$/,
  );
  if (rgbMatch) {
    return {
      r: parseInt(rgbMatch[1]) / 255,
      g: parseInt(rgbMatch[2]) / 255,
      b: parseInt(rgbMatch[3]) / 255,
    };
  }
  throw new Error(`Unknown color: "${input}"`);
}

/**
 * High-level API: generate a perceptually uniform color map from color strings.
 *
 * @example
 * ```ts
 * const map = generateColorMap(["white", "#3498db", "black"], 16);
 * // Returns 16 hex color strings with perceptually uniform spacing
 * ```
 */
export function generateColorMap(
  colors: string[],
  steps = 256,
): string[] {
  const stops = colors.map(parseColor);
  return perceptualColorMap(stops, steps).map(toHex);
}

// ---- Utilities ----

function clamp01(x: number): number {
  return x < 0 ? 0 : x > 1 ? 1 : x;
}
