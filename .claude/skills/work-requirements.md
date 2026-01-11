# Work on Requirements

Autonomously work through unsatisfied requirements until they are complete.

## Usage

```
/work-requirements [prefix]
```

- Without arguments: works on all unsatisfied requirements
- With prefix: works only on requirements under that prefix

## Instructions

When this skill is invoked:

1. **Get current status** by running:
   ```bash
   cargo run --quiet -- [prefix] --limit=100 2>&1
   ```

2. **Parse the output** to identify all incomplete requirements. Build a list of:
   - Requirement ID
   - What's missing (impl, test, example, etc.)
   - Depth in hierarchy (for prioritization)

3. **Select the next requirement to work on** using this priority order:
   - Prefer leaf requirements (deeper in hierarchy) - they're more concrete
   - Prefer requirements missing only one type (easier to complete)
   - Prefer `impl` before `test` (need implementation before you can test it)
   - Prefer requirements in already partially-satisfied subtrees

4. **Work on the selected requirement**:

   a. **Read the definition**: Find `[def <id>]` and understand what needs to be implemented/tested.

   b. **If missing impl**:
      - Find where related code should go (look at sibling implementations)
      - Implement the functionality
      - Add `[impl <id>]` annotation to the implementation
      - Verify it compiles

   c. **If missing test**:
      - Find where related tests are (look at sibling tests)
      - Write a test that verifies the requirement
      - Add `[test <id>]` annotation to the test
      - Run `cargo test` to verify it passes

   d. **If missing example**:
      - Add example code/documentation demonstrating the requirement
      - Add `[example <id>]` annotation

5. **Verify completion** by running the CLI again:
   ```bash
   cargo run --quiet -- <id> 2>&1
   ```
   The requirement should now show as satisfied.

6. **Loop**: Go back to step 1 and continue with the next requirement until all are satisfied (or until hitting a requirement that can't be easily completed).

7. **Report progress** at each iteration:
   ```
   ## Working on: _trace.syntax.brackets

   **Status**: missing impl
   **Action**: Adding [impl _trace.syntax.brackets] to parser.rs regex definition

   ...implementation details...

   **Result**: SATISFIED

   ---

   ## Progress: 5/53 requirements satisfied this session
   ## Remaining: 48 incomplete requirements
   ```

## Stopping Conditions

Stop working when:
- All requirements in scope are satisfied
- A requirement needs architectural decisions (flag it for human review)
- A requirement is ambiguous or unclear (flag for review)
- Hitting implementation complexity that needs discussion

## Example Session

```
/work-requirements _trace.syntax

## Working on: _trace.syntax.brackets (leaf, missing impl)

Reading definition from SPEC.md:29...
Definition: "An annotation is text wrapped in square brackets: `[...]`"

This is already implemented by ANNOTATION_PATTERN in parser.rs.
Adding annotation to mark it as implemented.

[Edit parser.rs to add [impl _trace.syntax.brackets] comment]

Verifying... SATISFIED

---

## Working on: _trace.syntax.not-preceded (leaf, missing test)

Reading definition from SPEC.md:32...
Definition: "Not recognized if immediately preceded by `]`, `)`, or backtick"

Implementation exists at parser.rs:48-54.
Adding test to verify this behavior.

[Edit parser.rs tests to add test_not_preceded]

Running cargo test... PASSED

Verifying... SATISFIED

---

## Session Complete
- Requirements satisfied: 12
- Remaining incomplete: 41
- Flagged for review: 2
  - _trace.types.custom: Needs design decision on validation
  - _trace.cli.watch: Marked as future work in spec
```

## Notes

- Make incremental commits after each requirement is satisfied
- If you break compilation or tests, fix immediately before proceeding
- Don't modify the spec (SPEC.md) - only add implementations and tests
- If a test string would itself be parsed as an annotation, escape or restructure it
- Prefer adding annotations to existing code over writing new code when possible
