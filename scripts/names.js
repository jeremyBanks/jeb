#!/usr/bin/env node

import fs from "fs";

import bip39 from "bip39";
import dictionary from "word-list-google";

const nameLengthMin = 12;
const nameLengthMax = 12;
const wordLengthMin = 3;
const wordLengthMax = 6;
const overlapMin = 2;
const overlapMax = 4;

Promise.resolve().then(async function main() {
  const path = "./scripts/names.txt";
  let names;
  try {
    names = fs
      .readFileSync(path, { encoding: "utf8" })
      .split("\n")
      .filter(Boolean);
    const invalidNames = names.filter(
      (name) =>
        !(
          nameLengthMin <= name.length &&
          name.length <= nameLengthMax &&
          name.match(/^[a-z]+/)
        )
    );
    if (invalidNames.length > 0) {
      throw new Error(
        `Existing names were invalid (example: ${invalidNames[0]}).`
      );
    }
    if (names.length <= 1000) {
      throw new Error(`Too few existing names (${names.length}).`);
    }
  } catch (error) {
    console.error("Regenerating names; ", error);
    names = await regenerate();
    fs.writeFileSync(path, names.join("\n"), {
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

const words = [
  ...new Set(
    [
      "vegeta",
      "goku",
      "frieza",
      ...dictionary.englishNoSwears
        .filter((word) => word.length >= 4)
        .slice(0, 2000),
      ...bip39.wordlists.english
    ]
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
  const startGenerate = Date.now();
  const names = [...new Set(makeNames())].sort();
  const elapsedGenerate = (Date.now() - startGenerate) / 1000;

  console.error(
    `Generated ${names.length} names in ${elapsedGenerate.toFixed(1)}s.`
  );

  return names;
}

function* makeNames(prefix = "") {
  if (nameLengthMin <= prefix.length && prefix.length <= nameLengthMax) {
    yield prefix;
  }

  const clampedOverlapMin = Math.min(overlapMin, prefix.length);
  const clampedOverlapMax = Math.max(overlapMax, clampedOverlapMin);

  for (
    let overlap = clampedOverlapMin;
    overlap <= clampedOverlapMax;
    overlap += 1
  ) {
    const overlapping = prefix.slice(-overlap);
    const preOverlapping = prefix.slice(0, -overlap);
    const minWordLength = Math.max(overlap + 1, wordLengthMin);
    const maxWordLength = nameLengthMax - prefix.length + overlap;
    for (const word of words) {
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
