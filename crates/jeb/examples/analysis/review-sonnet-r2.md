# Sonnet Review Round 2 of DESIGN-CONSTRAINTS.md

## Key Issues

### Clarity
- §0 uses "stability" before it's defined (§5) — forward-reference needed
- §2 P1 "visual diff" claim is misleading — raw sections are shorter, so downstream blocks shift left. "Position" needs precise definition (original-layout index vs output byte offset)
- §6 example arithmetic `V mod 85 = (b0+b1+b2+b3) mod 85` looks wrong without noting it's a derived identity
- §3 "~5 characters" is vague — "approximately" vs "at most" vs "usually fewer"
- §5 table not explicitly labeled as entry-only

### Contradictions
- §6 exit and entry tables use different column headers for same concept ("Escape bits needed" vs "Disambiguation bits when not")
- §6 exit 3-byte row says ~2 bits but entry 3-byte says ~5 bits — unexplained asymmetry
- §9 uses flat "4" for disambiguation in ÷64 but §5 shows it varies by cut position (3-4, 9-10, 27-28)

### Missing Assumptions
- Single escape char per raw section — never stated as constraint
- Z85 stream has known total length (needed for "raw to end" escape)
- Encoder needs full input or at least lookahead — streaming implications?
- **Raw bytes must be printable ASCII or can be arbitrary?** Core gap.
- Z85 alphabet chars in raw sections disambiguated purely by position (length-before-data) — worth stating
- "jeb" never introduced

### Missing Analysis
- Length encoding design space (byte count vs block count, fixed vs variable)
- Raw sections containing escape char byte values (solved by length-before-data, but state it)
- §11 Q6 consecutive raw sections: budget implications when no Z85 block between them
- Error detection / corruption behavior (even if out of scope, say so)
- §7 should show N=1,2,3 to reinforce why 4 is minimum

### Minor
- §12 feels out of place in constraints doc
- §0 "adversarial encoder" should be "adversarial input" — encoder cooperates with format
