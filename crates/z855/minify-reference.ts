#!/usr/bin/env -S deno run --allow-read --allow-write --allow-env --allow-run
/**
 * minify-reference.ts — minify z855-reference.ts using terser
 *
 * Produces two outputs:
 *   z855-decoder.min.mjs   — decoder-only (encoder sections stripped, terser tree-shakes rest)
 *   z855-reference.min.mjs — full encoder+decoder
 *
 * Then builds z855.zipng in a temp dir containing:
 *   d — the minified decoder (plain)
 *   z — the minified full script, passed through encode-lines (needs d to unlock)
 */
import { minify } from "npm:terser@^5.36";
import * as esbuild from "npm:esbuild@^0.25";

const INPUT       = "./z855-reference.ts";
const OUTPUT_FULL = "./z855-reference.min.mjs";
const OUTPUT_DEC  = "./z855-decoder.min.mjs";
const ZIPNG_CLI   = "/Users/matte/.openclaw/workspace/zipng/zipng-cli.ts";

// ── Strip encoder sections ────────────────────────────────────────────────────
function stripEncoder(source: string): string {
  const lines = source.split("\n");
  const out: string[] = [];
  let stripping = false;
  for (const line of lines) {
    if (line.includes("// @strip-encoder-start")) { stripping = true; continue; }
    if (line.includes("// @strip-encoder-end"))   { stripping = false; continue; }
    if (!stripping) out.push(line);
  }
  return out.join("\n");
}

// ── Transpile TS → JS via esbuild ────────────────────────────────────────────
async function transpile(source: string): Promise<string> {
  const result = await esbuild.transform(source, {
    loader: "ts", target: "esnext", format: "esm", banner: "",
  });
  return result.code;
}

const tsSource     = await Deno.readTextFile(INPUT);
const tsDecoder    = stripEncoder(tsSource);
const jsFull       = await transpile(tsSource);
const jsDecoder    = await transpile(tsDecoder);
await esbuild.stop();
console.error(`Transpiled full: ${jsFull.length}B  decoder: ${jsDecoder.length}B`);

// ── Custom identifier generator ──────────────────────────────────────────────
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

// ── Terser helpers ───────────────────────────────────────────────────────────
const baseCompress = {
  passes: 3, hoist_vars: false, inline: false, properties: false,
  pure_getters: false, unsafe: false, unsafe_arrows: false, unsafe_comps: false,
  dead_code: true, drop_debugger: true, collapse_vars: true,
};

async function tryMinify(code: string, compress: object, semicolons: boolean): Promise<string> {
  const result = await minify(code, {
    ecma: 2022, module: true, compress,
    mangle: { toplevel: true, nth_identifier },
    format: { comments: false, semicolons },
  });
  if (!result.code) throw new Error("Terser produced no output");
  return result.code;
}

function packLines(code: string, target: number): string {
  const allLines = code.split("\n").map(l => l.trim()).filter(l => l);
  const prefix: string[] = [];
  const lines: string[] = [];
  for (const l of allLines) {
    if (l.startsWith("#!") && prefix.length === 0 && lines.length === 0) prefix.push(l);
    else lines.push(l);
  }
  const packed: string[] = [];
  let cur = "", tgt = target;
  for (const line of lines) {
    if (!cur) { cur = line; continue; }
    const merged = cur + ";" + line;
    if (cur.length < 32) { cur = merged; continue; }
    const curPen = cur.length < tgt ? tgt - cur.length : 4*(cur.length - tgt);
    const mrgPen = merged.length < tgt ? tgt - merged.length : 4*(merged.length - tgt);
    if (mrgPen <= curPen) cur = merged;
    else { packed.push(cur); tgt = Math.max(48, Math.min(256, cur.length)); cur = line; }
  }
  if (cur) packed.push(cur);
  return [...prefix, ...packed].join("\n");
}

const size = (s: string) => new TextEncoder().encode(s).length;

const toggles: Array<[keyof typeof baseCompress, boolean]> = [
  ["hoist_vars", true], ["inline", true], ["properties", true],
  ["pure_getters", true], ["unsafe", true], ["unsafe_arrows", true], ["unsafe_comps", true],
];

// ── Hill-climb minification ──────────────────────────────────────────────────
async function hillClimb(jsCode: string, label: string): Promise<string> {
  let compress  = { ...baseCompress };
  let useSemis  = true;
  let curCode   = await tryMinify(jsCode, compress, useSemis);
  let curSize   = size(curCode);
  console.error(`\n[${label}] Baseline: ${curSize} bytes`);

  let improved = true, round = 0;
  while (improved) {
    improved = false; round++;
    for (const [name, val] of toggles) {
      const tryCompress = { ...compress, [name]: val };
      const tryCode = await tryMinify(jsCode, tryCompress, useSemis);
      const trySize = size(tryCode);
      if (trySize < curSize) {
        console.error(`[${label}] ✓ ${name}: ${curSize} → ${trySize}`);
        compress = tryCompress; curCode = tryCode; curSize = trySize; improved = true; break;
      }
    }
    const noSemiCode = await tryMinify(jsCode, compress, !useSemis);
    const noSemiSize = size(noSemiCode);
    if (noSemiSize <= curSize) {
      useSemis = !useSemis; curCode = noSemiCode; curSize = noSemiSize;
    }
    for (const target of [48, 64, 80, 96, 128]) {
      const packed = packLines(curCode, target);
      const packedSize = size(packed);
      if (packedSize < curSize) {
        console.error(`[${label}] ✓ pack${target}: ${curSize} → ${packedSize}`);
        curCode = packed; curSize = packedSize; improved = true; break;
      }
    }
    if (improved) console.error(`[${label}] → Restarting (${curSize})`);
    else          console.error(`[${label}] → No improvement`);
  }
  return curCode;
}

// ── Minify both variants ─────────────────────────────────────────────────────
const [minFull, minDecoder] = await Promise.all([
  hillClimb(jsFull,    "full"),
  hillClimb(jsDecoder, "decoder"),
]);

await Deno.writeTextFile(OUTPUT_FULL, minFull);
await Deno.writeTextFile(OUTPUT_DEC,  minDecoder);
console.error(`\nFull:    ${size(jsFull)} → ${size(minFull)} bytes → ${OUTPUT_FULL}`);
console.error(`Decoder: ${size(jsDecoder)} → ${size(minDecoder)} bytes → ${OUTPUT_DEC}`);

// ── Encode full script through encode-lines ──────────────────────────────────
// Run the decoder to encode the full script: needs d to unlock z
async function encodeLines(input: Uint8Array): Promise<string> {
  const cmd = new Deno.Command("deno", {
    args: ["run", "--allow-read", OUTPUT_FULL, "encode-lines"],
    stdin: "piped", stdout: "piped", stderr: "inherit",
  });
  const proc = cmd.spawn();
  const writer = proc.stdin.getWriter();
  await writer.write(input);
  await writer.close();
  const { stdout } = await proc.output();
  return new TextDecoder().decode(stdout);
}

const encodedFull = await encodeLines(new TextEncoder().encode(minFull));
console.error(`Encoded z: ${encodedFull.length} chars`);

// Verify roundtrip
async function decodeLines(input: string): Promise<Uint8Array> {
  const cmd = new Deno.Command("deno", {
    args: ["run", "--allow-read", OUTPUT_DEC, "decode-lines"],
    stdin: "piped", stdout: "piped", stderr: "inherit",
  });
  const proc = cmd.spawn();
  const writer = proc.stdin.getWriter();
  await writer.write(new TextEncoder().encode(input));
  await writer.close();
  const { stdout } = await proc.output();
  return stdout;
}

const decoded = await decodeLines(encodedFull);
const decodedStr = new TextDecoder().decode(decoded);
if (decodedStr !== minFull) {
  console.error("❌ ROUNDTRIP FAILED: decoded z does not match original");
  Deno.exit(1);
}
console.error("✅ Roundtrip verified: d can decode z");

// ── Build zipng in temp dir ──────────────────────────────────────────────────
const tmpDir = await Deno.makeTempDir({ prefix: "z855-zipng-" });
console.error(`\nBuilding zipng in: ${tmpDir}`);

// Write files with short names
await Deno.writeTextFile(`${tmpDir}/d`, minDecoder);          // decoder
await Deno.writeTextFile(`${tmpDir}/z`, encodedFull);         // encoded full (locked)

const zipngOut = `${tmpDir}/z855.zipng`;
const zipngCmd = new Deno.Command("deno", {
  args: ["run", "--allow-read", "--allow-write", ZIPNG_CLI,
         "-o", "z855.zipng", "d", "z"],
  cwd: tmpDir,
  stdout: "inherit",
  stderr: "inherit",
});
const { code: zipCode } = await zipngCmd.output();
if (zipCode !== 0) { console.error("❌ zipng failed"); Deno.exit(1); }

// Copy result out
await Deno.copyFile(zipngOut, "./z855.zipng");
console.error(`\n✅ z855.zipng written (${(await Deno.stat("./z855.zipng")).size} bytes)`);
console.error(`   d: ${size(minDecoder)} bytes (decoder only)`);
console.error(`   z: ${encodedFull.length} chars (encoded full — decode with d to unlock)`);

// Cleanup temp dir
await Deno.remove(tmpDir, { recursive: true });

console.log("./z855.zipng");
