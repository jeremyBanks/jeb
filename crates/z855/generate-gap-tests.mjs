#!/usr/bin/env -S deno run --allow-write

/**
 * Generate binary input files for gap testing
 *
 * Creates test files with the pattern: [run1 (As)] + [gap (zeros)] + [run2 (Bs)]
 *
 * Run lengths: 4, 5, 6, 7, 8 bytes
 * Gap sizes: 0, 1, 2, 3, 4, 5, 8, 16 bytes
 */

const RUN_LENGTHS = [4, 5, 6, 7, 8];
const GAP_SIZES = [0, 1, 2, 3, 4, 5, 8, 16];
const OUTPUT_DIR = "test-cases";

// Character codes
const CHAR_A = 0x41; // 'A'
const CHAR_B = 0x42; // 'B'
const CHAR_ZERO = 0x00;

function generateGapTestFile(gapSize, run1Len, run2Len) {
  // Calculate total length
  const totalLen = run1Len + gapSize + run2Len;

  // Create buffer
  const buffer = new Uint8Array(totalLen);

  // Fill first run with 'A'
  buffer.fill(CHAR_A, 0, run1Len);

  // Fill gap with zeros (already 0 by default, but explicit for clarity)
  buffer.fill(CHAR_ZERO, run1Len, run1Len + gapSize);

  // Fill second run with 'B'
  buffer.fill(CHAR_B, run1Len + gapSize, totalLen);

  return buffer;
}

function main() {
  let filesCreated = 0;
  const fileList = [];

  console.log("Generating gap test files...\n");
  console.log("Run lengths:", RUN_LENGTHS);
  console.log("Gap sizes:", GAP_SIZES);
  console.log("");

  // Generate all combinations
  for (const gapSize of GAP_SIZES) {
    for (const run1Len of RUN_LENGTHS) {
      for (const run2Len of RUN_LENGTHS) {
        // Generate filename: gap-<gapsize>-<run1len>-<run2len>.input
        const filename = `gap-${gapSize}-${run1Len}-${run2Len}.input`;
        const filepath = `${OUTPUT_DIR}/${filename}`;

        // Generate binary content
        const content = generateGapTestFile(gapSize, run1Len, run2Len);

        // Write file
        Deno.writeFileSync(filepath, content);

        filesCreated++;
        fileList.push({
          filename,
          gapSize,
          run1Len,
          run2Len,
          totalBytes: content.length
        });
      }
    }
  }

  // Print summary
  console.log(`\nGenerated ${filesCreated} test files in ${OUTPUT_DIR}/\n`);

  // Print grouped by gap size
  for (const gapSize of GAP_SIZES) {
    console.log(`\nGap size ${gapSize} bytes:`);
    const files = fileList.filter(f => f.gapSize === gapSize);
    for (const file of files) {
      const pattern = gapSize === 0
        ? `[${'A'.repeat(file.run1Len)}${'B'.repeat(file.run2Len)}]`
        : `[${'A'.repeat(file.run1Len)}] + ${gapSize} zeros + [${'B'.repeat(file.run2Len)}]`;
      console.log(`  ${file.filename.padEnd(25)} - ${file.totalBytes} bytes - ${pattern}`);
    }
  }

  console.log(`\n\nTotal files created: ${filesCreated}`);
  console.log(`Files are in: ${OUTPUT_DIR}/`);
}

main();
