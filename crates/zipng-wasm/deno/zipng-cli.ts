#!/usr/bin/env -S deno run --allow-read --allow-write
/**
 * Deno CLI for zipng polyglot encoder.
 *
 * Usage:
 *   deno run --allow-read --allow-write zipng-cli.ts -o output.png file1.txt file2.txt
 *   deno run --allow-read --allow-write zipng-cli.ts --mode rgba --font swiss -o out.png *.txt
 */

import { encode, type EncodeOptions, type FileInput } from "./lib.ts";
import { parse } from "https://deno.land/std@0.224.0/flags/mod.ts";

const MAX_FILE_SIZE = 60 * 1024; // 60KB

interface CliArgs {
  o?: string;
  output?: string;
  mode?: "auto" | "indexed" | "rgba";
  font?: "swiss" | "sixth" | "sky" | "monte" | "sugimori" | "mini" | "micro";
  sort?:
    | "lexicographic"
    | "reverse"
    | "by_size"
    | "by_extension"
    | "none";
  h?: boolean;
  help?: boolean;
  _: string[];
}

function printUsage() {
  console.log(`
zipng-cli - Encode files into PNG+ZIP polyglots

Usage:
  zipng-cli [OPTIONS] -o OUTPUT FILES...

Options:
  -o, --output PATH         Output PNG file path (required)
  --mode MODE               Color mode: auto, indexed, rgba (default: auto)
  --font FONT               Font: swiss, sixth, sky, monte, sugimori, mini, micro
  --sort MODE               Sort: lexicographic, reverse, by_size, by_extension, none (default: lexicographic)
  -h, --help                Show this help

Examples:
  # Basic usage
  zipng-cli -o archive.png file1.txt file2.txt

  # With options
  zipng-cli --mode indexed --font swiss --sort reverse -o out.png *.txt

  # Preserve insertion order
  zipng-cli --sort none -o out.png important.txt readme.txt data.bin
`);
}

async function main() {
  const args = parse(Deno.args, {
    string: ["o", "output", "mode", "font", "sort"],
    boolean: ["h", "help"],
    alias: {
      o: "output",
      h: "help",
    },
  }) as CliArgs;

  if (args.help || args.h) {
    printUsage();
    Deno.exit(0);
  }

  const outputPath = args.output || args.o;
  if (!outputPath) {
    console.error("Error: Output path required (-o or --output)");
    printUsage();
    Deno.exit(1);
  }

  const inputFiles = args._;
  if (inputFiles.length === 0) {
    console.error("Error: No input files specified");
    printUsage();
    Deno.exit(1);
  }

  // Read files
  const files: FileInput[] = [];
  for (const filePath of inputFiles) {
    const path = filePath.toString();

    try {
      const stat = await Deno.stat(path);

      if (!stat.isFile) {
        console.error(`Error: '${path}' is not a file`);
        Deno.exit(1);
      }

      if (stat.size > MAX_FILE_SIZE) {
        console.error(
          `Error: File '${path}' exceeds 60KB limit (${stat.size} bytes)`
        );
        Deno.exit(1);
      }

      const content = await Deno.readFile(path);
      files.push({
        path,
        content: Array.from(content),
      });

      console.log(`Read: ${path} (${stat.size} bytes)`);
    } catch (err) {
      console.error(`Error reading '${path}': ${err.message}`);
      Deno.exit(1);
    }
  }

  // Build options
  const options: EncodeOptions = {};

  if (args.mode) {
    if (!["auto", "indexed", "rgba"].includes(args.mode)) {
      console.error(
        `Error: Invalid mode '${args.mode}'. Must be: auto, indexed, or rgba`
      );
      Deno.exit(1);
    }
    options.mode = args.mode;
  }

  if (args.font) {
    if (
      !["swiss", "sixth", "sky", "monte", "sugimori", "mini", "micro"].includes(
        args.font
      )
    ) {
      console.error(
        `Error: Invalid font '${args.font}'. Must be: swiss, sixth, sky, monte, sugimori, mini, or micro`
      );
      Deno.exit(1);
    }
    options.font = args.font;
  }

  if (args.sort) {
    if (
      !["lexicographic", "reverse", "by_size", "by_extension", "none"].includes(
        args.sort
      )
    ) {
      console.error(
        `Error: Invalid sort mode '${args.sort}'. Must be: lexicographic, reverse, by_size, by_extension, or none`
      );
      Deno.exit(1);
    }
    options.sort_mode = args.sort;
  }

  // Encode
  console.log("\nEncoding polyglot...");
  try {
    const output = encode(files, options);
    await Deno.writeFile(outputPath, output);

    console.log(`\nSuccess! Polyglot written to: ${outputPath}`);
    console.log(`Output size: ${output.length} bytes`);
    console.log(`Files encoded: ${files.length}`);
  } catch (err) {
    console.error(`\nEncoding error: ${err.message}`);
    Deno.exit(1);
  }
}

if (import.meta.main) {
  main();
}
