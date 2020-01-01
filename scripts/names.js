#!/usr/bin/env node

import fs from "fs";
import path from "path";
import url from "url";

import bip39 from "bip39";

let nameLengthMin = 12;
let nameLengthMax = 12;
let wordLengthMin = 3;
let wordLengthMax = 6;
let overlapMin = 2;
let overlapMax = 4;

Promise.resolve().then(async function main() {
  let namesPath = path.join(
    path.dirname(url.fileURLToPath(import.meta.url)),
    "./names.txt"
  );
  let names;
  try {
    names = fs
      .readFileSync(namesPath, { encoding: "utf8" })
      .split("\n")
      .filter(Boolean);
    let invalidNames = names.filter(
      (name) =>
        !(
          nameLengthMin <= name.length &&
          name.length <= nameLengthMax &&
          name.match(/^[a-z]+/)
        )
    );
    if (invalidNames.length > 0) {
      throw new Error(
        `${
          invalidNames.length
        } existing names were invalid (example: ${JSON.stringify(
          invalidNames[0]
        )}).`
      );
    }
    if (names.length < 256) {
      throw new Error(`Too few existing names (${names.length}).`);
    }
  } catch (error) {
    console.error("Regenerating names; ", error);
    names = await regenerate();
    fs.writeFileSync(namesPath, names.join("\n"), {
      encoding: "utf8"
    });
  }

  let n = Number(process.argv[2]);
  if (!Number.isInteger(n)) {
    n = 1;
  }

  for (let i = 0; i < n; i += 1) {
    console.log(choose(names));
  }
});

let words = [
  ...new Set(
    [...bip39.wordlists.english]
      .map((word) => word.toLowerCase())
      .filter(
        (word) =>
          word.match(/^[a-z]+$/) &&
          !word.match(/^([a-z])\1+$/) &&
          wordLengthMin <= word.length &&
          word.length <= wordLengthMax
      )
  )
].sort();

async function regenerate() {
  console.error(
    "Generating all names matching",
    {
      nameLengthMin,
      nameLengthMax,
      wordLengthMin,
      wordLengthMax,
      overlapMin,
      overlapMax
    },
    `using a dictionary of ${words.length} words.`
  );
  let startGenerate = Date.now();
  let names = [...new Set(makeNames())].sort();
  let elapsedGenerate = (Date.now() - startGenerate) / 1000;

  console.error(
    `Generated ${names.length} names in ${elapsedGenerate.toFixed(1)}s.`
  );

  return names;
}

function* makeNames(prefix = "") {
  if (nameLengthMin <= prefix.length && prefix.length <= nameLengthMax) {
    yield prefix;
  }

  let clampedOverlapMin = Math.min(overlapMin, prefix.length);
  let clampedOverlapMax = Math.max(overlapMax, clampedOverlapMin);

  for (
    let overlap = clampedOverlapMin;
    overlap <= clampedOverlapMax;
    overlap += 1
  ) {
    let overlapping = prefix.slice(-overlap);
    let preOverlapping = prefix.slice(0, -overlap);
    let minWordLength = Math.max(overlap + 1, wordLengthMin);
    let maxWordLength = nameLengthMax - prefix.length + overlap;
    for (let word of words) {
      if (
        minWordLength <= word.length &&
        word.length <= maxWordLength &&
        word.startsWith(overlapping)
      ) {
        yield* makeNames(preOverlapping + word);
      }
    }
  }
}

function choose(array) {
  return array[Math.floor(array.length * Math.random())];
}
