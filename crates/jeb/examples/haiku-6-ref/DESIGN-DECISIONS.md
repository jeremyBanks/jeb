# Design Decisions for Extended Z85 Implementation

## Open Questions (§12) - Decisions Made

### Q1: How many escape characters?
**DECISION: 2 escape characters**
- Characters: `_` (underscore) and `~` (tilde)
- Both are Tier 1 (completely free) per §5
- Provides 2 bits of information (log2(2) = 1 bit)
- Sufficient for: distinguishing between entry/exit boundary conventions
- Block-aligned capacity: 2 × 85 = 170 length classes (generous)
- Mid-block capacity: ~85 combinations per boundary type (sufficient with disambiguation)

### Q2: What information does each escape character encode?
**DECISION: Escape character choice signals entry boundary convention**
- `_` = entry uses big-endian (BE) leading characters (default/most stable)
- `~` = entry uses little-endian (LE) context if beneficial (experimental)
- Exit always uses trailing characters with raw context disambiguation (per §7)
- For block-aligned entries (most common), convention doesn't matter, but consistency is good
- This allocates 1 bit; the remaining 1 bit per character goes to overhead character flexibility

### Q3: How are disambiguation bits laid out?
**DECISION: Fixed layout pattern for simplicity**
- Entry boundary: if needed (unstable byte), disambiguate with first overhead char
- Exit boundary: if needed, use raw context (free, per §7)
- Length encoding: use combinations of (escape char, overhead chars) as needed
- Overhead character (after escape) can encode:
  - Part of length (for finite raw sections)
  - Disambiguation info for unstable entry boundaries
  - "Raw to end" signal (max value in overhead char space)

### Q4: Raw block internal layout - syntax and ordering
**DECISION: Escape-first pattern (prefix-oriented)**
Layout for raw section:
1. Escape character (1 char) - signals raw + convention
2. Overhead character (1 char) - encodes length or length class
3. Partial Z85 chars (0-3 chars) - for entry boundary bytes
4. Raw data (N bytes) - the actual raw bytes
5. Partial Z85 chars (0-3 chars) - for exit boundary bytes

This makes the length available before reading raw data (§11 requirement).

## Additional Design Choices

### Minimum raw section length: 4 bytes
- Matches Z85 block size
- Budget=1 allows block-aligned sections guaranteed
- Mid-block cuts possible when boundary bytes are stable (68%+)
- No net character savings vs standard Z85, but transparency is valuable

### Mid-block support: Both entry AND exit
- Entry boundary: emits partial Z85 characters for known prefix bytes
- Exit boundary: emits partial Z85 characters for known suffix bytes, disambiguated from raw context
- Budget analysis in §10 guides implementation

### Length encoding strategy
- For finite-length raw sections:
  - Block-aligned (common): length in overhead character (0-85 blocks, 0-340 bytes)
  - Mid-block: length in overhead char, with disambiguated bytes also packed in
- For "raw to end": special overhead character value signals "consume until stream end"
- Maximum raw section length: unlimited (can split long sections if needed, but R4 allows consecutive raw sections)

### Raw byte eligibility (R3 default policy)
- Include bytes that are already printable ASCII (0x20-0x7E)
- Include Z85 alphabet characters (already visible in Z85 mode anyway)
- Include escape characters `_` and `~` (length-prefixed sections are unambiguous)
- Exclude non-printable bytes (0x00-0x1F, 0x7F-0xFF) unless explicitly configured
- Decoder accepts any byte value in raw sections (no restrictions)

### Stream boundary edge cases (Q5)
- Entry at stream start: if input begins with binary, process normally (no preceding Z85 block)
- Exit at stream end: if raw section extends to final byte, process normally (no following Z85 block)
- Both at boundaries: supported (may start/end with raw)
- No special handling needed; the position invariant naturally extends to stream boundaries

### Consecutive raw sections (Q6 / R4)
- Format allows consecutive raw sections (decoder must support)
- Well-behaved encoder should never produce them (merge into single section)
- Exception: if a hard max section length exists, encoder splits long sections
- No built-in max in this design (sections can be arbitrarily long)

## Position Invariant Implementation

Every complete Z85 block that remains encoded in extended output must match standard Z85:
- Same characters
- Same positions relative to stream start
- This is enforced by the layout: raw sections consume exactly the character budget that Z85 blocks would occupy
- Partial blocks at boundaries are separate from this invariant (explicitly allowed per §3)

## Priorities Applied (§3)

1. **P1 - Correctness & Position Invariant**: Non-negotiable. Every design choice preserves the invariant.
2. **P2 - Context Compatibility**: Both escape characters are Tier 1 (free). No additional escaping needed in JSON, shell args, etc.
3. **P3 - Transparency**: Raw passthrough makes readable ASCII visible. Mid-block support maximizes opportunity (non-aligned lengths + cuts).

## Complexity Acceptance

Per §0 and §11: "Implementation complexity (encoder or decoder) is not a significant concern." This design:
- Asymmetric entry/exit conventions (complex but justified by §7)
- Mid-block boundary handling (complex but necessary for R2)
- Exit disambiguation from raw context (adds decoder complexity but saves escape budget)
- Flexible length encoding (allows multiple layout strategies as needed)

These are all acceptable in this project's scope.
