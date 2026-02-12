# Extended Z85 Format — My Understanding

This is a binary-to-text encoding that extends Z85 (a base-85 encoding scheme) to reduce overhead when encoding data that already contains human-readable ASCII. I'll explain what I understood from the design document.

## Core Concept

Standard Z85 takes every 4 input bytes, treats them as a 32-bit big-endian number, breaks that into 5 base-85 digits, and maps each digit to a printable ASCII character. This gives a 25% expansion (4 bytes become 5 characters).

The problem: if your input is already readable text like "Hello", standard Z85 obscures it and wastes space encoding already-printable bytes.

Extended Z85's solution: let the encoder insert "escape sequences" that say "the next N bytes are raw—just copy them through unchanged." This eliminates encoding overhead for text-heavy regions.

Example: encoding 12 bytes where bytes 5-8 spell "Hell":
- Standard Z85: all 12 bytes → 15 encoded characters, "Hell" becomes gibberish
- Extended Z85: first 4 bytes Z85-encoded (5 chars), then escape + "Hell" raw (5 chars), then last 4 bytes Z85-encoded (5 chars) = 15 chars total but "Hell" is readable

The real savings come with longer raw sections—N raw bytes use roughly N characters instead of ~1.25×N.

## Critical Constraint: Position Invariance

Any Z85 block that stays Z85-encoded must produce **exactly** the same characters at the **exactly** the same positions as if you'd encoded the whole thing with standard Z85. You can't rearrange or change the Z85 portions—only replace complete or partial blocks with raw sections.

This isn't just a nice property; it's a hard requirement the encoder must satisfy. The decoder is simple and just follows unambiguous rules. The encoder is smart and decides when raw passthrough helps.

## Character Budget

There are 95 printable ASCII characters. Z85 uses 85 of them. That leaves 10 candidates for escape characters:

**Tier 1 (free)**: `_` and `~` — these don't break any context that Z85 doesn't already break

**Tier 1.5**: backtick — breaks markdown inline code (`` `code` ``) but nothing else new

**Tier 2**: `|` `,` `;` — break various CSV-like formats

**Tier 3**: `'` `"` `\` and space — break critical contexts like JSON, shell strings, SQL

The design highly values JSON compatibility (stay out of Tier 3), which leaves up to 6 characters (Tiers 1-2) as options. How many to actually use is an open question.

## The Budget Concept

For N raw bytes, standard Z85 would use ⌈N×5/4⌉ characters. The "budget" is those characters minus the N raw bytes themselves. This budget must cover:
- The escape character(s)
- Length information (how long is the raw section?)
- Boundary disambiguation (explained below)
- Any padding

For 4 raw bytes: budget is exactly 1 (5 chars - 4 bytes). Tight.
For 8 raw bytes: budget is 2. Still tight.
For 12 raw bytes: budget is 3. Getting comfortable.
For 16+: budget is 4+. Plenty of room.

You break even at 4 bytes (no savings, but raw is readable). You start saving characters at 5+ bytes.

## The Boundary Problem

Here's where it gets complex. The encoder might want to start or end a raw section partway through a Z85 block—not aligned to 4-byte boundaries.

**Entry boundary**: cutting a Z85 block to start a raw section. You emit partial Z85 characters for the bytes before the cut, then switch to raw.

**Exit boundary**: cutting a Z85 block to end a raw section. You emit raw bytes, then emit partial Z85 characters for the bytes after the cut.

The issue: when you only know some bytes of a 4-byte block, can you uniquely determine what the partial Z85 characters should be?

For **entry cuts** (cutting after K bytes):
- 1 byte known → first Z85 char is stable 68% of the time
- 2 bytes known → first 2 Z85 chars stable 89% of the time  
- 3 bytes known → first 3 Z85 chars stable 96% of the time

When unstable, you need ~2 bits of "disambiguation" information per boundary byte to specify which of several candidates is correct.

For **exit cuts**, there's an asymmetry: the bytes you're cutting away from were part of the raw section you just passed through—the decoder already has them! So the decoder can use those bytes to calculate what the partial Z85 characters should be, without needing extra disambiguation bits.

This means exit boundaries are "cheaper" than entry boundaries. Entry boundaries need disambiguation bits from the escape character's information capacity. Exit boundaries get disambiguation "for free" from context.

## Information Capacity

Each escape character choice provides log₂(N) bits of information, where N is how many distinct escape characters exist. With 2 escape chars: 1 bit. With 4: 2 bits. With 6: ~2.6 bits.

Plus, you can use overhead characters (Z85 alphabet, 85 values = ~6.4 bits each).

These bits can be "multiplexed"—one dimension of the escape char choice might signal byte order (big-endian vs little-endian for boundary disambiguation), another might signal length class, etc.

## The 2-Block Analysis

The tightest interesting case is 8 input bytes (2 standard Z85 blocks = 10 chars). Budget is 2.

If you want to support mid-block cuts at both entry and exit:
- Need to encode 4 entry positions (aligned or after 1, 2, 3 bytes): 2 bits
- Need to encode 4 exit positions (aligned or before 1, 2, 3 bytes): 2 bits
- Need entry disambiguation for unstable cases: ~2 bits
- Need length information: variable

Worst case: 4 × 4 × 4 = 64 combinations before you even encode length.

With 1 escape character: 1 × 85 = 85 combinations. Barely fits, but leaves ~1 length class.
With 2 escape characters: 2 × 85 = 170 combinations. Gives ~2-3 length classes after covering cut positions.
With 6 escape characters: 6 × 85 = 510 combinations. Gives ~8 length classes.

If you only support block-aligned cuts (no mid-block), all those combinations can encode length instead, which is much simpler.

## Decoder Requirements

The decoder must know the raw section length **before** reading raw bytes. No scanning for sentinel characters. Length is either:
- Implicit in escape character choice
- Explicit in prefix characters between escape and raw data
- "Everything remaining" for a special end-of-stream escape

The decoder processes left-to-right in one pass. It may reference recently decoded bytes (for exit boundary disambiguation) but never needs to look ahead.

## Resolved vs Open Questions

**Resolved:**
- Minimum raw section: 4 bytes (even though no savings, transparency has value)
- Support mid-block cuts at both entry and exit (be opportunistic)
- Accept high complexity (this project is okay with sophisticated decoders)
- Length before data (no sentinels)
- Exit boundaries use raw context for disambiguation (free), entry boundaries use escape bits (costly)

**Open:**
1. How many escape characters? (1-6 possible, what's optimal?)
2. How to allocate their information bits? (endianness vs length classes vs disambiguation)
3. How/where are disambiguation bits encoded in the output?
4. What's the exact layout syntax? (escape position, length encoding, padding placement)
5. Do stream boundaries (start/end of input) need special handling?
6. Can two raw sections be adjacent with no Z85 between them?

## Default Conventions

For tight budgets (e.g., 1 escape char with no room to signal which convention):
- **Entry boundaries**: use big-endian convention (leading Z85 characters), betting on stability (68%+ for small values common in headers/lengths)
- **Exit boundaries**: use little-endian convention (trailing Z85 characters), disambiguated from already-decoded raw bytes

These defaults are motivated by cost (exit is cheaper because disambiguation is free) and data patterns (entry bytes tend to be small values that are more stable).

## Philosophy

This is about **transparency**—making binary data human-readable without breaking the container format. Like zipng (embedding ZIP in PNG images), this accepts format complexity to achieve readability. The priority order is correctness (must work), compatibility (must work in context like JSON), then transparency (should be readable).

---

## Ambiguities I Noticed

1. **The exact stability percentages derivation**: I understand the principle (68% of first-byte values don't straddle an 85⁴ boundary), but I couldn't fully verify the 89.3% and 96.4% figures from the description alone. The mechanism is clear; the specific numbers feel like they require computation I didn't trace through.

2. **"Disambiguation from raw context" mechanics**: I understand that exit boundaries can use previously-decoded raw bytes to resolve trailing Z85 characters, and the mod-85 arithmetic example helps, but the exact algorithm for 2-byte and 3-byte exit cuts isn't fully specified. Is it always a simple modular arithmetic relationship? How does the decoder backtrack to reference those bytes?

3. **Multiplexing escape information bits**: The concept makes sense (one dimension = endianness, another = length class), but without seeing a concrete allocation example, I'm unsure how cleanly these dimensions compose. Do they interact or are they truly orthogonal?

4. **Budget calculation for mid-block cuts**: The document says mid-block cuts reduce raw bytes but don't reduce budget. I follow the logic (partial Z85 chars still occupy character positions), but I'm less clear on exactly how many Z85 characters a 1-byte or 2-byte boundary "consumes" when doing the arithmetic.

5. **"Raw to end" escape**: Mentioned as a special case where length is implicitly "everything remaining," but how does the decoder know when the input stream actually ends? Is there a separate stream-level framing, or does this only work when input length is known in advance externally?

6. **Multiple raw sections interleaved**: The document states they're supported and independent, but are there any constraints on how close together they can be? If I have 8 raw bytes, then 4 Z85-encoded bytes, then 8 more raw bytes, does that work smoothly or create layout problems?

Overall, I understood the structure, motivations, and key technical challenges. The open questions (§11) are appropriately flagged as undecided. The main ambiguity is in some of the technical details that would need to be pinned down in an actual specification.
