# Sonnet R4 Review of DESIGN-CONSTRAINTS.md (post-R3 revisions)

## Major Issues

### 1. "Exit disambiguation is free" claim (§6) misleading
Says it costs zero escape budget, which obscures decoder state/latency costs. Recommends saying "free in escape budget bits" explicitly and quantifying buffer requirement (last 3 raw bytes).
**My take:** #1 is partially wrong — we explicitly said complexity is an afterthought. But "free in escape budget bits" is a clearer phrasing. Worth a minor wording fix, not a rewrite.

### 2. §9 allocation math doesn't account for non-uniform distributions
64 combinations (4×4×4) treats all cases as equally likely, but aligned cuts need 0 disambig and 68% of 1-byte cuts are stable. Inflates budget requirement.
**My take:** Fair point. The 64 is worst-case, should be labeled as such.

### 3. Escape detection window (~5 chars) contradicts position invariant
If escape occupies replaced block positions and raw sections can be arbitrarily long, escape can be arbitrarily far from trailing end. The ~5 chars is either wrong or means "at the start of replaced region."
**My take:** Good catch. The ~5 chars is about the *entry* point specifically. Should clarify.

## Medium Issues

### 4. Worked example uses undefined syntax
`_Hell` assumes escape immediately followed by raw data, which is undecided.
**My take:** Fair — add disclaimer.

### 5. Block alignment ambiguous for consecutive raw sections
Is alignment relative to global stream start or current Z85 region?
**My take:** It's global. Should state explicitly.

### 6. "Raw to end" vs length-before-data
EOF unknown on streams/sockets. Violates no-scanning?
**My take:** Partially valid but overthinking it. Raw-to-end works for known-length contexts (files, framed messages). Not every encoding must support infinite streams.

### 7-10. Minor (stability calc, tiebreaker vagueness, §11 hedging, standard Z85 passthrough)
**My take:** #7 (show 174/256) and #10 (standard Z85 is valid extended Z85) are worth adding. Others are polish.
