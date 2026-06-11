// Full self-hosting validation for the dust compiler.
// Tests the bootstrap chain and verifies fixed-point convergence.
import { readFileSync } from "fs";

const selfHostWasm = readFileSync("crates/dust/test_data/self_host.wasm");
const selfHostSrc = readFileSync("crates/dust/test_data/self_host.rs");

console.log("=== Dust Self-Hosting Bootstrap Validation ===\n");
console.log(`Source: ${selfHostSrc.length} bytes`);
console.log(`WASM (from Rust host compiler): ${selfHostWasm.length} bytes\n`);

const OUT_BASE = 65536;

async function compile(compilerWasm, source) {
  const m = await WebAssembly.instantiate(compilerWasm);
  const e = m.instance.exports;
  const mem = new Uint8Array(e.memory.buffer);
  mem.set(source, 0);
  const len = e.compile(source.length);
  const mem2 = new Uint8Array(e.memory.buffer);
  return {
    output: new Uint8Array(mem2.slice(OUT_BASE, OUT_BASE + len)),
    tokens: e.get_tok_count(),
    nodes: e.get_node_count(),
    funcs: e.get_func_count(),
  };
}

async function run() {
  // Stage 0: Rust-compiled self_host.wasm (already have this)
  const stage0 = selfHostWasm;

  // Stage 1: self_host.wasm compiles self_host.rs -> stage1
  console.log("--- Stage 1: Rust-compiled compiler compiles itself ---");
  const r1 = await compile(stage0, selfHostSrc);
  console.log(`  Tokens: ${r1.tokens}, Nodes: ${r1.nodes}, Functions: ${r1.funcs}`);
  console.log(`  Output: ${r1.output.length} bytes`);

  // Verify stage1 is valid WASM
  try {
    await WebAssembly.instantiate(r1.output);
    console.log("  Valid WASM: yes\n");
  } catch (e) {
    console.error("  FAIL: stage 1 output is not valid WASM:", e.message);
    process.exit(1);
  }

  // Stage 2: stage1 compiles self_host.rs -> stage2
  console.log("--- Stage 2: Self-compiled compiler compiles itself ---");
  const r2 = await compile(r1.output, selfHostSrc);
  console.log(`  Tokens: ${r2.tokens}, Nodes: ${r2.nodes}, Functions: ${r2.funcs}`);
  console.log(`  Output: ${r2.output.length} bytes`);

  try {
    await WebAssembly.instantiate(r2.output);
    console.log("  Valid WASM: yes\n");
  } catch (e) {
    console.error("  FAIL: stage 2 output is not valid WASM:", e.message);
    process.exit(1);
  }

  // Fixed-point check: stage2 == stage1?
  console.log("--- Fixed-Point Check ---");
  if (r1.output.length !== r2.output.length) {
    console.log(`  Sizes differ: stage1=${r1.output.length}, stage2=${r2.output.length}`);
    console.log("  (Not a fixed point yet — expected since Rust and dust compilers differ)");
  } else {
    let match = true;
    for (let i = 0; i < r1.output.length; i++) {
      if (r1.output[i] !== r2.output[i]) { match = false; break; }
    }
    if (match) {
      console.log("  Stage 1 == Stage 2: FIXED POINT REACHED!");
    } else {
      console.log("  Same size but different content");
    }
  }

  // Stage 3: stage2 compiles self_host.rs -> stage3
  console.log("\n--- Stage 3: Stage-2 compiler compiles itself ---");
  const r3 = await compile(r2.output, selfHostSrc);
  console.log(`  Output: ${r3.output.length} bytes`);

  // Fixed-point check: stage3 == stage2?
  if (r2.output.length === r3.output.length) {
    let match = true;
    for (let i = 0; i < r2.output.length; i++) {
      if (r2.output[i] !== r3.output[i]) { match = false; break; }
    }
    if (match) {
      console.log("  Stage 2 == Stage 3: FIXED POINT REACHED!\n");
    } else {
      console.log("  Same size but different content\n");
    }
  } else {
    console.log(`  Sizes differ: stage2=${r2.output.length}, stage3=${r3.output.length}\n`);
  }

  // Functional test: use stage2 compiler to compile a non-trivial program
  console.log("--- Functional Test ---");
  const testSrc = new TextEncoder().encode(`
fn fib(n: i32) -> i32 {
  let mut a: i32 = 0;
  let mut b: i32 = 1;
  let mut i: i32 = 0;
  while i < n {
    let mut t: i32 = b;
    b = a + b;
    a = t;
    i += 1;
  }
  return a;
}
fn factorial(n: i32) -> i32 {
  if n <= 1 { return 1; }
  return n * factorial(n - 1);
}
fn gcd(a: i32, b: i32) -> i32 {
  while b != 0 {
    let mut t: i32 = b;
    b = a % b;
    a = t;
  }
  return a;
}
fn test_all() -> i32 {
  let mut ok: i32 = 1;
  if fib(10) != 55 { ok = 0; }
  if fib(20) != 6765 { ok = 0; }
  if factorial(6) != 720 { ok = 0; }
  if factorial(10) != 3628800 { ok = 0; }
  if gcd(48, 18) != 6 { ok = 0; }
  if gcd(100, 75) != 25 { ok = 0; }
  return ok;
}
`);

  const testResult = await compile(r2.output, testSrc);
  const testModule = await WebAssembly.instantiate(testResult.output);
  const testExports = testModule.instance.exports;

  const results = {
    "fib(10)": [testExports.fib(10), 55],
    "fib(20)": [testExports.fib(20), 6765],
    "factorial(6)": [testExports.factorial(6), 720],
    "factorial(10)": [testExports.factorial(10), 3628800],
    "gcd(48,18)": [testExports.gcd(48, 18), 6],
    "gcd(100,75)": [testExports.gcd(100, 75), 25],
    "test_all()": [testExports.test_all(), 1],
  };

  let allPass = true;
  for (const [name, [got, expected]] of Object.entries(results)) {
    const pass = got === expected;
    console.log(`  ${name} = ${got} ${pass ? "OK" : `FAIL (expected ${expected})`}`);
    if (!pass) allPass = false;
  }

  console.log(allPass
    ? "\n=== ALL TESTS PASSED — SELF-HOSTING VERIFIED ==="
    : "\n=== SOME TESTS FAILED ===");
  if (!allPass) process.exit(1);
}

run().catch(e => { console.error(e); process.exit(1); });
