

Here's my understanding of the Extended Z85 encoding format, its constraints, and open questions, written from scratch after reading the design document.

---

## What This Is

This is a design analysis document — not a spec — for an extension to Z85, a binary-to-text encoding used by ZeroMQ. The goal is to define an encoding that is strictly more capable than Z85: it can pass through regions of already-printable bytes verbatim instead of encoding them, saving space and making the output human-readable in those regions.

## The Core Idea

Standard Z85 takes every 4 bytes of input and encodes them as 5 printable ASCII characters, using an alphabet of 85 characters. This works fine for arbitrary binary data, but if your input already contains readable ASCII (like text strings, headers, or structured data), Z85 blindly re-encodes it into an unrecognizable 5-character sequence anyway. That's wasteful: you're paying a 25% size penalty to make already-printable bytes... printable.

Extended Z85 adds a mechanism for the encoder to say "the next N bytes are being passed through as-is." It does this by inserting a special escape character (one that isn't in Z85's 85-character alphabet) to signal a transition from normal Z85 decoding into a raw passthrough mode. The decoder reads the escape, determines how many raw bytes follow (from length information encoded in the escape and/or following characters), copies those bytes directly to output, and then resumes normal Z85 decoding.

The format is a strict superset of Z85: any standard Z85 output is valid Extended Z85 (the encoder just chose not to use any raw sections). But it's not a drop-in replacement — a standard Z85 decoder will reject the escape characters.

## Encoder/Decoder Roles

The design is explicitly asymmetric in terms of intelligence. The encoder is the smart party: it decides when and where to use raw passthrough, based on what's beneficial for a given input. The decoder is simple: it follows deterministic rules to reconstruct the original bytes. The format doesn't need to produce optimal encodings — it just needs to be unambiguous for any valid encoding the encoder might produce.

There's no uniqueness requirement. Multiple valid encodings can exist for the same input (different choices of where to insert raw sections). The encoder picks one; the decoder handles whatever it gets.

## The Position Invariant

This is the most important correctness constraint. Every complete Z85 block that remains Z85-encoded must produce exactly the same characters at exactly the same stream positions as standard Z85 would. Raw sections and their overhead (escape character, length info, etc.) must fit exactly into the character positions that the replaced Z85 blocks would have occupied.

This means the output is always the same length or shorter than standard Z85 for the same input. The savings come from raw sections using N characters instead of ceil(N*5/4).

One important exception: when the encoder cuts a Z85 block partway through (to start or end a raw section mid-block), the partial block's characters may differ from standard Z85 because they're encoding fewer bytes with a different convention. This is acceptable as long as the decoder can unambiguously reconstruct the original bytes.

## Character Budget and Compatibility

Z85 uses 85 of the 95 printable ASCII characters, leaving 10 unused. These are candidates for escape characters. The document ranks them by the compatibility cost of using them:

- **Free to use** (break nothing Z85 doesn't already break): underscore `_` and tilde `~`
- **Minor cost** (breaks markdown inline code spans): backtick
- **Moderate cost** (breaks CSV variants): pipe, comma, semicolon
- **High cost** (breaks JSON, SQL, shell quoting): single quote, double quote, backslash
- **Unusable**: space

The key insight is that cost is about which *contexts* break, not individual characters. If Z85 already breaks shell double-quoting (because it uses `$`), then using backtick doesn't add any shell cost — it just adds a markdown cost. The analysis is about which new contexts become incompatible, not about counting characters.

The previous design iteration used 6 escape characters (the two free ones plus backtick plus the three CSV ones), preserving JSON string compatibility. The current document leaves the exact count as an open question.

## The Padding Budget

When you replace N raw bytes with passthrough, standard Z85 would have used ceil(N*5/4) characters for those bytes. The raw passthrough uses N characters (one per byte). The difference — ceil(N*5/4) - N — is the "budget" available for overhead: the escape character, length encoding, disambiguation information, and any padding.

At the minimum interesting size of 4 raw bytes, the budget is exactly 1 character. That's just enough for the escape character itself, with nothing left for length encoding (so the length must be implied by which escape character was chosen). There are zero net character savings at this size — the benefit is purely that the raw bytes are readable.

At 5-8 bytes, budget is 2. At 9-12, budget is 3. It grows roughly as N/4 for large N.

This budget analysis drives many of the format's constraints. Mid-block cuts (starting or ending a raw section partway through a Z85 block) require disambiguation information, which costs budget. So at tight budgets, you can only do block-aligned raw sections.

## Mid-Block Boundaries: Entry and Exit

This is the most technically intricate part of the design. The encoder doesn't have to start and end raw sections at Z85's 4-byte block boundaries — it can cut mid-block. But this creates a problem: the partial Z85 characters for the remaining bytes in that block may be ambiguous.

**Entry boundaries** (where you leave Z85 mode and enter raw mode): You're cutting after K bytes of a 4-byte block, emitting the leading K+1 Z85 characters for those known bytes. But Z85's arithmetic means the leading characters depend on *all* bytes in the block, not just the first K. For a 1-byte cut, 68% of byte values produce a stable (unambiguous) leading character. For 2-byte cuts it's 89%, for 3-byte cuts 96%. When unstable, you need about 2 bits of disambiguation per boundary byte, which must come from the escape character's information capacity.

The stability percentages come from Z85's big-endian arithmetic. The leading base-85 digit is floor(V / 85^4) where V is the 4-byte value. When the first byte's range of possible V values (determined by the unknown low bytes) doesn't cross a 85^4 boundary, the digit is stable. 174 of 256 byte values avoid such crossings, giving 68%.

**Exit boundaries** (where you leave raw mode and return to Z85): The analysis is structurally similar but there's a critical asymmetry. At an exit boundary, the bytes *before* the cut were part of the raw section — the decoder already knows them. Due to a convenient property of Z85's modular arithmetic (all powers of 256 are congruent to 1 mod 85), the trailing Z85 digit is just the sum of all four bytes mod 85. The decoder can subtract the known raw bytes and solve for the unknown boundary bytes. This disambiguation is "free" — it doesn't consume any of the escape character's information budget. It just costs decoder complexity (back-referencing recently decoded raw bytes).

So entry boundaries are expensive (disambiguation costs escape budget) and exit boundaries are cheap (disambiguation comes from already-known data). This motivates asymmetric conventions: entry defaults to big-endian (leading characters), exit defaults to little-endian (trailing characters) disambiguated from raw context.

**Rejected alternative**: Instead of using Z85's natural partial characters and disambiguation, you could directly encode K boundary bytes as K+1 Z85 characters with a dedicated bijection (100% stable, no disambiguation needed). But this costs an extra character per boundary, and at tight budgets (like 2 characters for an 8-byte raw section), the natural approach wins because you can pack disambiguation into the escape character's information bits more efficiently than spending a whole Z85 character (~6.4 bits capacity) on it.

## Escape Character Information Theory

Each escape character the format uses provides log2(N) bits of free information at the point of escape, where N is the number of distinct escape characters. These bits can be multiplexed across multiple purposes: signaling endianness convention, length class, disambiguation for mid-block cuts, etc.

With the 1 overhead character from the budget (an 85-value Z85 character, ~6.4 bits), the total information capacity at a raw section header is N × 85 combinations.

The document works through the 2-block (8 byte) case as the tightest interesting scenario. With mid-block cuts at both boundaries, you need to encode 4 entry positions × 4 exit positions × up to 4 disambiguation candidates = 64 combinations in the worst case, leaving the remaining capacity for length encoding. With 2 escape characters (both free-tier), you get 170 combinations, which after the 64-way worst case leaves room for about 2 length classes. Block-aligned-only is much simpler: all N×85 combinations encode length.

## Resolved Decisions

Several questions have been settled:

- **Minimum raw section: 4 bytes.** Even though there are zero size savings at this length, the transparency value (readable bytes) justifies it.
- **Mid-block cuts: supported at both entry and exit.** The encoder is opportunistic.
- **High complexity is acceptable.** The project values format properties over implementation simplicity.
- **Asymmetric entry/exit conventions: justified.** Not because of stability differences (both sides have similar stability), but because exit disambiguation is free (uses raw context) while entry disambiguation is costly (uses escape budget).
- **Length-before-data, no sentinels.** The decoder must know the raw section length before reading raw bytes. Special case: a "raw to end of stream" escape where length is implicit.

## Open Questions

Several significant design decisions remain:

1. **How many escape characters?** Anywhere from 1 (just underscore, zero compatibility cost) to 6 (through the CSV tier). The right number depends on how much information capacity is needed at escape points.

2. **How to allocate the information bits?** The log2(N) bits from escape character choice can be split across endianness signaling, length classes, and disambiguation. The optimal split is undetermined.

3. **Layout of the raw section header.** The ordering and placement of escape character, length information, disambiguation bits, and padding characters within the overhead space is unspecified.

4. **Stream boundary edge cases.** Does the entry/exit analysis change at the very start or end of the stream where there's no adjacent Z85 block?

5. **Consecutive raw sections.** Can two raw sections be adjacent with no Z85 blocks between them? If so, what separates them?

## Philosophical Motivation

The document explicitly connects this work to a broader design philosophy of "transparency" — making binary data legible within its container format. It draws a parallel to another project (zipng) that embeds ZIP archives inside PNG images. In both cases, the goal is human readability at the cost of format complexity, which the author considers a worthwhile trade.

## Things I Found Unclear or Potentially Ambiguous

1. **The "position invariant" and mid-block cuts coexist uneasily.** The invariant says retained Z85 blocks must occupy the same positions as in standard Z85. Mid-block cuts produce partial blocks whose characters differ from standard Z85. The document says partial blocks are "a separate case, not an exception." I understand what's meant — the invariant applies to *complete* blocks only — but the phrasing could trip someone up, because the partial block characters do occupy positions that standard Z85 would have filled with different values.

2. **The 68% stability derivation.** The document says 174 of 256 byte values map to exactly one leading digit and 82 straddle a boundary. It explains the mechanism (crossing a 85^4 boundary) but doesn't show the actual counting. Taking it on faith, though the number is plausible: 256 / (85^4 / 2^24) ≈ 256 / 3.11 ≈ 82 boundary crossings, which checks out.

3. **Exit disambiguation "from raw context."** The document says this is "free" in terms of escape budget but costs decoder complexity. It's not entirely clear how much complexity. The decoder needs to remember the last few raw bytes and perform modular arithmetic — straightforward, but it means the decoder isn't purely streaming in the simplest sense (it needs a small lookback buffer). The document acknowledges this but could be more explicit about the exact lookback requirements.

4. **"Raw to end of stream" semantics.** This is described as a special case of length-before-data, but it's quite different operationally. The decoder doesn't know the byte count — it reads until EOF. This works for the last raw section in a stream but seems to preclude any trailing Z85 data. It's unclear whether this is intended only for streams that end with raw data, or if there's a mechanism to resume Z85 afterward.

5. **What bytes are allowed in raw sections.** The document says the decoder imposes no restriction — it passes through whatever bytes the length says to read. But the encoder decides what to include based on "compatibility profile." This is a policy distinction, not a format distinction, but it means that in principle a raw section could contain non-printable bytes, which would undermine the transparency goal. The document frames this as an encoder decision, which is reasonable, but the format-level permissiveness is worth noting.

6. **Multiple raw sections and their interaction.** The document says each raw section is independent, but doesn't discuss what happens when two raw sections are close together — specifically whether their budget calculations interact, or whether partial blocks from one section's exit can overlap with another section's entry. The open question about consecutive raw sections (Q6) acknowledges this gap.
