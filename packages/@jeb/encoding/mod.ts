/**
This module provides functions to encode and decode binary data as text using
variants of the [base64url][IETF RFC 4648 S5] and [Z85][ZMQ RFC 32] encodings
which attempt to preserve as much of the original data as ASCII as possible
without negatively affecting intended use cases or affecting the size or
alignment of surrounding data.

These use the encodings as usual for most data, but if one or more contiguous
full blocks (blocks are 3 bytes for base64url, 4 bytes for Z85) of data can be
represented using ASCII characters that are safe for the intended context,
they're instead encoded/preserved as ASCII text, with a prefix indicating the
length of the escaped content (in blocks) and with padding added to the end to
maintain the alignment of subsequent blocks.

__⚠️ warning:__ The current implementation are not very polished. They work
given valid input, but they're probably a lot slower than they need to be. They
can also produces invalid output or throw internal errors instead of throwing
useful errors in cases of invalid input. Consider this implementation to be more
of an example/proof-of-concept for now, and definitely don't use it with
untrusted inputs in a security-relevant context.

### {@link encode66} / {@link decode66} (base64url + 2 special digits)

For use in URLs, base64url is used for encoding, and the length prefix is `~`
for a single escaped block, `..` for two blocks, and `.N.` for 3 to 999 blocks,
where `N` is the number of blocks as a decimal number. Where neccessary (4 or
more contiguous escaped blocks), the blocks are padded with trailing `.`
characters to maintain the original length.

The characters `~` and `.` are chosen because those are the only two characters
that are [unreserved in URIs by RFC3986][IETF RFC 3986 S2.3] which aren't
already used by base64url.

Safe ASCII characters in URLs are limited to those that are allowed by RFC3986:
"uppercase and lowercase letters, decimal digits, hyphen, period, underscore,
and tilde."

```
   input: "hello world!\ngoodbye world!\n123456789012345678901234567890123"
  output: "~helbG8g~worbGQhCmdv~odbeWUg~worbGQhCjEy.10.345678901234567890123456789012......Mw"
(escaped)   hel     wor         odb     wor            345678901234567890123456789012
(encoded)      bG8g    bGQhCmdv    eWUg    bGQhCjEy                                        Mw
```

### {@link encode92} / {@link decode92} (Z85 + 7 special digits)

For use in string literals, Z85 is used for encoding, and the length prefix is
`~` for a single escaped block, `||` for two blocks, and `|N|` for 3 to 9999
blocks, where `N` is the number of blocks as a decimal number.  Where neccessary
(4 or more contiguous escaped blocks), the blocks are padded with trailing `.`
characters to maintain the original length.

The characters `~` and `|` are chosen arbitrarily from the handful of safe
characters that are not already used by Z85.

Safe ASCII characters in any JavaScript string literal type include the space
character and all "printable" ASCII characters except for the backslash, the
double quote, the backtick, and the dollar sign. This extends the typical set
of Z85 digits by adding the comma, semicolon, tilde, bar, underscore, backtick,
and space characters.

```
   input: "hello world!\ngoodbye world!\n123456789012345678901234567890123"
  output: "|3|hello world!3tjo}||dbye wory?aW@|8|12345678901234567890123456789012.....gx"
(escaped)     hello world!       dbye wor        12345678901234567890123456789012
(encoded)                 3tjo}          y?aW@                                        gx
```

This encoding also deviates from the Z85 specification by allowing data that
isn't a multiple of 4 bytes in length.

If other characters are known to be safe in the context where you're using the
encoded data, you can also allow them to be treated as safe by specifying them
in {@linkcode EncodeOptions.extraSafeCharacters} with. By default, the decoding
functions will not verify that escaped characters are actually marked as safe,
and that's left as exclusively an encoding concern. If you require a canonical
encoding (a one-to-one mapping between encoded and decoded values) you can
enforce that by setting {@linkcode DecodeOptions.strict} as well as any other
options you set while encoding.

[ZMQ RFC 32]: https://rfc.zeromq.org/spec/32/
[IETF RFC 4648 S5]: https://datatracker.ietf.org/doc/html/rfc4648#section-5
[IETF RFC 3986 S2.3]: https://datatracker.ietf.org/doc/html/rfc3986#section-2.3

@module */

export * from "./encoding.ts";
export * from "./encodings.ts";
