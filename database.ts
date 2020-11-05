#!/usr/bin/env deno run --allow-hrtime --allow-write=. --allow-read=.
import colors from 'https://deno.land/std@0.75.0/fmt/colors.ts';
import SQL from 'https://deno.land/x/lite/mod.ts';

const db = SQL("./data.sqlite");

try {
  await db(SQL`
    create table Numbers (n int64)
  `);
} catch {}

try {
  await Deno.remove('./data.sqlite.vacuum');
} catch{}

db(SQL`vacuum into ${"./data.sqlite.vacuum"}`);
