import * as denoBase64Url from "@std/encoding/base64url";
import * as denoHex from "@std/encoding/hex";
import * as denoAscii85 from "@std/encoding/ascii85";
import * as denoBase32 from "@std/encoding/base32";

import { BASE_32, BASE_64_URL } from "./standards.ts";
import { assertEquals } from "@std/assert";
import { HEX } from "./standards.ts";

const BYTES = [
  "",
  "000000",
  "010000",
  "000010",
  "123456",
  "00",
  "0000",
  "00000000",
  "0000000000",
  "01",
  "0100",
  "01000000",
  "0100000000",
  "10",
  "0010",
  "00000010",
  "0000000010",
  "12",
  "1234",
  "12345678",
  "123456789A",
  "123456789ABC",
  "123456789ABCDE",
  "123456789ABCDEF0",
].map(denoHex.decodeHex);

const LENGTHS = [0, 3, 6, 1, 2, 4, 5, 7, 8, 13, 15];

Deno.test({
  name: "base64url matches Deno standard library",
  fn() {
    for (const bytes of BYTES) {
      const expected = denoBase64Url.encodeBase64Url(bytes);
      const actual = BASE_64_URL.encode(bytes);

      assertEquals(
        actual,
        expected,
        `incorrect encoding for ${denoHex.encodeHex(bytes)}`,
      );
    }
  },
});

Deno.test({
  name: "hex matches Deno standard library",
  fn() {
    for (const bytes of BYTES) {
      const expected = denoHex.encodeHex(bytes);
      const actual = HEX.encode(bytes);

      assertEquals(
        actual,
        expected,
        `incorrect encoding for ${denoHex.encodeHex(bytes)}`,
      );
    }
  },
});

Deno.test({
  name: "Z85 matches Deno standard library",
  fn() {
    for (const bytes of BYTES) {
      const expected = denoAscii85.encodeAscii85(bytes, { standard: "Z85" });
      const actual = HEX.encode(bytes);

      assertEquals(
        actual,
        expected,
        `incorrect encoding for ${denoHex.encodeHex(bytes)}`,
      );
    }
  },
});

Deno.test({
  name: "base32 matches Deno standard library",
  fn() {
    for (const bytes of BYTES) {
      const expected = denoBase32.encodeBase32(bytes);
      const actual = BASE_32.encode(bytes);

      assertEquals(
        actual,
        expected,
        `incorrect encoding for ${denoHex.encodeHex(bytes)}`,
      );
    }
  },
});
