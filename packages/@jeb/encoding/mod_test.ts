import { encode66 } from "./66.ts";
import * as module from "./mod.ts";

import { assertEquals, assertThrows } from "@std/assert";

const utf8 = (text: string) => new TextEncoder().encode(text);

Deno.test("encode66() and decode92() produced expected output", () => {
  for (
    const [input, expected, options] of [
      [
        utf8("hey\nhello\nhi"),
        "~heyCmhl~lloCmhp",
      ],
      [
        utf8("hello world!\ngoodbye world!\n123456789012345678901234567890123"),
        "~helbG8g~worbGQhCmdv~odbeWUg~worbGQhCjEy.10.345678901234567890123456789012......Mw",
      ],
      [
        Uint8Array.from(new Array(256).fill(0).map((_, index) => index)),
        "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4v.3.012345678OTo7PD0-P0BB.8.BCDEFGHIJKLMNOPQRSTUVWXY.....WltcXV5fYGFi.8.cdefghijklmnopqrstuvwxyz.....e3x9fn-AgYKDhIWGh4iJiouMjY6PkJGSk5SVlpeYmZqbnJ2en6ChoqOkpaanqKmqq6ytrq-wsbKztLW2t7i5uru8vb6_wMHCw8TFxsfIycrLzM3Oz9DR0tPU1dbX2Nna29zd3t_g4eLj5OXm5-jp6uvs7e7v8PHy8_T19vf4-fr7_P3-_w",
      ],
      [
        Uint8Array.from(new Array(256).fill(0).map((_, index) => index)),
        "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUm~'()KissLS4v.3.012345678OTo7PD0-P0BB.8.BCDEFGHIJKLMNOPQRSTUVWXY.....WltcXV5fYGFi.8.cdefghijklmnopqrstuvwxyz.....e3x9fn-AgYKDhIWGh4iJiouMjY6PkJGSk5SVlpeYmZqbnJ2en6ChoqOkpaanqKmqq6ytrq-wsbKztLW2t7i5uru8vb6_wMHCw8TFxsfIycrLzM3Oz9DR0tPU1dbX2Nna29zd3t_g4eLj5OXm5-jp6uvs7e7v8PHy8_T19vf4-fr7_P3-_w",
        { extraSafeCharacters: module.SAFE_CHARACTERS.RFC_2396_URL },
      ],
    ] as [Uint8Array, string, undefined | module.EncodeOptions][]
  ) {
    const encoded = module.encode66(input, options);
    assertEquals(encoded, expected, "encoded value didn't match expectation");

    const decoded = module.decode66(encoded, options);
    assertEquals(decoded, input, "round-tripped value didn't match input");
  }
});

Deno.test("decode66() throws an error for non-canonical input iff strict option is set", () => {
  const input = Uint8Array.from(
    new Array(256).fill(0).map((_, index) => index),
  );
  const nonCanonicalEncoding = encode66(input, {
    extraSafeCharacters: module.SAFE_CHARACTERS.RFC_2396_URL,
  });
  module.decode66(nonCanonicalEncoding);
  assertThrows(
    () => module.decode66(nonCanonicalEncoding, { strict: true }),
    module.DecodeError,
  );
});

Deno.test("encode92() and decode92() produce expected output", () => {
  for (
    const [input, expected, options] of [
      [
        utf8("hey\nhello\nhi"),
        "xK#C@~hellzWHe(",
      ],
      [
        utf8("hello world!\ngoodbye world!\n123456789012345678901234567890123"),
        "|3|hello world!3tjo}||dbye wory?aW@|8|12345678901234567890123456789012.....gx",
      ],
      [
        Uint8Array.from(new Array(256).fill(0).map((_, index) => index)),
        "009c61o!#m2NH?C3>iWS5d]J*6CRx17-skh9337xar.{NbQB=+|13|()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[.........tWpVy|7|`abcdefghijklmnopqrstuvwxyz{....E0{D[FpSr8GOteoH(41EJe-<UKDCY&L:dM3N3<zjOsMmzPRn9PQ[%@^ShV!$TGwUeU^7HuW6^uKXvGh.YUh4]Z})[9-kP:p:JqPF+*1CV^9Zp<!yAd4/Xb0k*$*&A&nJXQ<MkK!>&}x#)cTlf[Bu8v].4}L}1:^-@qDS{",
      ],
      [
        Uint8Array.from(new Array(256).fill(0).map((_, index) => index)),
        "009c61o!#m2NH?C3>iWS5d]J*6CRx17-skh9337xar.{N|14|$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[..........tWpVy|7|`abcdefghijklmnopqrstuvwxyz{....E0{D[FpSr8GOteoH(41EJe-<UKDCY&L:dM3N3<zjOsMmzPRn9PQ[%@^ShV!$TGwUeU^7HuW6^uKXvGh.YUh4]Z})[9-kP:p:JqPF+*1CV^9Zp<!yAd4/Xb0k*$*&A&nJXQ<MkK!>&}x#)cTlf[Bu8v].4}L}1:^-@qDS{",
        { extraSafeCharacters: module.SAFE_CHARACTERS.DOUBLE_QUOTE_STRING },
      ],
    ] as [Uint8Array, string, undefined | module.EncodeOptions][]
  ) {
    const encoded = module.encode92(input, options);
    assertEquals(encoded, expected, "encoded value didn't match expectation");

    const decoded = module.decode92(encoded, options);
    assertEquals(decoded, input, "round-tripped value didn't match input");
  }
});

Deno.test("decode92() throws an error for non-canonical input iff strict option is set", () => {
  const input = Uint8Array.from(
    new Array(256).fill(0).map((_, index) => index),
  );
  const nonCanonicalEncoding = encode66(input, {
    extraSafeCharacters: module.SAFE_CHARACTERS.DOUBLE_QUOTE_STRING,
  });
  module.decode92(nonCanonicalEncoding);
  assertThrows(
    () => module.decode92(nonCanonicalEncoding, { strict: true }),
    module.DecodeError,
  );
});
