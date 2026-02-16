#!/usr/bin/env -S deno run --allow-read --allow-write --allow-env
import { minify } from "npm:terser@^5.36";

const filePath = "./min.mjs";

// Custom identifier generator following your frequency preference
// _JjEeBb0Zz85IiPpNnGgFfTt1AaOo2SsRr3HhLl4DdCc6UuMm7WwKk9YyVvXxQq$
const identifierChars = "_JjEeBbZzIiPpNnGgFfTtAaOoSsRrHhLlDdCcUuMmWwKkYyVvXxQq$0123456789";
const firstChars = "_JjEeBbZzIiPpNnGgFfTtAaOoSsRrHhLlDdCcUuMmWwKkYyVvXxQq$"; // No digits at start

function nthIdentifier(n: number): string {
  const firstCharCount = firstChars.length;
  const restCharCount = identifierChars.length;

  // First identifier is just the first char
  if (n === 0) return firstChars[0];

  // For single character identifiers (n < firstCharCount)
  if (n < firstCharCount) {
    return firstChars[n];
  }

  // For multi-character identifiers
  // Adjust n to account for single-char identifiers already used
  let remaining = n - firstCharCount;

  // Build identifier from right to left
  let result = "";
  let isFirst = true;

  while (remaining >= 0 || result === "") {
    if (isFirst) {
      result = identifierChars[remaining % restCharCount] + result;
      remaining = Math.floor(remaining / restCharCount);
      isFirst = false;
    } else {
      if (remaining === 0 && result.length > 0) break;
      result = firstChars[remaining % firstCharCount] + result;
      remaining = Math.floor(remaining / firstCharCount);
    }

    if (remaining === 0 && result.length > 1) break;
  }

  return result;
}

// Base minify options
const baseOptions = {
  ecma: 2020,
  module: true,
  compress: {
    passes: 3,
    hoist_vars: false,
    inline: false,
    properties: false,
    pure_getters: false,
    unsafe: false,
    unsafe_arrows: false,
    unsafe_comps: false,
  },
  mangle: {
    toplevel: true,
    nth_identifier: nthIdentifier,
  },
  format: {
    comments: false,
  },
};

const testOptions = [
  "hoist_vars",
  "inline",
  "properties",
  "pure_getters",
  "unsafe",
  "unsafe_arrows",
  "unsafe_comps",
] as const;

async function tryMinify(code: string, optionName: string, enabled: boolean) {
  const options = JSON.parse(JSON.stringify(baseOptions)); // Deep copy
  options.compress[optionName] = enabled;

  const result = await minify(code, options);
  if (!result.code) {
    throw new Error("Terser failed to produce output");
  }

  return result.code;
}

// Read the current file
let currentCode = await Deno.readTextFile(filePath);
let currentSize = new TextEncoder().encode(currentCode).length;

console.error(`Starting size: ${currentSize} bytes\n`);

let improved = true;
let round = 0;

// Keep looping until no option produces improvement
while (improved) {
  improved = false;
  round++;
  console.error(`=== Round ${round} ===`);

  for (const option of testOptions) {
    const testCode = await tryMinify(currentCode, option, true);
    const testSize = new TextEncoder().encode(testCode).length;
    const delta = testSize - currentSize;

    if (testSize < currentSize) {
      console.error(`✓ ${option}: ${currentSize} → ${testSize} (${delta} bytes)`);
      currentCode = testCode;
      currentSize = testSize;
      improved = true;
      // Break and restart from beginning when we find improvement
      break;
    } else {
      console.error(`✗ ${option}: ${currentSize} → ${testSize} (${delta >= 0 ? '+' : ''}${delta} bytes)`);
    }
  }

  if (improved) {
    console.error(`→ Restarting loop with new baseline: ${currentSize} bytes\n`);
  } else {
    console.error(`→ No improvements found in this round\n`);
  }
}

// Read original file to compare final result
const originalCode = await Deno.readTextFile(filePath);
const originalSize = new TextEncoder().encode(originalCode).length;

if (currentSize < originalSize) {
  await Deno.writeTextFile(filePath, currentCode);
  const saved = originalSize - currentSize;
  const percent = ((saved / originalSize) * 100).toFixed(2);
  console.error(`\n✓ Final: ${originalSize} → ${currentSize} bytes (saved ${saved} bytes, ${percent}%)`);
  console.error(`File updated: ${filePath}`);
} else if (currentSize === originalSize) {
  console.error(`\n✗ No improvement: size unchanged at ${currentSize} bytes`);
} else {
  console.error(`\n✗ Result larger: ${originalSize} → ${currentSize} bytes. Keeping original.`);
}
