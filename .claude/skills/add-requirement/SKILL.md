---
name: add-requirement
description: Add or update requirements in SPEC.md with full clarity and user confirmation
allowed-tools: Read, Grep, Glob, Bash, Edit, AskUserQuestion
---

# Add or Update Requirement

Carefully define new requirements or refine existing ones with full clarity and
context.

## Usage

```
/add-requirement [description or topic]
```

## Critical Principle

**Requirements are the foundation of all work.** Implementation, testing, and
documentation all flow from requirement definitions. A poorly-defined or
misunderstood requirement propagates confusion through the entire project.
Therefore, this skill prioritizes achieving **complete mutual understanding**
between you and the user before committing any change.

**Never assume. Always confirm.**

## Instructions

When this skill is invoked:

### 1. Understand the Request

First, analyze what the user is asking for:

- Is this a new requirement or modification to an existing one?
- What is the core purpose or intent?
- What problem does this solve or capability does it add?

If anything is unclear, **ask clarifying questions before proceeding**.
Examples:

- "I want to make sure I understand: you're asking for X to do Y, correct?"
- "This seems related to [existing requirement]. Should this be a child of that,
  or separate?"
- "What's the intended relationship between this and [related feature]?"

### 2. Research Existing Context

Read the relevant parts of the specification to understand:

- Where in the hierarchy this might belong
- Related requirements it might depend on or enable
- Existing patterns for similar requirements
- Required annotation types that would apply

Show the user what you found:

```
Based on the existing spec, I see:
- Related requirements: _trace.syntax.brackets, _trace.syntax.structure
- This appears to be about [topic], which would fit under _trace.syntax
- Similar requirements use @children mode and require impl + test
```

### 3. Propose the Requirement Definition

Draft the requirement with:

- **ID**: Where it fits in the hierarchy
- **Mode**: @self, @children, or @either (default)
- **Required types**: What modifiers (+type, -type) apply
- **Definition text**: Clear, testable description

Example proposal:

```markdown
## Proposed Addition

[def _trace.syntax.backtick-exclusion] Annotations are not recognized if
immediately preceded or followed by a backtick. This prevents matching inline
code examples like `[annotation]`.

**Location**: Add after _trace.syntax.not-followed (line ~36 in SPEC.md)
**Mode**: @either (default) **Types**: impl, test (inherited defaults)
**Parent**: _trace.syntax

Does this capture your intent?
```

### 4. Confirm Understanding

Before making any changes, explicitly confirm with the user:

- "Does this definition accurately capture what you want?"
- "Is the placement in the hierarchy correct?"
- "Are there any edge cases or clarifications needed?"

**Wait for explicit approval.** Do not proceed if there's any ambiguity.

### 5. Check for Implications

Consider and discuss:

- Does this requirement conflict with or duplicate existing ones?
- Will this require changes to other requirements?
- Are there implementation implications to consider?
- Should related requirements be updated for consistency?

### 6. Make the Change

Only after confirmation:

1. Edit SPEC.md to add/update the requirement
2. Use consistent formatting with surrounding content
3. Ensure the ID follows naming conventions
4. Add any cross-references if needed

### 7. Verify

Run `_trace` to confirm:

- The requirement is parsed correctly
- It appears in the expected place in the hierarchy
- Required types are computed correctly

Report back to the user with the result.

## Patterns for Good Requirements

**Clear and Testable**:

- BAD: "The system should handle errors gracefully"
- GOOD: "Invalid IDs produce a non-fatal error including the location and
  reason"

**Appropriately Scoped**:

- BAD: "Implement the entire parsing system" (too broad)
- GOOD: "Annotations inside markdown code fences are ignored" (specific)

**Hierarchically Placed**:

- Consider if it's a refinement of existing requirement → make it a child
- Consider if it's a new capability → might be sibling or new subtree
- Consider if it modifies behavior → might need to update existing definition

## Relationship Types

Most relationships are hierarchical (parent.child), but sometimes you need to:

- Reference another requirement in the definition text
- Add annotations linking definitions (e.g., `[depends some.other.req]`)
- Note interactions in the description

When you see these patterns in existing requirements, follow them.

## Dialogue Examples

**Good dialogue**:

```
User: We need to handle nested brackets somehow
Agent: I want to make sure I understand - are you referring to cases like [[nested]] or [outer [inner] end]? And should these be:
(a) Ignored entirely as invalid annotations?
(b) Parsed in some specific way?
(c) Treated as an error?

User: Option (a) - just ignore them as they're probably not intentional annotations

Agent: Got it. So the requirement would be:
"Text containing nested brackets (e.g., `[[x]]` or `[a [b] c]`) is not recognized as an annotation."

This would be a child of _trace.syntax, similar to not-preceded/not-followed.
Should I add it as [def _trace.syntax.nested-brackets]?
```

**Bad dialogue** (don't do this):

```
User: We need to handle nested brackets somehow
Agent: [immediately edits SPEC.md without clarification]
```

## Remember

- Requirements define what the project IS. Get them right.
- Ambiguity now means confusion later.
- It's always better to ask one more question than to define something wrong.
- The user's time spent clarifying now saves everyone's time later.

---

<!-- Metadata - ignore this section -->
<!-- [impl _trace.skills.add] -->
<!-- [impl _trace.skills.self-contained] -->
