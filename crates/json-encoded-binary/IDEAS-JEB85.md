# JEB85: JSON encoded bytes, Z85 extension

**JEB85** is how we refer to the scheme that JEB uses for encoding arbitrary
byte streams as JSON-friendly ASCII strings.

JEB85-encoded strings have two modes: Binary (distinguished with a `\b`/`0x08`
prefix), and Text (everything else). Text strings will be valid UTF-8.

Binary strings, following their `\b` prefix, use our extension of Z85 which uses
`|` to mark arbitrary-sized of embedded raw/unencoded data, while maintaining
the byte alignment for all non-raw chunks in the file. A single raw block is
marked by starting with `|`, while larger blocks are marked with between 1 and 4
Z85 digits, indicating the number of raw blocks - 2, followed by `|`. A block
starting with `||` indicates that the rest of file/stream is raw data (this is a
terminal raw chunk), so a prefix of `\b||` can be used as a full passthrough
(but this output isn't assumed to be UTF-8, so if you emit any non-ASCII bytes
you're getting Latin-1 passthrough, not proper JSON UTF-8 text). Note that
parsing also accepts standard Z85 data if the `\b` prefix is added first.

In JEB, Text mode will be used if the string is valid UTF-8, and doesn't contain
`0x08` or any ASCII control characters other than the three common whitespace
characters (tab, newline, carriage return) — so no `\x00`-`\x08`, `0x0B`-`0x1F`,
or `0x7F` — and if the string is not greater than 64KiB in size.

In JEB, Binary mode uses raw chunks for any chunks that contain only the
standard printable ASCII characters that can be displayed in JSON without any
escape sequences. This preserves as much ASCII text as possible in JSON without
compromising horizontal alignment of encoded data. JEB defaults to a maximum
chunk size of 64KiB when serializing (terminal or non-terminal), but supports
JEB85's maximum when deserializing (unlimited terminal, 208,802,504 bytes (~199
MiB) non-terminal).

Standard Z85 Alphabet:

```
 0 -  9:  0 1 2 3 4 5 6 7 8 9
10 - 19:  a b c d e f g h i j
20 - 29:  k l m n o p q r s t
30 - 39:  u v w x y z A B C D
40 - 49:  E F G H I J K L M N
50 - 59:  O P Q R S T U V W X
60 - 69:  Y Z . - : + = ^ ! /
70 - 79:  * ? & < > ( ) [ ] {
80 - 84:  } @ % $ #
```

---

Z85 means our input is blocks of 4 bytes (32 bits) and our output is blocks of 5
characters (40 bits).

If a block contains no `|`, then it's a normal Z85 block.

```
8ajCn
```

If it contains a `|` in the first position, it's a single raw block.

```
|yes!
```

If it contains a `|` in any later position, then the digits to the left are a
Z85 number indicating the number of raw blocks - 2, so if you had two raw blocks
you could see

```
0|correct!
```

and if you had three raw blocks you could see

```
1|that's right!
```

We may also have an option that only emits single-block raw chunks for maximum
stability.

```
|This| is |stil|l so|mewh|at r|eada|ble.
```

We pad out raw blocks to match the length of encoded blocks, to maintain the
alignment of later encoded data. We pad it using `.` the period character
because `` large blocks of spaces can be awkward to work with in editors, and
`_` underscores can be counted as word characters. The behavior of `.` the
period character in editors is closer to what we want, and there's precedent for
it being used for padding visually.

Z85 requires the last block to be a multiple of 4 bytes, but we do not. We pad
the last block with trailing `0x00`s internally, but truncate the output to the
appropriate length.

If a binary is being converted to JEB and it's going to be displayed in a text
that's ever visible to humans, one may want to split it into chunks of 64 input
bytes, because that's a nice power-of-two that often aligns with data
structures, but it also encodes as 80 characters, a standard text line width.

In JEB, binary data that's been boxed into a JSON entity will be wrapped inside
`,{"":"\b`...`"}`, 10 characters, which would still keep a reasonable baseline
of 90.
