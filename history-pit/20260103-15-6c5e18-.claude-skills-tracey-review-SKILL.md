---
name: tracey-review
description: Review TRACEY.md specification files to identify overly broad rules that should be split into more specific, granular requirements. Use when asked about tracey rule quality, specification granularity, identifying redundancy, or improving tracey coverage metrics.
allowed-tools: Read, Grep, Glob, Bash
---

# Tracey Rule Review Skill

## Overview

This skill analyzes TRACEY.md specification files to identify rules that are too
broad and could be split into multiple more-specific, focused requirements. This
improves traceability by making implementation and verification more granular.

## When to Use This Skill

Invoke this skill when you:

- Ask about tracey rule quality or specification granularity
- Want to review TRACEY.md files for overly broad rules
- Ask how to improve tracey coverage metrics
- Ask which rules should be split or broken down
- Ask about specification traceability issues

## Instructions

### Step 1: Locate TRACEY.md Files

Find all TRACEY.md files in the project:

```bash
find ~/jeb -name "TRACEY.md" -type f 2>/dev/null
```

This should find files at:

- `/Users/jeb/jeb/TRACEY.md` (workspace-level rules)
- `/Users/jeb/jeb/crates/jeb-value/TRACEY.md` (crate-specific rules)

### Step 2: Get Coverage Baseline

Run the tracey CLI to get current coverage statistics:

```bash
cd ~/jeb && tracey matrix
```

This provides context about how many rules are currently implemented/verified.

### Step 3: Extract and Analyze Rules

For each TRACEY.md file, read the entire file and parse the rules. Look for
patterns that indicate a rule is "too broad":

#### Pattern 1: Multiple Trait Implementations

**Indicator**: Rule text contains multiple trait names like `Eq`, `PartialEq`,
`Ord`, `PartialOrd`, `Hash`, `Copy`, `Clone`, `Debug`, `Display`, `Default`,
`From`, `Into`, `Try From`, `Deref`, `DerefMut`, etc.

**Example**: `"Value MUST implement Eq, PartialEq, Ord, PartialOrd, and Hash"`

**Heuristic**: If the rule mentions 2+ different trait names, flag as
potentially too broad.

#### Pattern 2: Multiple Type Conversions Bundled

**Indicator**: Rule text contains phrases like:

- "for all of primitive types"
- "for all integer types"
- "for all numeric types"
- "for type T where T is one of..."
- Multiple specific types listed (e.g., `bool`, `f32`, `f64`, `char`, `u8`,
  etc.)

**Example**:
`"Bytes variant MUST implement From<T> for all of primitive types (), bool, f32, f64, char and all integer types"`

**Heuristic**: If a rule covers 3+ different type conversions or uses language
like "all X types", flag as potentially too broad.

#### Pattern 3: Conditional Logic

**Indicator**: Rule contains if/then/else branching:

- "If ... then ... must..."
- "If and only if..."
- "... otherwise..."
- "Either ... or ..."

**Example**:
`"If V implements From<INNER_W> then V MUST also implement From<W>, otherwise if V implements TryFrom<INNER_W> then..."`

**Heuristic**: If the rule has 2+ conditional branches, flag as potentially too
broad. These often should be split into separate rules per condition.

#### Pattern 4: Enumerated List of Requirements

**Indicator**: Rule lists multiple specific, distinct requirements:

- Using commas or "and" to separate items
- Multiple verbs describing different actions
- Multiple specific values or types
- Lists starting with "all of" or "each of"

**Example**:
`"Value MUST be marked #[must_use], implement Clone, implement Debug, and implement Default"`

**Heuristic**: If a rule has 3+ distinct requirements separated by commas or
"and", flag as potentially too broad.

### Step 4: Generate Detailed Report

For each broad rule found, create an entry with:

1. **Rule ID and Text**: Quote the exact rule from TRACEY.md
2. **Issues Identified**: Which patterns it matches (e.g., "Multiple traits" +
   "Enumerated list")
3. **Suggested Splits**: Propose specific new rule IDs and their individual
   requirements
4. **Rationale**: Brief explanation of why splitting improves traceability
5. **Current Coverage**: If this rule has implementation/verification
   annotations, note them

### Step 5: Provide Summary and Recommendations

Summarize:

- Total rules analyzed
- Number of potentially broad rules found
- Percentage of rules needing review
- Benefits of splitting (more granular tracking, clearer testing, better
  coverage metrics)
- Next steps for user

## Examples

See [examples.md](examples.md) for concrete examples of:

- Well-scoped rules (good examples to follow)
- Overly broad rules from the actual codebase
- How rules should be split
- Before/after rule IDs

## Expected Output Format

```
# Tracey Rule Review Report

## Summary Statistics
- Total rules analyzed: X
- Potentially broad rules found: Y
- Percentage requiring review: Z%
- Current tracey coverage: A/B rules (C%)

## Detailed Findings

### r[namespace.feature.aspect]
**Patterns Matched**: [Pattern names]
**Issue Description**: [Why it's too broad]

**Current Rule Text**:
> [Quote from TRACEY.md]

**Suggested Splits**:
- r[namespace.feature.aspect-a]: [Specific requirement 1]
- r[namespace.feature.aspect-b]: [Specific requirement 2]
...

**Current Annotations**: [Any [impl] or [verify] found for this rule]

---

## Summary and Recommendations
- Key insight 1
- Key insight 2
- Actionable next steps
```

## Tips for Quality Review

1. **Be precise with rule IDs**: Keep them consistent with project naming
   conventions
2. **Quote exact text**: Always include the actual rule text from TRACEY.md
3. **Explain the benefit**: Help the user understand why splitting helps
4. **Consider implementation**: Think about whether the suggested splits make
   sense for how the code is structured
5. **Note coverage gaps**: If a rule is partially implemented, that's important
   context
