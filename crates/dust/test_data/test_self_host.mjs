// Test harness for the self-hosting dust compiler
// Usage: node test_self_host.mjs
import { readFileSync } from "fs";

const selfHostWasm = readFileSync("crates/dust/test_data/self_host.wasm");

// A simple test program to compile
const testProgram = new TextEncoder().encode(
  `fn add(a: i32, b: i32) -> i32 { return a + b; }
fn main() -> i32 { return add(19, 23); }
`
);

// Instantiate the self-hosting compiler
const compilerModule = await WebAssembly.instantiate(selfHostWasm);
const compiler = compilerModule.instance.exports;
const compilerMem = new Uint8Array(compiler.memory.buffer);

// Copy source into compiler's memory at offset 0 (SRC_BASE)
compilerMem.set(testProgram, 0);

// Check what the compiler exports
console.log("Compiler exports:", Object.keys(compiler).sort());

// Run the compiler
const outputLen = compiler.compile(testProgram.length);

// The compiler may have grown memory, refresh the view
const compilerMem2 = new Uint8Array(compiler.memory.buffer);

// Read output from OUT_BASE (0x10000 = 65536)
const outputWasm = compilerMem2.slice(65536, 65536 + outputLen);

console.log(`Compiled test program: ${testProgram.length} bytes source -> ${outputLen} bytes WASM`);

// Verify it's valid WASM
if (outputWasm[0] !== 0 || outputWasm[1] !== 0x61 || outputWasm[2] !== 0x73 || outputWasm[3] !== 0x6d) {
  console.error("ERROR: output doesn't start with WASM magic number!");
  console.error("First 16 bytes:", Array.from(outputWasm.slice(0, 16)).map(b => b.toString(16).padStart(2, "0")).join(" "));
  process.exit(1);
}
console.log("WASM header valid: \\0asm v1");

// Try to instantiate the compiled WASM
try {
  const testModule = await WebAssembly.instantiate(outputWasm);
  const testExports = testModule.instance.exports;

  console.log("Compiled module exports:", Object.keys(testExports));

  if (testExports.main) {
    const result = testExports.main();
    console.log(`main() = ${result}`);
    if (result === 42) {
      console.log("SUCCESS: 19 + 23 = 42");
    } else {
      console.error(`FAIL: expected 42, got ${result}`);
      process.exit(1);
    }
  }

  if (testExports.add) {
    const result = testExports.add(100, 200);
    console.log(`add(100, 200) = ${result}`);
    if (result !== 300) {
      console.error(`FAIL: expected 300, got ${result}`);
      process.exit(1);
    }
  }

  console.log("\nAll tests passed!");
} catch (e) {
  console.error("ERROR instantiating compiled WASM:", e);
  console.error("First 64 bytes:", Array.from(outputWasm.slice(0, 64)).map(b => b.toString(16).padStart(2, "0")).join(" "));
  process.exit(1);
}
