#!/usr/bin/env -S deno run --allow-read --allow-write --allow-env --allow-run
/**
 * minify-reference.ts — minify z855-reference.ts using terser
 *
 * Strategy: transpile TS→JS via deno (strip types), then hill-climb
 * terser options to find smallest output. Never modifies source file.
 * Output: z855-reference.min.mjs
 */
import { minify } from "npm:terser@^5.36";

const INPUT  = "./z855-reference.ts";
const OUTPUT = "./z855-reference.min.mjs";

// ── Transpile TS → JS via esbuild ────────────────────────────────────────────
import * as esbuild from "npm:esbuild@^0.25";

const tsSource = await Deno.readTextFile(INPUT);
const transformed = await esbuild.transform(tsSource, {
  loader: "ts",
  target: "es2020",
  format: "esm",
  banner: "",
});
await esbuild.stop();
const jsCode = transformed.code;
console.error(`Transpiled: ${jsCode.length} bytes`);

// ── Custom identifier generator ──────────────────────────────────────────────
// Frequency-ordered chars (Jeremy's preference)
const identifierChars = "ZzJEBjebIiPpNnGgFfTtAaOoSsRrHhLlDdCcUuMmWwKkYyVvXxQq_$0123456789";
const firstChars      = "ZzJEBjebIiPpNnGgFfTtAaOoSsRrHhLlDdCcUuMmWwKkYyVvXxQq_$";

function nthIdentifier(n: number): string {
  if (n < firstChars.length) return firstChars[n];
  let remaining = n - firstChars.length;
  let result = "";
  let isFirst = true;
  while (remaining >= 0 || result === "") {
    if (isFirst) {
      result = identifierChars[remaining % identifierChars.length] + result;
      remaining = Math.floor(remaining / identifierChars.length);
      isFirst = false;
    } else {
      if (remaining === 0 && result.length > 0) break;
      result = firstChars[remaining % firstChars.length] + result;
      remaining = Math.floor(remaining / firstChars.length);
    }
    if (remaining === 0 && result.length > 1) break;
  }
  return result;
}

const nth_identifier = { get: nthIdentifier };

// ── Terser option search ─────────────────────────────────────────────────────
const baseCompress = {
  passes: 3,
  hoist_vars: false,
  inline: false,
  properties: false,
  pure_getters: false,
  unsafe: false,
  unsafe_arrows: false,
  unsafe_comps: false,
  dead_code: true,
  drop_debugger: true,
  collapse_vars: true,
};

async function tryMinify(code: string, compress: object, semicolons: boolean): Promise<string> {
  const result = await minify(code, {
    ecma: 2020,
    module: true,
    compress,
    mangle: { toplevel: true, nth_identifier },
    format: { comments: false, semicolons },
  });
  if (!result.code) throw new Error("Terser produced no output");
  return result.code;
}

function packLines(code: string, target: number): string {
  const lines = code.split("\n").map(l => l.trim()).filter(l => l);
  const packed: string[] = [];
  let cur = "";
  let tgt = target;
  for (const line of lines) {
    if (!cur) { cur = line; continue; }
    const merged = cur + ";" + line;
    if (cur.length < 32) { cur = merged; continue; }
    const curPen  = cur.length    < tgt ? tgt - cur.length    : 4*(cur.length - tgt);
    const mrgPen  = merged.length < tgt ? tgt - merged.length : 4*(merged.length - tgt);
    if (mrgPen <= curPen) { cur = merged; }
    else { packed.push(cur); tgt = Math.max(48, Math.min(256, cur.length)); cur = line; }
  }
  if (cur) packed.push(cur);
  return packed.join("\n");
}

const size = (s: string) => new TextEncoder().encode(s).length;

const toggles: Array<[keyof typeof baseCompress, boolean]> = [
  ["hoist_vars",    true],
  ["inline",        true],
  ["properties",    true],
  ["pure_getters",  true],
  ["unsafe",        true],
  ["unsafe_arrows", true],
  ["unsafe_comps",  true],
];

let compress  = { ...baseCompress };
let useSemis  = true;
let curCode   = await tryMinify(jsCode, compress, useSemis);
let curSize   = size(curCode);
console.error(`Baseline: ${curSize} bytes`);

let improved = true;
let round = 0;
while (improved) {
  improved = false;
  round++;
  console.error(`\n=== Round ${round} ===`);

  // Test each compress toggle
  for (const [name, val] of toggles) {
    const tryCompress = { ...compress, [name]: val };
    const tryCode = await tryMinify(jsCode, tryCompress, useSemis);
    const trySize = size(tryCode);
    const delta = trySize - curSize;
    const sign = delta >= 0 ? "+" : "";
    if (trySize < curSize) {
      console.error(`✓ ${name}: ${curSize} → ${trySize} (${sign}${delta})`);
      compress = tryCompress;
      curCode  = tryCode;
      curSize  = trySize;
      improved = true;
      break;
    } else {
      console.error(`✗ ${name}: ${curSize} → ${trySize} (${sign}${delta})`);
    }
  }

  // Test semicolons toggle
  const noSemiCode = await tryMinify(jsCode, compress, !useSemis);
  const noSemiSize = size(noSemiCode);
  if (noSemiSize <= curSize) {
    const delta = noSemiSize - curSize;
    console.error(`✓ semicolons=${!useSemis}: ${curSize} → ${noSemiSize} (${delta})`);
    useSemis = !useSemis;
    curCode  = noSemiCode;
    curSize  = noSemiSize;
    if (noSemiSize < curSize) improved = true;
  } else {
    console.error(`✗ semicolons=${!useSemis}: ${curSize} → ${noSemiSize} (+${noSemiSize - curSize})`);
  }

  // Test line packing
  for (const target of [48, 64, 80, 96, 128]) {
    const packed     = packLines(curCode, target);
    const packedSize = size(packed);
    if (packedSize < curSize) {
      const delta = packedSize - curSize;
      console.error(`✓ pack${target}: ${curSize} → ${packedSize} (${delta})`);
      curCode  = packed;
      curSize  = packedSize;
      improved = true;
      break;
    }
  }

  if (improved) console.error(`→ Restarting (baseline: ${curSize})`);
  else          console.error(`→ No improvement`);
}

// Write output
await Deno.writeTextFile(OUTPUT, curCode);
const origSize = size(jsCode);
const saved    = origSize - curSize;
const pct      = ((saved / origSize) * 100).toFixed(1);
console.error(`\nDone: ${origSize} → ${curSize} bytes (${pct}% smaller)`);
console.error(`Written: ${OUTPUT}`);
console.log(OUTPUT); // stdout: path for piping
