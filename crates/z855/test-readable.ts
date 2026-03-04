import { encode, decode, Z855DecodeError } from "./z855-readable.ts";
import { join } from "https://deno.land/std@0.224.0/path/mod.ts";

const dir = "test-cases";
const names = [...Deno.readDirSync(dir)].filter(e => e.name.endsWith(".input"))
  .map(e => e.name.replace(".input", "")).sort();

let pass = 0, fail = 0;
const failures: string[] = [];

for (const name of names) {
  const input = Deno.readFileSync(join(dir, `${name}.input`));
  let encRaw: Uint8Array;
  try { encRaw = Deno.readFileSync(join(dir, `${name}.encoded`)); } catch { continue; }
  const encStr = new TextDecoder("latin1").decode(encRaw).trim();

  if (new TextDecoder().decode(input).trim() === "<error />") {
    let threw = false;
    try { decode(encStr); } catch (e) { if (e instanceof Z855DecodeError) threw = true; }
    threw ? pass++ : (fail++, failures.push(`${name}: expected error`));
    continue;
  }

  try {
    const decoded = decode(encStr);
    if (decoded.length !== input.length || !decoded.every((b, i) => b === input[i]))
      throw new Error(`decode mismatch: got ${decoded.length}b, want ${input.length}b`);
    const roundtrip = decode(encode(input));
    if (roundtrip.length !== input.length || !roundtrip.every((b, i) => b === input[i]))
      throw new Error("roundtrip mismatch");
    pass++;
  } catch (e) {
    fail++; failures.push(`${name}: ${e}`);
  }
}

console.log(`${pass} / ${pass + fail} passed`);
if (failures.length) {
  failures.slice(0, 30).forEach(f => console.log("  FAIL:", f));
  if (failures.length > 30) console.log(`  ... and ${failures.length - 30} more`);
  Deno.exit(1);
}
