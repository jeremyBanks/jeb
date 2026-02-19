#!/usr/bin/env -S deno run --allow-write

/**
 * Generate RANDOMIZED binary input files for gap testing
 *
 * Unlike the pattern-based gap tests (all A's, all B's, all zeros),
 * this generates tests with:
 * - First run: RANDOM safe characters from Z855 safe set
 * - Second run: DIFFERENT random safe characters
 * - Gaps: RANDOM unsafe characters (not just zeros)
 *
 * Creates test files with the pattern: [run1 (random safe)] + [gap (random unsafe)] + [run2 (random safe)]
 *
 * Run lengths: 4, 5, 6, 7, 8 bytes
 * Gap sizes: 0, 1, 2, 3, 4, 5, 8, 16 bytes
 * Variations: 5 per combination (for randomness coverage)
 */

const RUN_LENGTHS = [4, 5, 6, 7, 8];
const GAP_SIZES = [0, 1, 2, 3, 4, 5, 8, 16];
const VARIATIONS = 5; // Generate 5 variations per combination
const OUTPUT_DIR = "test-cases";

// Z855 safe character set (Z85 alphabet + escape chars: ,;|~_)
// From z855.ts line 34: 0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#,;|~_
const SAFE_CHARS = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#,;|~_";

// Convert to byte array for easy random selection
const SAFE_BYTES = new Uint8Array(SAFE_CHARS.split('').map(c => c.charCodeAt(0)));

// Unsafe bytes: anything NOT in the safe set
// This includes: control chars (0x00-0x1F), space (0x20), delete (0x7F), and high bytes (0x80+)
// Plus some punctuation not in the safe set: " ' ` \ and a few others
function generateUnsafeBytes(count) {
  const unsafe = new Uint8Array(count);
  const unsafeRanges = [
    // Control characters (0x00-0x1F)
    { start: 0x00, end: 0x1F },
    // Space and some punctuation not in safe set
    { start: 0x20, end: 0x20 }, // space
    { start: 0x22, end: 0x22 }, // "
    { start: 0x27, end: 0x27 }, // '
    { start: 0x5C, end: 0x5C }, // \
    { start: 0x60, end: 0x60 }, // `
    { start: 0x7F, end: 0x7F }, // delete
    // High bytes (0x80-0xFF)
    { start: 0x80, end: 0xFF },
  ];

  for (let i = 0; i < count; i++) {
    // Pick a random unsafe range
    const range = unsafeRanges[Math.floor(Math.random() * unsafeRanges.length)];
    // Pick a random byte from that range
    unsafe[i] = range.start + Math.floor(Math.random() * (range.end - range.start + 1));
  }

  return unsafe;
}

// Generate random safe bytes, trying to avoid duplicates within the same run
function generateSafeBytesUnique(count, excludeBytes = new Set()) {
  const result = new Uint8Array(count);
  const available = SAFE_BYTES.filter(b => !excludeBytes.has(b));

  if (available.length === 0) {
    throw new Error("No safe bytes available after exclusions!");
  }

  for (let i = 0; i < count; i++) {
    // If we have enough unique bytes left, try to pick one we haven't used yet in this run
    const used = new Set(result.slice(0, i));
    const remaining = available.filter(b => !used.has(b));

    if (remaining.length > 0) {
      // Pick from remaining unique bytes
      result[i] = remaining[Math.floor(Math.random() * remaining.length)];
    } else {
      // Fall back to any available byte (allows duplicates if run is longer than alphabet)
      result[i] = available[Math.floor(Math.random() * available.length)];
    }
  }

  return result;
}

function generateRandomGapTestFile(gapSize, run1Len, run2Len) {
  // Calculate total length
  const totalLen = run1Len + gapSize + run2Len;

  // Create buffer
  const buffer = new Uint8Array(totalLen);

  // Fill first run with random safe bytes
  const run1 = generateSafeBytesUnique(run1Len);
  buffer.set(run1, 0);

  // Collect run1 bytes to avoid duplicates in run2
  const run1Set = new Set(run1);

  // Fill gap with random unsafe bytes (if gap > 0)
  if (gapSize > 0) {
    const gap = generateUnsafeBytes(gapSize);
    buffer.set(gap, run1Len);
  }

  // Fill second run with random safe bytes (different from run1 if possible)
  const run2 = generateSafeBytesUnique(run2Len, run1Set);
  buffer.set(run2, run1Len + gapSize);

  return buffer;
}

function bytesToHexString(bytes) {
  return Array.from(bytes, b => b.toString(16).padStart(2, '0')).join(' ');
}

function bytesToDisplayString(bytes) {
  return Array.from(bytes, b => {
    if (b >= 0x20 && b <= 0x7E) {
      return String.fromCharCode(b);
    } else {
      return `\\x${b.toString(16).padStart(2, '0')}`;
    }
  }).join('');
}

function main() {
  let filesCreated = 0;
  const fileList = [];
  const safeCharsUsed = new Set();
  const unsafeCharsUsed = new Set();
  const sampleInputs = [];

  console.log("Generating RANDOMIZED gap test files...\n");
  console.log("Run lengths:", RUN_LENGTHS);
  console.log("Gap sizes:", GAP_SIZES);
  console.log("Variations per combination:", VARIATIONS);
  console.log("");

  // Generate all combinations with variations
  for (const gapSize of GAP_SIZES) {
    for (const run1Len of RUN_LENGTHS) {
      for (const run2Len of RUN_LENGTHS) {
        for (let variation = 0; variation < VARIATIONS; variation++) {
          // Generate filename: gap-rand-<variation>-<gapsize>-<run1len>-<run2len>.input
          const filename = `gap-rand-${variation}-${gapSize}-${run1Len}-${run2Len}.input`;
          const filepath = `${OUTPUT_DIR}/${filename}`;

          // Generate random binary content
          const content = generateRandomGapTestFile(gapSize, run1Len, run2Len);

          // Write file
          Deno.writeFileSync(filepath, content);

          filesCreated++;
          fileList.push({
            filename,
            variation,
            gapSize,
            run1Len,
            run2Len,
            totalBytes: content.length,
            content
          });

          // Track character usage
          for (let i = 0; i < run1Len; i++) {
            safeCharsUsed.add(content[i]);
          }
          for (let i = run1Len; i < run1Len + gapSize; i++) {
            unsafeCharsUsed.add(content[i]);
          }
          for (let i = run1Len + gapSize; i < content.length; i++) {
            safeCharsUsed.add(content[i]);
          }

          // Collect samples (first 3 files)
          if (sampleInputs.length < 3) {
            sampleInputs.push({
              filename,
              content,
              gapSize,
              run1Len,
              run2Len
            });
          }
        }
      }
    }
  }

  // Print summary
  console.log("\n" + "=".repeat(80));
  console.log("GENERATION COMPLETE");
  console.log("=".repeat(80));
  console.log(`Total files created: ${filesCreated}`);
  console.log(`Files location: ${OUTPUT_DIR}/gap-rand-*.input`);
  console.log("");

  // Character variety statistics
  console.log("CHARACTER VARIETY:");
  console.log("-".repeat(80));
  console.log(`Safe characters used: ${safeCharsUsed.size} unique bytes`);
  console.log(`  (out of ${SAFE_BYTES.length} total safe chars in Z855 set)`);
  console.log(`Unsafe characters used: ${unsafeCharsUsed.size} unique bytes`);
  console.log("");

  // Show safe character samples
  const safeCharsSorted = Array.from(safeCharsUsed).sort((a, b) => a - b);
  const safeCharsDisplay = safeCharsSorted.slice(0, 30).map(b =>
    String.fromCharCode(b)
  ).join('');
  console.log(`Safe chars sample: ${safeCharsDisplay}${safeCharsSorted.length > 30 ? '...' : ''}`);

  // Show unsafe character samples
  const unsafeCharsSorted = Array.from(unsafeCharsUsed).sort((a, b) => a - b);
  const unsafeCharsDisplay = unsafeCharsSorted.slice(0, 15).map(b =>
    `0x${b.toString(16).padStart(2, '0')}`
  ).join(' ');
  console.log(`Unsafe bytes sample: ${unsafeCharsDisplay}${unsafeCharsSorted.length > 15 ? ' ...' : ''}`);
  console.log("");

  // Print sample random inputs
  console.log("SAMPLE RANDOM INPUTS:");
  console.log("-".repeat(80));
  for (const { filename, content, gapSize, run1Len, run2Len } of sampleInputs) {
    const run1 = content.slice(0, run1Len);
    const gap = content.slice(run1Len, run1Len + gapSize);
    const run2 = content.slice(run1Len + gapSize);

    console.log(`\n${filename}`);
    console.log(`  Run1 (${run1Len} bytes): ${bytesToDisplayString(run1)}`);
    console.log(`    Hex: ${bytesToHexString(run1)}`);
    if (gapSize > 0) {
      console.log(`  Gap  (${gapSize} bytes): ${bytesToDisplayString(gap)}`);
      console.log(`    Hex: ${bytesToHexString(gap)}`);
    } else {
      console.log(`  Gap  (0 bytes): [none]`);
    }
    console.log(`  Run2 (${run2Len} bytes): ${bytesToDisplayString(run2)}`);
    console.log(`    Hex: ${bytesToHexString(run2)}`);
  }

  // Print breakdown by gap size
  console.log("\n" + "=".repeat(80));
  console.log("FILES BY GAP SIZE:");
  console.log("=".repeat(80));
  for (const gapSize of GAP_SIZES) {
    const files = fileList.filter(f => f.gapSize === gapSize);
    console.log(`\nGap size ${gapSize} bytes: ${files.length} files`);
    console.log(`  Pattern: gap-rand-{0-${VARIATIONS-1}}-${gapSize}-{${RUN_LENGTHS.join(',')}}-{${RUN_LENGTHS.join(',')}}.input`);
  }

  console.log("\n" + "=".repeat(80));
  console.log("NEXT STEPS:");
  console.log("-".repeat(80));
  console.log("1. Run: ./encode-gap-tests.mjs");
  console.log("   (This will encode ALL gap tests, including these new random ones)");
  console.log("");
  console.log("2. Run: ./verify-gap-tests.mjs");
  console.log("   (This will verify ALL gap tests round-trip correctly)");
  console.log("");
  console.log("3. Compare coverage between pattern-based and random tests");
  console.log("");
  console.log(`Total files created: ${filesCreated}`);
  console.log(`Files are in: ${OUTPUT_DIR}/gap-rand-*.input`);
}

main();
