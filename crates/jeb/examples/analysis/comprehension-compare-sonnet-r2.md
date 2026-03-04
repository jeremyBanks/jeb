

I'll analyze both documents systematically for discrepancies, contradictions, and omissions.

## Verdict: Document B is largely faithful, with a few errors and some notable omissions.

---

## 1. Direct Errors in B

### 1a. Worked example character count

**Document A:**
> Extended: `rZUgH _Hell 0sjjE` (14 characters: 5+1+4+4≈14, 1 saved)

A explicitly says 14 characters total, noting "zero savings here" for the middle block specifically but showing the total is 14 vs 15.

**Document B:**
> Extended Z85: first 4 bytes Z85-encoded (5 chars), then escape + "Hell" raw (5 chars), then last 4 bytes Z85-encoded (5 chars) = **15 chars total** but "Hell" is readable

B counts 5+5+5=15, claiming no savings at all. But A counts 5+(1+4)+5=15 characters occupying 14 character *positions* — wait, actually A says 14 characters. Let me re-read A.

A says: `(14 characters: 5+1+4+4≈14, 1 saved)` — but 5+1+4+4=14 doesn't work either since the last block is 5 Z85 chars not 4. A's own arithmetic is `5 + 1 + 4 + 5 = 15` for the extended version, but A writes "14" and "1 saved." A then immediately says: *"zero savings here"* in the next paragraph. This is actually an internal inconsistency in A itself. The parenthetical says "14 characters... 1 saved" but the prose says "zero savings." The escape + 4 raw bytes = 5 characters, replacing 5 Z85 characters = zero savings, which matches A's prose. **B's count of 15 is actually correct; A's parenthetical of "14" appears to be a typo in A.**

Actually, re-reading more carefully — A writes `5+1+4+4≈14`. That last `4` seems wrong; the third block should be 5 Z85 chars. A has a bug in its own example. B's "15 chars total" is the correct count.

### 1b. Last Z85 block character count in example

A writes the extended output as:
> `rZUgH _Hell 0sjjE`

That's 5 + 1 + 4 + 5 = 15 characters (counting `0sjjE` as 5). But A's parenthetical says `5+1+4+4≈14`. The `4` for the last block and the total of `14` are both wrong — it should be `5+1+4+5=15`. B correctly says 15.

---

## 2. Facts in A Missing from B

### 2a. Input length flexibility

**A (§1):**
> The original ZeroMQ spec requires input length to be a multiple of 4 bytes; we lift that restriction — arbitrary input lengths are supported, with partial final blocks handled the same way as mid-block boundary cuts (see §5-6).

B never mentions this. This is a significant design decision — standard Z85 requires 4-byte-multiple inputs, extended Z85 does not.

### 2b. Raw byte values / encoder policy

**A (§0, "Raw byte values"):**
> The decoder imposes no restriction on what bytes appear in a raw section — it knows the length from the prefix and passes bytes through without validation. The *encoder* decides which bytes to include based on the desired compatibility profile...

B doesn't discuss this encoder-policy-vs-format-constraint distinction at all. B doesn't mention that raw sections can technically contain *any* byte value (even non-printable ones) at the format level.

### 2c. Block alignment reference point

**A (§0, "Block alignment"):**
> "Block-aligned" always means aligned to Z85's 4-byte / 5-character block boundaries relative to the **start of the Z85 stream**, not relative to the raw section or any other reference point.

B never clarifies this. For a reader of B alone, "block-aligned" is ambiguous.

### 2d. Position invariant applies to output length

**A (§2):**
> Output length is always **≤ standard Z85 length** for the same input

B doesn't state this explicitly as a consequence of the position invariant.

### 2e. Partial blocks vs position invariant distinction

**A (§2):**
> **Partial blocks at boundaries:** When the encoder cuts a Z85 block partway through to begin or end a raw section, the partial block's Z85 characters may differ from what standard Z85 would produce... This is acceptable as long as the decoder can unambiguously reconstruct the original bytes. Partial blocks are a separate case from the position invariant above, not an exception to it.

B doesn't make this distinction clearly. B says "only replace complete or partial blocks with raw sections" but doesn't clarify that partial blocks are *not* subject to the position invariant.

### 2f. Alignment preference tiebreaker (P3)

**A (§2, P3):**
> **Alignment preference for tiebreaking:** When choosing between escape placement options, prefer the one that produces more structurally aligned raw data (more trailing zero bits → more likely to be at a meaningful boundary)

B omits this heuristic entirely.

### 2g. Escape character placement constraint

**A (§3):**
> The escape character must appear within the character positions of the Z85 block(s) being replaced — practically, within ~5 characters of the transition point.

B doesn't mention this spatial constraint on where the escape character can appear.

### 2h. The "direct byte encoding" rejected alternative

**A (§6, "Rejected Alternative: Direct Byte Encoding"):**
> Instead of using Z85's natural leading/trailing characters for boundary bytes, we could encode them directly: map K bytes → K+1 Z85 characters via a bijection independent of the block's other bytes.

A discusses and rejects this alternative. B doesn't mention it at all.

### 2i. Mod-85 arithmetic detail

**A (§6):**
> `2^8 ≡ 2^16 ≡ 2^24 ≡ 1 (mod 85)` ... This means `V mod 85 = (b0 + b1 + b2 + b3) mod 85`

This key mathematical property is what makes exit disambiguation tractable. B mentions the mod-85 example vaguely but doesn't include this specific identity.

### 2k. Self-signaling property

**A (§0):**
> The presence of non-Z85 characters (escape chars) in the output self-signals that this is extended Z85, not standard Z85. A standard Z85 decoder will reject the escape characters as invalid... Conversely, standard Z85 output (no escape characters) is valid extended Z85.

B doesn't discuss this compatibility property.

### 2l. Budget requirements table for mid-block transitions

**A (§7)** provides a specific table of minimum budget needed for each configuration (block-aligned no length = 1, block-aligned with length = 2, etc.). B discusses budgets but doesn't reproduce these specific thresholds.

---

## 3. Facts in B Missing from A

### 3a. B's "Ambiguities I Noticed" section

B includes 6 self-identified ambiguities. These are B's own observations, not from A, so they don't represent discrepancies — but they're worth noting as places where B found A unclear. Most are reasonable questions about unspecified details.

---

## 4. Possible Misunderstandings in B

### 4a. "Little-endian convention" for exit

**B:**
> **Exit boundaries**: use little-endian convention (trailing Z85 characters), disambiguated from already-decoded raw bytes

**A (§6):**
> **Exit: Trailing characters (LE), disambiguated from raw context**

B's phrasing is acceptable here — it matches A's shorthand. But B could mislead a reader into thinking byte ordering changes. A's §0 clarifies: *"Endianness discussion (§6) applies only to how partial Z85 blocks encode boundary bytes, not to raw data ordering."* B doesn't include this clarification.

### 4b. Stability framing

**B:**
> 1 byte known → first Z85 char is stable 68% of the time

This is correct but B doesn't note A's qualifier:

**A (§0):**
> Stability percentages in §5 and §6 assume uniformly random byte values. Real data (small integers, ASCII-adjacent values, structured headers) will often have higher stability rates.

B omits this important caveat about what "68%" means.

---

## Summary

B is a **largely faithful** rewrite. The main issues are:

1. **B gets the worked example count right (15)** where A's parenthetical actually has a typo (says 14).
2. **Several meaningful omissions**: arbitrary input length support, raw byte value policy, block alignment reference point, self-signaling property, the direct-encoding rejected alternative, escape character placement constraint, and the mod-85 identity.
3. **No outright contradictions** on substance — B doesn't claim anything that directly conflicts with A's technical content.
4. **Missing caveats**: stability percentage assumptions, endianness-applies-only-to-boundary-blocks clarification, partial-blocks-aren't-subject-to-position-invariant distinction.
