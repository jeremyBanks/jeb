# Extended Z85 — Design Constraints & Analysis

*Living document. Captures the invariants, constraints, and analytical findings
that will inform the final design. Not a specification — a foundation for one.*

## 0. Assumptions

These are stated up front to avoid burying them in later sections.

**Motivation.** Z85 encodes arbitrary binary data as printable ASCII, expanding
4 bytes into 5 characters (~25% overhead). When the input already contains
printable ASCII (text strings, human-readable headers, structured data), that
overhead is pure waste — the bytes are already printable. This design extends
Z85 with escape sequences that let the encoder pass those bytes through raw,
eliminating the overhead for text-heavy regions.

**Encoder/decoder asymmetry.** The encoder is smart; the decoder is simple. The
encoder decides opportunistically when raw passthrough is beneficial. The decoder
just follows unambiguous rules to reconstruct the original bytes. The format
doesn't need to be optimal for all inputs — just unambiguous for any valid
encoding. Implementation complexity (encoder or decoder) is not a significant
concern — it's an afterthought relative to the format properties themselves.
We care about what the format *is*, not how hard it is to implement.

**Self-signaling.** The presence of non-Z85 characters (escape chars) in the
output self-signals that this is extended Z85, not standard Z85. A standard Z85
decoder will reject the escape characters as invalid. This is acceptable —
extended Z85 is a superset, not a drop-in replacement. Conversely, standard Z85
output (no escape characters) is valid extended Z85 — the encoder simply chose
not to use any raw sections.

**Data distribution.** Stability percentages in §5 and §6 assume uniformly
random byte values. Real data (small integers, ASCII-adjacent values, structured
headers) will often have higher stability rates. The uniform-random case
represents a conservative baseline, not a worst case — adversarial input could
maximize unstable boundaries, but the encoder can always avoid unstable cuts. The percentages describe "what fraction of
byte values allow a stable cut at this position."

**Block alignment.** "Block-aligned" always means aligned to Z85's 4-byte /
5-character block boundaries relative to the **start of the Z85 stream**, not
relative to the raw section or any other reference point.

**Raw byte order.** Bytes within raw sections appear in their original
sequential order, matching the input stream. Endianness discussion (§6) applies
only to how partial Z85 blocks encode boundary bytes, not to raw data ordering.

**Raw byte values.** The decoder imposes no restriction on what bytes appear in
a raw section — it knows the length from the prefix and passes bytes through
without validation. The *encoder* decides which bytes to include based on the
desired compatibility profile: by default, any byte that wouldn't change the
output's compatibility characteristics (Z85 alphabet characters, escape
characters, and other printable ASCII that the output already uses). The encoder
may allow user overrides for specialized use cases. This is an encoder policy
decision, not a format constraint.

**Multiple raw sections.** A single encoded stream can contain multiple raw
sections interleaved with standard Z85 blocks. Each raw section is independent
(its own escape, length, boundary handling). There is no limit on the number
of raw sections per stream.

**Forward-only decoding.** The decoder processes the stream left to right in a
single pass. It does not need to look ahead past the current raw section's
prefix to determine length or boundaries. (It *may* reference recently decoded
bytes — see §6 exit disambiguation — but never needs to scan forward.)

## 1. Mental Model

**Z85 background:** Z85 ([ZeroMQ RFC 32](https://rfc.zeromq.org/spec/32/)) is a
binary-to-text encoding using 85 printable ASCII characters. It processes input
in 4-byte blocks: each block is interpreted as a big-endian u32, then divided
into 5 base-85 digits (most-significant first), each mapped to a character in
the Z85 alphabet. This is a 4:5 expansion (~25% overhead). The original ZeroMQ
spec requires input length to be a multiple of 4 bytes; we lift that restriction
— arbitrary input lengths are supported, with partial final blocks handled the
same way as mid-block boundary cuts (see §5-6).

Extended Z85 is this same encoding with one addition: the encoder can
opportunistically replace runs of Z85 blocks with an escape character followed
by the raw input bytes themselves. The raw bytes pass through unencoded,
eliminating overhead for regions that are already printable.

### Worked Example

Consider encoding 12 input bytes where the middle 4 are printable ASCII:

```
Input bytes:  [0xDE 0xAD 0xBE 0xEF] [0x48 0x65 0x6C 0x6C] [0x01 0x02 0x03 0x04]
                (binary data)          "Hell"                  (binary data)
```

**Standard Z85** encodes all 12 bytes as 15 characters (3 blocks × 5 chars):

```
Standard:     rZUgH  4erBi  0sjjE       (15 characters, no raw visibility)
Position:     01234  56789  ABCDE
```

**Extended Z85** could pass the middle block through raw, saving 1 character:

```
Extended:     rZUgH  _Hell  0sjjE       (15 characters: 5+1+4+5)
Position:     01234  5 6789  ABCDE
```

Here `_` is the escape character signaling "the next 4 bytes are raw." The
Z85 blocks at positions 0-4 and 10-14 are **identical** to standard Z85 (the
position invariant). The escape + raw bytes replace the 5 characters that
block 1 would have occupied, using 5 characters (1 escape + 4 raw) — zero net
savings at this size, but "Hell" is now readable. With longer raw sections,
the savings grow: N raw bytes use N+overhead vs ⌈N×5/4⌉ standard.

*(This example uses block-aligned boundaries for simplicity. Mid-block
transitions — cutting partway through a Z85 block — are analyzed in §5-6.)*

## 2. Priority Order

### P1: Correctness & The Position Invariant

Every **complete Z85 block** that remains Z85-encoded must produce the exact
same characters at the exact same positions (relative to the start of the
standard Z85 encoding) as it would in a standard Z85 encoding of the same
input. Raw sections replace Z85 blocks — the escape + raw bytes + overhead
consume exactly the character positions those blocks would have occupied.

This means:
- Output length is always **≤ standard Z85 length** for the same input (the
  savings come from raw sections using N characters instead of `⌈N×5/4⌉`)
- Each retained Z85 block is byte-for-byte identical to its standard Z85
  encoding and occupies the same position in the original layout

This is a **constraint the encoder must satisfy**. Any encoding that violates
position invariance is invalid, regardless of whether the decoder could
reconstruct the bytes.

**Partial blocks at boundaries:** When the encoder cuts a Z85 block partway
through to begin or end a raw section, the partial block's Z85 characters may
differ from what standard Z85 would produce (because they encode fewer bytes
using a different convention). This is acceptable as long as the decoder can
unambiguously reconstruct the original bytes. Partial blocks are a separate
case from the position invariant above, not an exception to it.

### P2: Context Compatibility

The characters we use (Z85 alphabet + escape characters) determine where
encoded data can be used without additional escaping. This isn't just "minimize
character count" — some contexts break with *any* character from a group, so
using one more from that group costs nothing.

See §4 for the full analysis.

### P3: Transparency / Readability

How much of the original data is visible in the encoded output. Quality matters
more than quantity:
- **Aligned data is more useful** — bytes at structural boundaries (word
  boundaries, field starts) are more informative than random mid-structure bytes
- **Alignment preference for tiebreaking:** When choosing between escape
  placement options, prefer the one that produces more structurally aligned raw
  data (more trailing zero bits → more likely to be at a meaningful boundary)
- This is subjective and heuristic — a tiebreaker, not a hard rule

## 3. Escape Detection & Decoder Constraints

A non-Z85 escape character signals the transition from standard Z85 decoding to
raw passthrough. The escape character must appear within the character positions
of the Z85 block(s) being replaced — practically, within ~5 characters of the
transition point. (This is a design target, not yet a hard specification. The
exact bound depends on layout decisions in §11 Q4.)

The decoder must know the raw section length **before reading raw bytes** (see
§10). No scanning or sentinel detection.

Raw sections can be **any byte length** (not constrained to multiples of 4 bytes
or 5 characters). The encoding is opportunistic — if a given length at a given
alignment would cause problems (insufficient budget — see §7 — or unstable
boundary), the encoder simply doesn't use it.

## 4. Character Compatibility Analysis

### Z85 Alphabet (85 characters)

```
0123456789abcdefghijklmnopqrstuvwxyz
ABCDEFGHIJKLMNOPQRSTUVWXYZ
.-:+=^!/*?&<>()[]{}@%$#
```

### The 10 Non-Z85 Printable ASCII Characters

Ranked by the cost of using them as escape characters:

**Tier 1 — Completely free** (break no context Z85 doesn't already break):
- `_` (underscore)
- `~` (tilde)

**Tier 1.5 — Breaks markdown inline code only:**
- `` ` `` (backtick) — Z85's `$` already breaks shell double-quote (both do
  command substitution), but backtick also terminates `` `code` `` spans in
  markdown. Workaround exists (double-backtick delimiters) but it's a real
  compatibility regression.

**Tier 2 — Break CSV variants:**
- `|` (pipe) — breaks `|`-delimited files
- `,` (comma) — breaks standard CSV (RFC 4180)
- `;` (semicolon) — breaks European CSV (`;` as separator)

**Tier 3 — Break important contexts:**
- `'` — breaks shell single-quote strings, SQL single-quote strings
- `"` — breaks JSON strings, TOML basic strings, CSV quoting
- `\` — breaks JSON strings, TOML basic strings
- ` ` (space) — breaks word splitting everywhere, visual alignment. Unusable.

### Context Compatibility Summary

| Context | Z85 already breaks? | Additional break characters |
|---------|---------------------|---------------------------|
| JSON double-quoted strings | **NO** | `"` `\` |
| Shell single-quoted strings | **NO** | `'` |
| CSV unquoted fields | **NO** | `,` `"` (`;` `|` in variants) |
| TOML basic strings | **NO** | `"` `\` |
| SQL single-quoted strings | **NO** | `'` (`\` in some dialects) |
| URL components | YES | (free) |
| HTML/XML text | YES | (free) |
| Shell unquoted args | YES | (free) |
| Shell double-quoted strings | YES | (free — `$` already breaks) |
| YAML unquoted scalars | YES | (free) |
| Regex | YES | (free) |
| Markdown | YES | (free — but `` ` `` adds inline code) |
| Filenames | YES | (free) |

### Key Insight

The previous design (IDEATION.md) used all 6 characters from Tiers 1-2:
`_` `~` `` ` `` `|` `,` `;`. This preserves JSON string compatibility — likely
the most important context for jeb. This constraints document leaves the escape
character count as an open design decision (§11 Q1). The number of escape
characters and how their information capacity is allocated is central to the
design (see §8).

## 5. Mid-Block Stability (Entry Boundaries)

This section analyzes **entry boundaries** — where the encoder cuts a Z85 block
to begin a raw section, emitting partial Z85 characters for the bytes before
the cut. Exit boundaries are analyzed separately in §6.

When cutting a Z85 block after K known bytes, the K+1 leading Z85 characters
may or may not be "stable" (uniquely determined regardless of the unknown
bytes' values).

**Definition:** Stability = the fraction of byte values (for the known bytes)
where the leading Z85 characters are the same for ALL possible values of the
remaining unknown bytes. In other words: positions where the encoder can emit
partial Z85 characters without needing disambiguation.

| Cut after K bytes | Characters emitted | Stable (%) | Disambiguation bits when not |
|-------------------|-------------------|------------|------------------------------|
| 1 | 1 leading character | 68.0% | ~2 bits (3-4 candidates) |
| 2 | 2 leading characters | 89.3% | ~4 bits (~9-10 candidates) |
| 3 | 3 leading characters | 96.4% | ~5 bits (~27-28 candidates) |

### Why These Numbers

Z85 encodes 4-byte blocks as big-endian u32 (`V`), then extracts base-85 digits
most-significant first. The leading digit `c0 = V / 85^4` depends primarily on
the first byte. Stability fails when the unknown low bytes could push V across
a `85^4` boundary (where `85^4 = 52,200,625`).

For a 1-byte entry cut, the first byte `b0` maps to a range
`[b0 × 2^24, (b0+1) × 2^24 - 1]` of width `2^24 = 16,777,216`. This range
spans at most one `85^4` boundary. When it doesn't cross a boundary, the
leading character is stable.

Of the 85 possible leading-digit values (0–84), the number that each `b0` maps
to determines stability:
- If `b0` maps to exactly 1 leading digit: stable (68% of byte values)
- If `b0` straddles a boundary → 2 possible leading digits: unstable, need
  ~2 bits to disambiguate which one

Of 256 possible byte values for `b0`, 174 map to exactly 1 leading digit
(stable) and 82 straddle a `85^4` boundary (unstable): 174/256 ≈ 68%.

For 2-byte and 3-byte cuts, the same logic applies at finer granularity, with
stability improving because more bits are known.

## 6. Entry vs Exit Boundary Analysis

### The Key Asymmetry

Entry and exit boundaries have the **same cost structure** (~2 bits of
disambiguation per boundary byte) but **different disambiguation sources** due to
the position of known bytes within the Z85 block. This motivates asymmetric
default conventions.

- **Entry boundary:** Known bytes are at the START of the block (high-order in
  Z85's big-endian convention). Information lives in the **leading** Z85
  characters.
- **Exit boundary:** Known bytes are at the END of the block (low-order).
  Information lives in the **trailing** Z85 characters.

### Stability by Position

**Entry (prefix bytes → leading characters):**

| Bytes known | Characters emitted | Disambiguation needed | Naturally stable |
|------------|-------------------|----------------------|-----------------|
| 1 | 1 leading (char 0) | ~2 bits | 68% |
| 2 | 2 leading (chars 0-1) | ~4 bits | 89% |
| 3 | 3 leading (chars 0-2) | ~5 bits | 96% |

**Exit (suffix bytes → trailing characters):**

| Bytes known | Characters emitted | Disambiguation needed | Source |
|------------|-------------------|----------------------|--------|
| 1 | 1 trailing (char 4) | ~2 bits | Raw context (free) |
| 2 | 2 trailing (chars 3-4) | ~4 bits | Raw context (free) |
| 3 | 3 trailing (chars 2-4) | ~5 bits | Raw context (free) |

### Exit Disambiguation Is Free (From Raw Context)

At an **exit boundary**, the block looks like `[raw raw raw | Z85]`. The
bytes before the cut were part of the raw section — the decoder already has
them. To decode the trailing Z85 character(s) for the remaining bytes, the
decoder substitutes the known raw bytes into the block's Z85 arithmetic and
solves for the boundary bytes.

**Example (1-byte exit):** Block is `[b0 b1 b2 | b3]` where b0-b2 were raw.
The trailing character encodes `V mod 85` where `V = b0×2^24 + b1×2^16 +
b2×2^8 + b3`. A key property of Z85's arithmetic:

> **`2^8 ≡ 2^16 ≡ 2^24 ≡ 1 (mod 85)`**
>
> Since `gcd(256, 85) = 1`, all powers of 256 are congruent to 1 mod 85.
> This means `V mod 85 = (b0 + b1 + b2 + b3) mod 85` — the trailing digit
> is a simple sum of all four bytes, regardless of position.

The decoder knows b0, b1, b2 (from raw), and knows the character value, so it
computes `b3 mod 85 = (char4_value - b0 - b1 - b2) mod 85`. Since b3 has 256
possible values and mod 85 gives ~3 candidates per residue, the decoder needs
~2 bits to pick the right one — but these bits can come from the raw context
rather than from escape character info bits.

**Contrast with entry boundaries:** At an entry, the block is
`[Z85 | raw raw raw]`. The bytes after the cut are raw, but the decoder hasn't
read them yet. The decoder must resolve disambiguation from the escape prefix
(costly — uses limited escape char info bits).

**Key asymmetry:**
- **Entry:** ~2 bits disambiguation per boundary byte, must come from escape
  prefix (costly)
- **Exit:** ~2 bits disambiguation per boundary byte, can come from
  already-decoded raw bytes (free in terms of escape budget, costs decoder
  complexity)

Exit boundaries are **cheaper** than entry boundaries in escape bit budget.
The cost is decoder complexity (back-referencing recently decoded raw data).
This is exactly the kind of complexity this project is willing to accept (§10).

### Recommended Defaults

When the encoding has no budget to signal which convention is in use (e.g.,
tight budget with only 1 escape character), these defaults apply:

**Entry: BE (leading characters)** — The byte just before a raw section tends to
be a small value (length field, type tag, null terminator). Small values have
zeros in high bits, placing them far from `85^4` boundaries → higher stability.
68% baseline, likely higher for real data.

**Exit: Trailing characters (LE), disambiguated from raw context** — The
decoder already has the raw bytes from earlier in the block. It uses them to
resolve the trailing Z85 characters without spending escape char info bits.
This makes exit boundaries cheaper than entry boundaries in terms of escape
budget.

**The asymmetry:**
- Entry: ~2 bits/byte disambiguation from escape prefix (expensive, uses
  limited info budget)
- Exit: ~2 bits/byte disambiguation from already-decoded raw bytes (free in
  escape budget, costs decoder complexity)

When budget allows, the escape character choice can signal which convention
is in use (see §8).

### Rejected Alternative: Direct Byte Encoding

Instead of using Z85's natural leading/trailing characters for boundary bytes,
we could encode them directly: map K bytes → K+1 Z85 characters via a bijection
independent of the block's other bytes. This gives 100% stability (no
disambiguation needed) at the cost of 1 extra character per boundary.

| Approach | Characters used | Disambiguation cost | Total cost |
|----------|----------------|--------------------|-----------| 
| Natural (leading/trailing) | K | ~2 bits/byte from escape char info | K characters + borrowed info bits |
| Direct encoding | K+1 | 0 | K+1 characters |

**Natural wins for tight budgets.** At budget=2 (8 raw bytes), natural uses 1
character + 1 escape = 2 characters of budget, with disambiguation packed into
the escape character's information bits. Direct encoding would need 2 characters
+ 1 escape = 3 characters — doesn't fit.

The key insight: each escape character choice provides `log2(N)` bits of free
information (where N is the number of distinct escape characters). Those bits
can carry disambiguation more efficiently than spending a full extra Z85
character (~6.4 bits of capacity) on it.

## 7. Padding Budget

**Budget** = `⌈N × 5/4⌉ - N`: the number of Z85 characters that standard
encoding would use for N bytes, minus the N characters consumed by raw
passthrough. This budget must cover: escape character(s), length encoding,
disambiguation bits, and any padding.

| Raw bytes | Standard Z85 characters | Budget | Notes |
|-----------|------------------------|--------|-------|
| 4 | 5 | 1 | Escape only. No mid-block cuts. |
| 5-8 | 7-10 | 2 | Escape + 1 character |
| 9-12 | 12-15 | 3 | Comfortable for most cases |
| 13-16 | 17-20 | 4 | Comfortable for all cases |
| N (large) | ~5N/4 | ~N/4 | Abundant |

### Break-Even Analysis

At 4 raw bytes: standard Z85 = 5 characters, raw passthrough = 4 bytes + 1
escape = 5 characters. **Zero net savings.** The benefit is transparency (raw
bytes are readable), not compactness. Net character savings begin at 5+ raw
bytes (budget ≥ 2, with at least 1 character saved after escape overhead).

For larger raw sections, savings grow linearly: N raw bytes save approximately
`N/4 - overhead` characters compared to standard Z85.

### Budget Requirements for Mid-Block Transitions

| Configuration | Minimum budget needed |
|--------------|---------------------|
| Block-aligned, no explicit length | 1 (escape only) |
| Block-aligned, with length | 2 (escape + length character) |
| Mid-block entry OR exit | 2 (escape + disambiguation) |
| Mid-block both + length | 4 (escape + length + 2× disambiguation) |

Note: "block-aligned, no explicit length" at budget=1 requires the escape
character alone to imply the raw section length. This is only possible if a
specific escape character is dedicated to a specific length (spending one of the
available escape characters on this case).

### Critical Thresholds

- **4 bytes (budget=1):** Block-aligned raw only. No mid-block cuts. Length
  must be implied by escape character choice.
- **5-8 bytes (budget=2):** Mid-block cut at ONE boundary possible. Tight.
- **9-12 bytes (budget=3):** Mid-block at both boundaries feasible.
- **13+ bytes (budget=4+):** Everything works comfortably.

## 8. The Escape Character Budget

Each escape character beyond Z85's 85 is a precious resource. It costs context
compatibility (§4) and provides information at critical decision points.

The choice of which escape character to use at a given point provides
`log2(N)` bits of information, where N is the total number of escape characters.
These bits can be **multiplexed** across different purposes — they aren't
limited to one use per character. For example, with 4 escape characters (2 bits):
the high bit could signal endianness while the low bit signals a length class.

**What escape character info bits could encode** (can be combined):
- Endianness of boundary blocks (BE vs LE)
- Length class (short vs long raw sections)
- Entry/exit boundary convention
- Part of disambiguation information for mid-block cuts

| Escape characters | Info bits (`log2(N)`) | Example allocation |
|------------------|----------------------|-------------------|
| 1 | 0 | Fixed conventions, length implicit |
| 2 | 1 | BE/LE, OR 2 length classes |
| 3 | ~1.6 | 3-way split (e.g., short/long/to-end) |
| 4 | 2 | Endianness × length class |
| 6 | ~2.6 | Full IDEATION.md design |

The number and allocation of escape characters is **undecided**. The analysis
above maps the constraint space; the choice depends on which capabilities matter
most for real-world data.

### Information Flow Constraint

For escape character info bits to carry disambiguation, the decoder must
encounter the escape character **before** it needs to decode the boundary block.
This constrains the layout: the escape character must precede (or be part of)
the prefix that appears before raw data. This is compatible with the length-
before-data constraint (§10) — both require a prefix-oriented layout.

## 9. Concrete Allocation Analysis (2-Block Case)

The 2-block case (8 input bytes = 10 Z85 characters) is the minimum
interesting case for mid-block transitions and the tightest budget scenario
worth analyzing in detail.

### Budget

For any 2-block raw section, the budget is always **2 characters** (1 escape
+ 1 overhead), regardless of where the entry/exit cuts fall. Mid-block cuts
reduce the number of raw bytes but don't reduce budget — the partial Z85
characters occupy positions that would otherwise be raw.

| Configuration | Raw bytes | Partial Z85 characters | Budget |
|--------------|-----------|----------------------|--------|
| Block-aligned both | 8 | 0 | 2 |
| Mid-block entry only | 7 | 1 (leading) | 2 |
| Mid-block exit only | 7 | 1 (trailing) | 2 |
| Mid-block both | 6 | 2 | 2 |

### What the Decoder Needs

| Information | Options | Bits |
|------------|---------|------|
| Entry cut position | 4 (aligned, after 1/2/3 bytes) | 2 |
| Exit cut position | 4 (aligned, before 1/2/3 bytes) | 2 |
| Entry disambiguation | up to 4 candidates (when unstable) | ~2 |
| Length / span | variable | variable |

Exit disambiguation is free (from raw context, §6), so it doesn't consume
escape budget.

Worst-case for mid-block both ends: 4 entry positions × 4 exit positions × 4
entry disambiguation candidates = **64 combinations** needed before any length
encoding. This is a ceiling — aligned cuts need 0 disambiguation, and 68% of
1-byte cuts are stable (also 0). The 4 disambiguation candidates apply only to
the ~32% of 1-byte cuts that are unstable.

### Information Capacity: N Escape Characters × 1 Overhead Character

The escape character choice gives `log2(N)` bits. The overhead character is
from the Z85 alphabet (85 values = ~6.4 bits). Total combinations = N × 85.

| Escape characters | Combinations (N×85) | After cuts+disambig (÷64) | Length classes | Context cost |
|------------------|--------------------|--------------------------|--------------:|-------------|
| 1 | 85 | 1.3× | **1** | Free (Tier 1) |
| 2 | 170 | 2.7× | **2** | Free (Tier 1) |
| 3 | 255 | 4.0× | **3** | +backtick (Tier 1.5) |
| 4 | 340 | 5.3× | **5** | +1 CSV char (Tier 2) |
| 6 | 510 | 8.0× | **7** | +3 CSV chars (Tier 2) |

### Block-Aligned Only (Simpler Case)

If we drop mid-block cuts entirely, all N×85 combinations encode length:

| Escape characters | Length classes | Maximum raw bytes (×4) |
|------------------|---------------|----------------------|
| 1 | 85 | 340 |
| 2 | 170 | 680 |
| 6 | 510 | 2040 |

### Key Trade-offs

1. **1 escape character** can barely fit mid-block cuts for a fixed 2-block
   span (85 ≥ 64), but has zero length flexibility. Block-aligned-only gives
   85 length classes — generous.

2. **2 escape characters** (both from Tier 1: `_` and `~`, zero context cost)
   give enough headroom for mid-block cuts with 2 length classes, or 170
   block-aligned length classes.

3. **3+ escape characters** require Tier 1.5+ characters, adding context
   compatibility cost. The extra capacity enables more length classes or
   more sophisticated cut/disambiguation encoding.

4. **The 2-block minimum is special** because budget=2 is tight. Longer raw
   sections (3+ blocks, budget=3+) have progressively more headroom — an
   additional overhead character per block gives another ~6.4 bits each time.

5. **Length encoding interacts with the "raw to end" escape** (§10). If one
   escape character means "everything remaining is raw," that's one of the
   N options consumed — reducing the combinations available for finite-length
   sections.

## 10. Resolved Design Decisions

These were open questions; they've been answered:

- **Minimum raw section length: 4 bytes.** The format supports raw sections as
  short as 4 bytes. At this length, budget=1, which only allows block-aligned
  sections with length implied by escape character choice. There are no net
  character savings (5 characters either way), but transparency (readable raw
  bytes) is still valuable. The encoder is opportunistic — it uses raw sections
  wherever they fit and provide value.

- **Mid-block cuts: support both entry AND exit.** We are being opportunistic;
  if a mid-block cut is possible and beneficial, do it. Both boundaries should
  be supported.

- **Complexity budget: high.** This project accepts high complexity in exchange
  for value. Asymmetric entry/exit conventions are acceptable if well-motivated.

- **Asymmetric entry/exit is justified, but for different reasons than
  originally thought.** Not because trailing characters are self-stable (they
  aren't), but because exit boundaries can use already-decoded raw bytes for
  disambiguation (free), while entry boundaries must spend escape char info
  bits (costly). The asymmetry is in disambiguation cost, not in stability.

- **Length before data; no sentinels.** The decoder must know the raw section
  length before reading raw bytes. Length is encoded either implicitly (in the
  escape character choice) or explicitly (in prefix characters between the
  escape and the raw data). Sentinels are rejected. Special case: a "raw to end
  of input" escape, where length is implicitly "everything remaining" — the
  decoder doesn't need to know the exact byte count in advance because it reads
  until the stream ends. This is still length-before-data in spirit: the escape
  character tells the decoder the termination rule before any raw bytes appear.

## 11. Open Design Questions

These require discussion and decision before a specification can be written.
Grouped by dependency:

### Design Logic

1. **How many escape characters?** Tiers 1-2 give up to 6 without breaking
   JSON. What's the right number? Does the layout complexity scale with the
   number of escape characters, or is there a unified layout that works for any?

2. **What information does each escape character encode?** The `log2(N)` bits
   can be multiplexed across endianness, length classes, disambiguation, and
   boundary conventions. What's the optimal allocation?

### Layout & Encoding

3. **How are disambiguation bits laid out?** Where in the output stream do they
   go? Are they encoded as part of the escape character choice, as padding
   characters, or as part of a length prefix? How does this interact with the
   information flow constraint (§8)?

4. **Raw block internal layout:** What is the syntax and ordering of non-raw
   components? Specifically:
   - Where does the escape character go relative to the raw data?
   - Where does length information go? (Implied by escape? Explicit prefix?)
   - Where do padding characters go?
   - Where do disambiguation bits for entry/exit boundary blocks go?
   - Is the layout fixed, or does it vary based on escape character choice or
     raw section length?

### Edge Cases

5. **Stream boundaries:** Does the entry/exit analysis change at the very start
   or end of the encoded stream (no preceding/following Z85 block)?

6. **Consecutive raw sections:** Can two raw sections be adjacent with no Z85
   blocks between them? If so, what separates them? If not, what's the minimum
   Z85 gap?

## 12. Design Philosophy

This encoding shares a core value with zipng: **transparency** — making binary
data legible without breaking the container format. zipng embeds ZIP archives in
PNG images (binary data visually transparent via an image format). Extended Z85
embeds raw text in Z85 encoding (binary data textually transparent via a text
encoding). Both accept format complexity in exchange for human-readable output.

This philosophy informs the priority order (§2): correctness first (the format
must work), compatibility second (it must work *in context*), transparency third
(it should be readable where possible). Transparency is what motivates the
entire project — without it, standard Z85 already works fine.
