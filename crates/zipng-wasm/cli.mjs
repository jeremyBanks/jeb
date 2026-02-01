#!/usr/bin/env node
/**
 * Node.js CLI for zipng polyglot encoder
 *
 * Usage:
 *   zipng -o output.png file1.txt file2.txt
 *   npx zipng-wasm --mode rgba --font swiss -o out.png *.txt
 */

import { encode } from './zipng_wasm.js';
import { readFileSync, writeFileSync, statSync } from 'fs';
import { resolve } from 'path';

const MAX_FILE_SIZE = 60 * 1024; // 60KB

function printUsage() {
  console.log(`
zipng - Encode files into PNG+ZIP polyglots

Usage:
  zipng [OPTIONS] -o OUTPUT FILES...

Options:
  -o, --output PATH         Output PNG file path (required)
  --mode MODE               Color mode: auto, indexed, rgba (default: auto)
  --font FONT               Font: swiss, sixth, sky, monte, sugimori, mini, micro
  --sort MODE               Sort: lexicographic, reverse, by_size, by_extension, none (default: lexicographic)
  -h, --help                Show this help

Examples:
  # Basic usage
  zipng -o archive.png file1.txt file2.txt

  # With options
  zipng --mode indexed --font swiss --sort reverse -o out.png *.txt

  # Use as library
  import { encode } from 'zipng-wasm';
`);
}

function parseArgs(args) {
  const parsed = {
    output: null,
    mode: 'auto',
    font: null,
    sort: 'lexicographic',
    help: false,
    files: []
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];

    if (arg === '-h' || arg === '--help') {
      parsed.help = true;
    } else if (arg === '-o' || arg === '--output') {
      parsed.output = args[++i];
    } else if (arg === '--mode') {
      parsed.mode = args[++i];
    } else if (arg === '--font') {
      parsed.font = args[++i];
    } else if (arg === '--sort') {
      parsed.sort = args[++i];
    } else if (!arg.startsWith('-')) {
      parsed.files.push(arg);
    }
  }

  return parsed;
}

async function main() {
  const args = parseArgs(process.argv.slice(2));

  if (args.help) {
    printUsage();
    process.exit(0);
  }

  if (!args.output) {
    console.error('Error: Output path required (-o or --output)');
    printUsage();
    process.exit(1);
  }

  if (args.files.length === 0) {
    console.error('Error: No input files specified');
    printUsage();
    process.exit(1);
  }

  // Read files
  const files = [];
  for (const filePath of args.files) {
    const path = resolve(filePath);

    try {
      const stat = statSync(path);

      if (!stat.isFile()) {
        console.error(`Error: '${path}' is not a file`);
        process.exit(1);
      }

      if (stat.size > MAX_FILE_SIZE) {
        console.error(
          `Error: File '${path}' exceeds 60KB limit (${stat.size} bytes)`
        );
        process.exit(1);
      }

      const content = readFileSync(path);
      files.push({
        path: filePath,
        content: Array.from(content),
      });

      console.log(`Read: ${filePath} (${stat.size} bytes)`);
    } catch (err) {
      console.error(`Error reading '${path}': ${err.message}`);
      process.exit(1);
    }
  }

  // Build options
  const options = {
    mode: args.mode,
    sort_mode: args.sort,
  };

  if (args.font) {
    options.font = args.font;
  }

  // Validate options
  if (!['auto', 'indexed', 'rgba'].includes(options.mode)) {
    console.error(
      `Error: Invalid mode '${options.mode}'. Must be: auto, indexed, or rgba`
    );
    process.exit(1);
  }

  if (args.font && !['swiss', 'sixth', 'sky', 'monte', 'sugimori', 'mini', 'micro'].includes(args.font)) {
    console.error(
      `Error: Invalid font '${args.font}'. Must be: swiss, sixth, sky, monte, sugimori, mini, or micro`
    );
    process.exit(1);
  }

  if (!['lexicographic', 'reverse', 'by_size', 'by_extension', 'none'].includes(options.sort_mode)) {
    console.error(
      `Error: Invalid sort mode '${options.sort_mode}'`
    );
    process.exit(1);
  }

  // Encode
  console.log('\nEncoding polyglot...');
  try {
    const input = {
      files,
      options,
    };
    const inputJson = JSON.stringify(input);
    const output = encode(inputJson);

    writeFileSync(args.output, Buffer.from(output));

    console.log(`\nSuccess! Polyglot written to: ${args.output}`);
    console.log(`Output size: ${output.length} bytes`);
    console.log(`Files encoded: ${files.length}`);
  } catch (err) {
    console.error(`\nEncoding error: ${err.message}`);
    process.exit(1);
  }
}

main().catch((err) => {
  console.error('Fatal error:', err);
  process.exit(1);
});
