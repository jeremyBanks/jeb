Now I have a thorough understanding of the document. Here is my restatement:

---

## My Understanding of the Extended Z85 Encoding Format

### What problem is being solved

Standard Z85 (from ZeroMQ RFC 32) converts binary data into printable ASCII by turning every 4 input bytes into 5 output characters — a fixed 25% size increase. This works fine for genuinely opaque binary data, but it's wasteful when the input already contains printable text. The bytes for "Hello" are already perfectly readable, yet Z85 would scramble them into something unrecognizable and make the output bigger in the process. Extended Z85 adds a mechanism to let the encoder selectively skip the encoding for regions that are already human-readable, passing them through verbatim into the output.

### How it works at a high level

The output stream is a mix of two kinds of regions: ordinary Z85-encoded blocks (identical to what standard Z85 would produce) and "raw" passthrough sections. A raw section is introduced by an escape character — a character that is *not* in the standard Z85 alphabet — followed by some prefix information (at minimum a length indicator), followed by the literal input bytes. The decoder reads left-to-right in one pass: when it sees a normal Z85 character it decodes normally; when it hits an escape character, it reads the prefix to learn how many raw bytes follow, then copies those bytes directly to the output.

The encoder is the smart one — it makes opportunistic decisions about when raw passthrough is worth doing. The decoder is comparatively mechanical, just following explicit rules. The format doesn't have to produce the most compact possible encoding for every input; it just needs to be unambiguous for any valid encoding.

### The position invariant (highest priority constraint)

Any complete 4-byte Z85 block that the encoder chooses to keep as Z85 must produce *exactly the same characters at exactly the same positions* as a standard Z85 encoding of the whole input would. The raw section (escape + raw bytes + any overhead) must fit precisely into the character slots that the replaced Z85 blocks would have occupied. This means the extended encoding is always the same length or shorter than standard Z85 — never longer. It's a hard constraint: violating it makes the encoding invalid, even if the data would still be recoverable.

Partial blocks at boundaries (where a raw section starts or ends in the middle of a 4-byte block) are a separate matter — their Z85 characters may differ from what a full-block encoding would produce, and that's fine as long as the decoder can still reconstruct the original bytes.

### Priority ranking

1. **Correctness** — the position invariant and unambiguous decodability
2. **Context compatibility** — which embedding contexts (JSON strings, CSV fields, shell arguments, etc.) the output can be used in without additional escaping
3. **Transparency** — how much of the original data is visible in the output, with preference for structurally meaningful alignment (e.g., data at word boundaries is more useful to expose than data mid-structure)

### Escape character selection and compatibility analysis

The Z85 alphabet uses 85 of the 95 printable ASCII characters, leaving 10 candidates for escape characters. The document ranks them by what contexts they would break:

- **Tier 1 (free):** `_` and `~`. These break nothing that Z85 doesn't already break. Using both gives 2 escape characters at zero compatibility cost.
- **Tier 1.5:** Backtick. Only adds a problem for markdown inline code spans.
- **Tier 2:** `,`, `;`, `|`. Each breaks some flavor of CSV.
- **Tier 3:** `'`, `"`, `\`, space. These break important contexts (JSON strings, shell single-quotes, SQL, word splitting). Space is completely unusable.

The key observation is that compatibility cost is *per context*, not per character. If Z85 already can't be used inside, say, a URL without escaping, then using another URL-hostile character costs nothing extra. So the analysis focuses on which contexts are still *safe* under standard Z85 (JSON strings, shell single-quotes, CSV, TOML, SQL) and ensures the chosen escape characters don't break those.

Using up to 6 escape characters (Tiers 1-2) preserves JSON string compatibility, which is identified as probably the most important context.

### Self-signaling

The presence of non-Z85 characters in the output inherently signals "this is extended Z85, not standard Z85." A standard Z85 decoder will reject it. Conversely, any standard Z85 output (no escape characters) is also valid extended Z85 — the encoder just didn't find any opportunities for raw sections. So extended Z85 is a strict superset of standard Z85.

### Mid-block boundary analysis (the hard part)

The simplest case is when raw sections start and end on 4-byte block boundaries. But the document thoroughly explores what happens when the encoder wants to cut into the middle of a block — starting a raw section after, say, 1 or 2 bytes of a block, or ending one before the last 1-2 bytes.

**Entry boundaries** (transitioning from Z85 into raw): When you know K bytes at the start of a 4-byte block and want to emit partial Z85 characters for just those bytes, there's a stability question. The leading Z85 digit(s) are derived from the full 32-bit value of the block, and knowing only the first K bytes might not uniquely determine them — the unknown trailing bytes could push the value across an 85^4 boundary. Stability rates for uniformly random data: 68% for 1 byte known, 89% for 2, 96% for 3. When unstable, you need roughly 2 bits of extra information ("disambiguation") to resolve which Z85 character is correct.

**Exit boundaries** (transitioning from raw back into Z85): Same mathematical cost, but a crucial asymmetry in *where the disambiguation comes from*. At an exit, the bytes before the cut were already emitted as raw data — the decoder has them. It can plug them into the Z85 arithmetic and solve for the remaining unknown byte(s). Specifically, because `256^k mod 85 = 1` for all k (since gcd(256,85)=1), the trailing Z85 digit is just the sum of all four bytes mod 85, regardless of position. So the decoder can subtract the known bytes' contribution and narrow down the unknown byte to ~3 candidates — then use ~2 bits from context to pick the right one, without spending any of the limited escape-character information budget.

At entry boundaries, the decoder hasn't seen the raw bytes yet (they come *after* the cut), so it can't use them. The disambiguation bits have to come from the escape character choice or from overhead characters — both of which are scarce resources.

This is the fundamental asymmetry: **exit disambiguation is free (paid for by decoder complexity), while entry disambiguation is expensive (paid for from the limited encoding budget).** The project explicitly accepts this decoder complexity.

### The budget concept

When N bytes are passed through raw instead of Z85-encoded, the raw representation uses N characters (the bytes themselves) while standard Z85 would have used ceil(N*5/4) characters. The difference — ceil(N*5/4) - N — is the "budget": the number of extra character slots available for the escape character, length encoding, disambiguation bits, and padding.

At 4 raw bytes, the budget is just 1 (5 - 4), enough for only the escape character with no room for anything else. At 8 raw bytes it's 2, at 12 it's 3, and so on. The break-even point for *size savings* is 5 raw bytes — below that, there's no compaction benefit, only transparency (the raw bytes become readable). For large raw sections, savings grow linearly at roughly N/4.

The budget constrains what features are possible at each raw section length: block-aligned sections with implicit length need only budget=1; mid-block cuts at one boundary need budget=2; mid-block at both ends plus explicit length needs budget=4.

### Information capacity from escape characters

The choice of *which* escape character to use provides log2(N) bits of free information (where N is the number of available escape characters). With 2 escape characters, that's 1 bit. With 6, about 2.6 bits. These bits can be divided across multiple purposes simultaneously — one bit for boundary convention, another for length class, etc.

Combined with overhead characters from the Z85 alphabet (each providing ~6.4 bits from 85 possible values), the total information capacity per raw section is N_escape × 85^(overhead_chars). The document works through the 2-block case (8 input bytes, budget=2) in detail, showing that the worst case needs 64 combinations (4 entry positions × 4 exit positions × 4 disambiguation candidates), and even 1 escape character × 85 overhead values = 85 combinations is barely sufficient with no room left for variable length. Two escape characters gives a more comfortable 170 combinations.

### Resolved decisions

- Minimum raw section: 4 bytes (budget=1, block-aligned only, length implied by escape choice)
- Mid-block cuts: supported at both entry and exit
- Complexity tolerance: high — asymmetric conventions are acceptable
- The entry/exit asymmetry is justified by disambiguation cost (not by stability differences)
- Length is always communicated before raw data (no sentinel/terminator scanning), either implicitly via escape character choice or explicitly in prefix characters. A "raw to end of stream" escape is a special case of this.
- Arbitrary input lengths are supported (not limited to multiples of 4)

### What's still open

1. **How many escape characters to use.** 2 (Tier 1 only) is free. 3 costs a backtick. 6 costs CSV compatibility. The right number depends on how much encoding flexibility is actually needed.
2. **How to allocate the information bits** from escape character selection — endianness signaling, length classes, disambiguation, boundary conventions.
3. **Where disambiguation bits go in the output stream** — in the escape character choice, in padding characters, in a length prefix, or some combination.
4. **The exact byte layout of a raw section** — ordering of escape character, length info, partial Z85 characters, padding, and raw bytes.
5. **Behavior at stream boundaries** — does the analysis change when a raw section is at the very start or end of the stream?
6. **Consecutive raw sections** — can two appear back-to-back with no intervening Z85 block, and if so, how are they delimited?

### Design philosophy

The project values transparency above all else — making encoded data human-readable. Without that motivation, standard Z85 already works perfectly well. The document draws an analogy to "zipng" (embedding ZIP archives in PNG images for visual transparency): both accept significant format complexity in exchange for legibility. The priority ordering reflects this: correctness is non-negotiable, compatibility determines where the encoding can be used, and transparency is the whole reason the project exists.

---

### Things I found unclear or worth flagging

1. **The "~2 bits" for exit disambiguation is described as "free," but the source of those bits isn't fully pinned down.** The document says they come from "raw context" and that ~3 candidates remain after the mod-85 reduction, but how the decoder actually selects among them isn't specified. It says this is "free in terms of escape budget" but acknowledges it costs decoder complexity — the mechanism for that final selection step seems like it still needs concrete design work.

2. **The partial-block handling for the very end of the input stream** (when the input length isn't a multiple of 4) is mentioned as being handled "the same way as mid-block boundary cuts" but isn't elaborated on. This seems like it could interact with the "raw to end" escape in subtle ways.

3. **The document sometimes references "endianness" (BE/LE) for boundary blocks**, and this is listed as something the escape character bits might encode. But the actual distinction between the BE and LE conventions for boundary blocks isn't fully spelled out — it's gestured at (leading characters = BE, trailing characters = LE) but I wasn't sure if this means the encoder would literally have two different encoding schemes for partial blocks and would signal which one is in use.

4. **The rejected "direct byte encoding" alternative** is dismissed because it costs an extra character per boundary, but the comparison assumes very tight budgets. For longer raw sections where budget is abundant, the simplicity advantage might be worth revisiting — the document doesn't address whether a hybrid approach (direct encoding when budget is plentiful, natural when tight) is on the table.

5. **The interaction between "raw to end" and length encoding** is noted as consuming one of the N escape character options, but the implications aren't fully explored. If one escape character is reserved for "everything remaining is raw," that's a big reduction in the combinatorial space available for all other raw sections, especially at N=2.

6. **The 68% stability figure** counts uniformly random byte values, and the document notes real data will likely be better. But no analysis is given for the *adversarial* case — what happens if an attacker deliberately crafts input that maximizes unstable boundaries? The encoder can always avoid unstable cuts, so correctness isn't at risk, but it could force the encoder to waste opportunities for raw passthrough.
