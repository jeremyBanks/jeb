#!/usr/bin/env -S deno run --allow-read --allow-write --allow-env
import { minify } from "npm:terser@^5.36";

const filePath = "./min.mjs";

// Custom identifier generator following your frequency preference
// ZzJEBjebIiPpNnGgFfTtAaOoSsRrHhLlDdCcUuMmWwKkYyVvXxQq_$
const identifierChars = "ZzJEBjebIiPpNnGgFfTtAaOoSsRrHhLlDdCcUuMmWwKkYyVvXxQq_$0123456789";
const firstChars = "ZzJEBjebIiPpNnGgFfTtAaOoSsRrHhLlDdCcUuMmWwKkYyVvXxQq_$"; // No digits at start

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
    nth_identifier: {
      get(n: number) {
        return nthIdentifier(n);
      },
    },
  },
  format: {
    comments: false,
    semicolons: true,
  },
};

const testOptions = [
  { name: "hoist_vars", section: "compress", testValue: true },
  { name: "inline", section: "compress", testValue: true },
  { name: "properties", section: "compress", testValue: true },
  { name: "pure_getters", section: "compress", testValue: true },
  { name: "unsafe", section: "compress", testValue: true },
  { name: "unsafe_arrows", section: "compress", testValue: true },
  { name: "unsafe_comps", section: "compress", testValue: true },
] as const;

async function tryMinify(
  code: string,
  section: "compress" | "format",
  optionName: string,
  value: boolean
) {
  const options = JSON.parse(JSON.stringify(baseOptions)); // Deep copy
  options[section][optionName] = value;
  // Restore the nth_identifier cache (lost in JSON serialization)
  options.mangle.nth_identifier = {
    get(n: number) {
      return nthIdentifier(n);
    },
  };

  const result = await minify(code, options);
  if (!result.code) {
    throw new Error("Terser failed to produce output");
  }

  return result.code;
}

function packLinesToTarget(code: string, initialTarget: number): string {
  // Adaptive packing: each line's target is the previous line's actual width
  // Target is clamped to [48, 256], over-target is penalized 4x vs under
  const minLength = 32;
  const minTarget = 48;
  const maxTarget = 256;
  const lines = code.split("\n").map((l) => l.trim()).filter((l) => l);
  const packed: string[] = [];
  let currentLine = "";
  let targetWidth = initialTarget;

  for (const line of lines) {
    if (!currentLine) {
      currentLine = line;
      continue;
    }

    const merged = currentLine + ";" + line;

    // Enforce minimum length - keep merging until we hit it
    if (currentLine.length < minLength) {
      currentLine = merged;
      continue;
    }

    // Calculate penalties (over is 4x worse than under)
    const currentPenalty = currentLine.length < targetWidth
      ? (targetWidth - currentLine.length)
      : 4 * (currentLine.length - targetWidth);
    const mergedPenalty = merged.length < targetWidth
      ? (targetWidth - merged.length)
      : 4 * (merged.length - targetWidth);

    if (mergedPenalty <= currentPenalty) {
      // Merging is better (or equal), keep going
      currentLine = merged;
    } else {
      // Better to start new line
      packed.push(currentLine);
      // Next target is this line's actual width (clamped)
      targetWidth = Math.max(minTarget, Math.min(maxTarget, currentLine.length));
      currentLine = line;
    }
  }

  if (currentLine) {
    packed.push(currentLine);
  }

  return packed.join("\n");
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
    const testCode = await tryMinify(currentCode, option.section, option.name, option.testValue);
    const testSize = new TextEncoder().encode(testCode).length;
    const delta = testSize - currentSize;

    if (testSize < currentSize) {
      console.error(`✓ ${option.name}: ${currentSize} → ${testSize} (${delta} bytes)`);
      currentCode = testCode;
      currentSize = testSize;
      improved = true;
      // Break and restart from beginning when we find improvement
      break;
    } else {
      console.error(`✗ ${option.name}: ${currentSize} → ${testSize} (${delta >= 0 ? '+' : ''}${delta} bytes)`);
    }
  }

  // Special handling for semicolons: test at end of each round, favor newlines on tie
  const noSemicolonsCode = await tryMinify(currentCode, "format", "semicolons", false);
  const noSemicolonsSize = new TextEncoder().encode(noSemicolonsCode).length;
  const semicolonsDelta = noSemicolonsSize - currentSize;

  if (noSemicolonsSize <= currentSize) {
    if (noSemicolonsSize < currentSize) {
      console.error(`✓ semicolons: ${currentSize} → ${noSemicolonsSize} (${semicolonsDelta} bytes)`);
    } else {
      console.error(`= semicolons: ${currentSize} → ${noSemicolonsSize} (tie, favoring newlines)`);
    }
    currentCode = noSemicolonsCode;
    currentSize = noSemicolonsSize;
  } else {
    console.error(`✗ semicolons: ${currentSize} → ${noSemicolonsSize} (${semicolonsDelta >= 0 ? '+' : ''}${semicolonsDelta} bytes)`);
  }

  // Custom pass: pack lines to ~64 characters by replacing newlines with semicolons
  const packedCode = packLinesToTarget(currentCode, 64);
  const packedSize = new TextEncoder().encode(packedCode).length;
  const packDelta = packedSize - currentSize;

  if (packedSize <= currentSize) {
    if (packedSize < currentSize) {
      console.error(`✓ pack64: ${currentSize} → ${packedSize} (${packDelta} bytes)`);
      currentCode = packedCode;
      currentSize = packedSize;
      improved = true;
    } else {
      console.error(`= pack64: ${currentSize} → ${packedSize} (tie)`);
    }
  } else {
    console.error(`✗ pack64: ${currentSize} → ${packedSize} (${packDelta >= 0 ? '+' : ''}${packDelta} bytes)`);
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
