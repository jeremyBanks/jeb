# Validate Requirements

Deep validation of requirements beyond simple annotation detection.

## Usage

```
/validate-requirements [prefix]
```

- Without arguments: validates all incomplete requirements
- With prefix: validates only requirements under that prefix (e.g., `_trace.syntax`)

## Instructions

When this skill is invoked:

1. **Run the _trace CLI** to get current requirement status:
   ```bash
   cargo run --bin _trace -- [prefix] --limit=100 2>&1
   ```
   If a prefix argument was provided, use it. Otherwise run without prefix to get all incomplete items.

2. **Parse the output** to identify incomplete requirements. Each line like:
   ```
   - _trace.syntax.brackets: missing impl, test
   ```
   indicates a requirement that needs validation.

3. **For each incomplete requirement**, perform deep validation:

   a. **Find the definition**: Search for `[def <requirement-id>]` to locate where the requirement is defined. Read the surrounding context to understand what the requirement specifies.

   b. **Find claimed implementations**: Search for `[impl <requirement-id>]` annotations. For each one:
      - Read the surrounding code/documentation
      - Assess whether the implementation actually fulfills the requirement
      - Note any gaps or concerns

   c. **Find claimed tests**: Search for `[test <requirement-id>]` annotations. For each one:
      - Read the test code
      - Assess whether the test adequately verifies the requirement
      - Note if test coverage seems incomplete

   d. **Check for unlabeled implementations**: Sometimes code implements a requirement but lacks the annotation. Search for relevant keywords from the requirement definition to find potential implementations that should be annotated.

4. **Report findings** for each requirement:
   - **Status**: `SATISFIED`, `PARTIALLY SATISFIED`, `NOT SATISFIED`, or `NEEDS REVIEW`
   - **Definition summary**: Brief description of what the requirement specifies
   - **Implementation assessment**: Whether impl annotations exist and if they truly satisfy the requirement
   - **Test assessment**: Whether test annotations exist and if they adequately verify the requirement
   - **Recommendations**: Specific actions needed (add annotation, write implementation, write test, etc.)

5. **Summary**: At the end, provide:
   - Count of requirements by status
   - Priority list of items needing attention
   - Any systemic issues noticed (e.g., "many tests lack annotations")

## Example Output

```
## Validating _trace.syntax requirements

### _trace.syntax.brackets
**Status**: SATISFIED (missing annotation)
**Definition**: An annotation is text wrapped in square brackets: `[...]`
**Implementation**: Found in src/parser.rs:13-16 - regex pattern correctly matches bracketed text
**Test**: Found in src/parser.rs:289-298 - test_brackets_detection verifies bracket matching
**Recommendation**: Add `[impl _trace.syntax.brackets]` annotation to ANNOTATION_PATTERN

### _trace.syntax.not-preceded
**Status**: SATISFIED
**Definition**: Annotations not recognized if preceded by `]`, `)`, or backtick
**Implementation**: src/parser.rs:48-55 correctly checks prev_char
**Test**: src/parser.rs:301-309 verifies markdown links are ignored
**Recommendation**: None - requirement is fully satisfied

### _trace.types.custom
**Status**: NOT SATISFIED
**Definition**: Custom annotation types can be defined with +type modifier
**Implementation**: No implementation found
**Test**: No test found
**Recommendation**: Implement custom type support in parser and hierarchy modules

## Summary
- SATISFIED: 2
- NOT SATISFIED: 1
- Total validated: 3

Priority items:
1. Implement _trace.types.custom - no implementation exists
```

## Notes

- Focus on semantic correctness, not just annotation presence
- Be thorough but concise in assessments
- If unsure whether something satisfies a requirement, mark it NEEDS REVIEW
- Consider the full context of requirements (parent/child relationships)
- For hierarchical projects, requirements in child documents should trace back to parent requirements

---

<!-- Metadata - ignore this section -->
<!-- [impl _trace.skills.validate] -->
<!-- [impl _trace.skills.self-contained] -->
