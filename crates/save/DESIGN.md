# Save Commit Message Design

## Overview

The `save` tool generates commit messages that encode metadata about a commit's
position in repository history. This document describes the high-level design
and semantics of each component.

## Message Format

```
[r|s|z]N [/ gG] [/ nC] [/ xHHHH] [/ oHHHH]
```

## Components

### Prefix (`r`, `s`, or `z`)

Indicates the repository state and calculation mode:

| Prefix | Meaning                                                      |
| ------ | ------------------------------------------------------------ |
| `r`    | Regular repository with full history visible                 |
| `s`    | Shallow clone (limited history)                              |
| `z`    | Z-mode: scan depth limit reached before finding trusted data |

### Revision Index (`N`)

**Definition:** Count of commits along the first-parent chain from this commit
back to the root.

**Calculation:** Walk backward following only first parents. Count the hops
until reaching a commit with no parents.

```
root -> A -> B -> C -> HEAD
 r0     r1   r2   r3    r4
```

**Inheritance:** If a parent commit is trusted, add 1 to its revision index.

### Generation Index (`gG`, optional)

**Definition:** Maximum topological distance from any root commit to this
commit.

**Calculation:** Longest path from any root through any parent chain.

**Display:** Only shown if different from revision index (indicates merge
commits exist in history).

### Commit Index (`nC`, optional)

**Definition:** Total count of unique commits reachable from this commit, minus
one.

**Display:** Only shown if different from generation index.

### Tree Hash (`xHHHH`, optional)

**Definition:** First 4 hex characters of the commit's tree object SHA.

**Special behavior:** The commit timestamp is adjusted (brute-forced) so the
resulting commit hash starts with these same 4 characters, creating a visual
match between tree and commit hashes.

**Display:** Omitted if the tree is empty.

### Origin (`oHHHH`, optional)

**Definition:** Fingerprint identifying the repository's root commits (commits
with no parents).

**Semantics:** Origin represents the TRUE initial commits of the repository's
history. It is NOT a "virtual" or "calculated" value that combines different
histories.

**Display:** Omitted for root commits (r0/s0/z0) since they ARE the origin.

#### Origin Calculation Rules

1. **If we can see all true roots** (commits with no parents are within our scan
   depth and not cut off by shallow boundary):

   - Single root: Last 2 bytes of that root's commit ID
   - Multiple roots: Last 2 bytes of SHA1(concatenated sorted root IDs)

2. **If we cannot see all true roots** (shallow clone boundary or depth limit
   prevents reaching them):

   - **Omit the origin component entirely**

3. **When inheriting from trusted parents:**

   - Single parent: Inherit its origin (if present)
   - Multiple parents with same origin: Use that origin
   - Multiple parents with different origins: **Cannot inherit.** Must scan for
     true roots. If visible, calculate origin. If not visible, omit.

#### Why This Design?

Previous designs attempted to "combine" different origins when merging branches
with different histories. This led to origin drift:

```
Branch A (oAAAA) + Branch B (oBBBB) -> Merge M1 (oXXXX = hash(A,B))
M1 (oXXXX) + Branch B (oBBBB) -> Merge M2 (oYYYY = hash(X,B)) <- DRIFT!
```

The true roots haven't changed, but the calculated origin keeps changing.

The solution: Origin is ONLY ever the true roots. If we can't see them, we don't
guess - we omit.

#### Implications for Implementation

Even when trusting parent metadata for revision/generation/commit indices, the
algorithm may need to scan deeper specifically for origin when:

- Parents have different origins (indicating merged histories or one branch
  merged additional roots)
- Verification is needed that true roots are actually visible

## Trust and Inheritance

### What "Trusted" Means

A parent commit's message is trusted if:

1. It parses correctly in our format
2. The prefix is acceptable for the current repository state:
   - `r` (Regular) commits are **always trusted** - they were calculated with
     full history visibility, so their values are accurate regardless of current
     repo state
   - `s` (Shallow) commits are **only trusted in shallow repositories** - a
     regular repo with full visibility shouldn't rely on values calculated with
     limited information
   - `z` (Z-mode) commits are **only trusted when we ourselves enter z-mode** -
     otherwise we should try to do better
3. The tree hash in the message matches the commit's actual tree

### Inheritance Rules

| Field            | Inheritance                                             |
| ---------------- | ------------------------------------------------------- |
| Revision Index   | Parent's value + 1                                      |
| Generation Index | Max of all parents' values + 1                          |
| Commit Index     | Cannot inherit; must count reachable commits            |
| Origin           | Inherit if all parents agree; otherwise scan or omit    |

### Z-Mode

Z-mode activates when scanning hits the configured depth limit before finding
trusted commits or true roots. It indicates uncertainty - the values are based
on incomplete information.

## Purpose

The commit message format serves several purposes:

| Component        | Purpose                                                 |
| ---------------- | ------------------------------------------------------- |
| Prefix           | Immediately indicates repository state (full/shallow/uncertain) |
| Revision Index   | Simple incrementing version number along main branch    |
| Generation Index | Reveals merge history when different from revision      |
| Commit Index     | Shows total reachable history size                      |
| Tree Hash        | Visual identifier; commit hash matches tree hash prefix |
| Origin           | Detects history changes, verifies same lineage          |

## Examples

```
r0 / x1234                    # Root commit, no origin (it IS the origin)
r1 / x5678 / oABCD            # Second commit, can see root ABCD
r142 / g150 / n200 / xDEF0 / oABCD   # Deep commit with merges, same origin
s50 / g75 / n100 / x9999      # Shallow clone, can't see roots, no origin
z10 / x1111                   # Hit depth limit, uncertain values, no origin
```
