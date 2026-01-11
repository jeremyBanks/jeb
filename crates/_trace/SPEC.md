# _trace Specification

[def _trace @children]
_trace is a language-agnostic requirements tracking tool. It finds annotations in source files, extracts their context, and tracks whether requirements are satisfied by corresponding implementations and tests.

This is a simplified variant of [tracey](https://github.com/bearcove/tracey), exploring some ideas to see if anything is worth suggesting upstream. Key simplifications include looser annotation detection (anywhere in text, not position-dependent), unified file handling (no separate spec vs. source distinction), and a minimal CLI focused on AI-agent usability.

This specification uses its own annotation syntax to define requirements.

---

## File Discovery

[def _trace.files @children]
Requirements for discovering files to scan for annotations.

[def _trace.files.globs]
The tool searches for annotations in all files matching `**/*.md` or `src/**/*`. Hidden directories (dotfiles) are included by default, except `.git/` which is always excluded.

[def _trace.files.language-agnostic]
File scanning is entirely language-agnostic. Files under `src/` may be any format—source code, configuration, documentation, or any other text content.

---

## Annotation Syntax

[def _trace.syntax @children]
Requirements for annotation syntax and parsing.

[def _trace.syntax.brackets]
An annotation is text wrapped in square brackets: `[...]`.

[def _trace.syntax.not-preceded]
An annotation is not recognized if immediately preceded by `]`, `)`, or a backtick. This prevents matching markdown link syntax like `[text](url)` or `[text][ref]`, and inline code like `` `[example]` ``.

[def _trace.syntax.not-followed]
An annotation is not recognized if immediately followed by `[`, `(`, or a backtick. This prevents matching markdown link syntax and inline code.

[def _trace.syntax.backtick-escape]
An annotation wrapped in backticks on both sides (e.g., `` `[def foo]` ``) is not recognized. This allows documentation and tests to include example annotations without them being parsed. Note: a backtick on only one side also prevents recognition per the not-preceded/not-followed rules.

[def _trace.syntax.structure]
An annotation contains space-separated components. There must be at least two components: the type and the ID. Brackets containing only one component (or zero) are not recognized as annotations.

[def _trace.syntax.type]
The first component of an annotation is its type (e.g., `def`, `impl`, `test`, `example`). There is no default type—the type must be explicitly specified.

[def _trace.syntax.id +example]
The requirement ID is the last space-separated component. It consists of period-separated segments representing a hierarchy.

[def _trace.syntax.id.segments]
Each segment of an ID must contain at least one character. All characters must be alphanumeric (a-z, A-Z, 0-9), dashes (`-`), or underscores (`_`).

[def _trace.syntax.id.minimum]
An ID must have at least one segment. A single segment implies no periods.

[def _trace.syntax.id.hierarchy]
ID segments represent a hierarchy. The order of segments is significant: `foo.bar` is a child of `foo`.

---

## Context Extraction

[def _trace.context @children]
Requirements for extracting the descriptive context around an annotation.

[def _trace.context.boundaries]
An annotation's context consists of its line plus all contiguous lines above and below, stopping when a line contains no alphanumeric characters. Such boundary lines (containing no alphanumeric characters) are not included in the context.

[def _trace.context.prefix-strip]
If all lines in the extracted context share a common prefix consisting entirely of non-alphanumeric characters (excluding dashes, underscores, and periods), that prefix is stripped from each line. This handles comment prefixes like `// `, `# `, `> `, etc.

[def _trace.context.shared]
Multiple annotations on the same line share the same context. This is not a conflict.

---

## Location Tracking

[def _trace.location @children]
Requirements for recording annotation locations.

[def _trace.location.file]
Each annotation records the file path where it was found.

[def _trace.location.line]
Each annotation records its line number, 1-indexed.

[def _trace.location.column]
Each annotation records its column number, 1-indexed, pointing to the opening `[`.

---

## Annotation Types

[def _trace.types @children]
Requirements for annotation type handling.

[def _trace.types.def]
The `def` type defines a requirement. It establishes a requirement ID that other annotations can reference.

[def _trace.types.custom]
Annotation types other than `def` are user-defined. Common types include `impl` (implementation), `test` (verification), `example` (examples), and `doc` (documentation). Any string of alphanumeric characters, dashes, or underscores is a valid type name.

[def _trace.types.def-required]
Requiring `def` as a satisfaction type is a non-fatal error (the definition itself is the `def`). The error is reported and the modifier is ignored.

---

## Hierarchy

[def _trace.hierarchy @children]
Requirements for hierarchical requirement relationships.

[def _trace.hierarchy.implicit-ancestors]
Defining a requirement implicitly creates all ancestor requirements. If `[def foo.bar.baz]` is found but `foo.bar` and `foo` are not explicitly defined, they are treated as if defined with `[def foo.bar]` and `[def foo]` with no description.

[def _trace.hierarchy.inheritance]
Implicit ancestors inherit satisfaction criteria from their nearest explicitly-defined ancestor. If no ancestor is explicitly defined, they inherit the global defaults.

[def _trace.hierarchy.completion]
A requirement is complete when it is satisfied AND all of its descendants are complete.

---

## Satisfaction Criteria

[def _trace.satisfaction @children]
Requirements for determining when a requirement is satisfied. Note the distinction between "satisfied" (the requirement's own criteria are met) and "complete" (satisfied AND all descendants are complete). The CLI shows incomplete requirements by default.

[def _trace.satisfaction.defaults]
By default, a requirement needs both `impl` and `test` annotations with matching IDs to be satisfied.

[def _trace.satisfaction.inheritance]
A requirement inherits its required types from its parent. If no ancestor modifies requirements, the global defaults (`impl` and `test`) apply.

[def _trace.satisfaction.modifiers +example]
Required types are modified in a `def` annotation using `+type` to add a required type and `-type` to remove one. Multiple modifiers can be joined: `+doc-test` adds `doc` and removes `test`.

[def _trace.satisfaction.modifier-descendants]
Modifications to required types apply to the requirement and all its descendants (unless overridden by a descendant's own modifiers).

[def _trace.satisfaction.mode]
A `def` may specify `@self`, `@children`, or `@either` as the satisfaction mode. The default is `@either`. Unlike required types, the satisfaction mode does not inherit—each requirement defaults to `@either` unless explicitly specified.

[def _trace.satisfaction.mode.self]
With `@self`, the requirement is satisfied when annotations of all required types exist for this exact ID.

[def _trace.satisfaction.mode.child]
With `@children`, the requirement is satisfied when: (1) for each required type, all direct children that also require that type have it satisfied, AND (2) at least one direct child requires each type. A requirement with `@children` and zero children is automatically unsatisfied.

[def _trace.satisfaction.mode.either]
With `@either`, the requirement is satisfied if either the `@self` or `@children` condition is met.

---

## Error Handling

[def _trace.errors @children -test]
Requirements for error handling. All errors described here are non-fatal—the tool continues processing and reports them.

[def _trace.errors.duplicate-def]
Defining the same requirement ID more than once is a non-fatal error. For determinism, the definition whose file path comes first lexicographically is used.

[def _trace.errors.add-existing]
Adding a required type that already exists (e.g., `+impl` when `impl` is already required) is a non-fatal error.

[def _trace.errors.remove-missing]
Removing a required type that does not exist (e.g., `-doc` when `doc` is not required) is a non-fatal error.

[def _trace.errors.unrequired-annotation]
An annotation of a type not required by the target requirement is a non-fatal error.

---

## CLI Interface

[def _trace.cli @children]
Requirements for the command-line interface.

[def _trace.cli.help-in-output]
Every CLI command output includes a succinct summary of available options and related commands, helping users (including AI agents) discover functionality.

[def _trace.cli.default-output]
Running `_trace` with no arguments shows incomplete requirements by default. If all requirements are complete, it displays a summary with counts (total, satisfied, unsatisfied). If any are incomplete, it displays the list of incomplete items. Use `--all` to show all requirements regardless of completion status.

[def _trace.cli.list]
The tool can list requirements as a nested tree showing each requirement's satisfaction status.

[def _trace.cli.list.format]
Requirements are displayed as a nested list. Each entry shows the ID and its status (e.g., "done with impl, test" or "missing impl").

[def _trace.cli.filter-prefix]
One or more ID prefixes may be provided as arguments to filter the output (e.g., `_trace foo.bar` shows only requirements under `foo.bar`).

[def _trace.cli.filter-type]
The `--type` flag filters output to show only specific annotation types (e.g., `--type=impl,test`).

[def _trace.cli.limit]
Output is limited to 32 items by default. A message indicates if more items exist.

[def _trace.cli.pagination]
The `--limit` and `--skip` flags control pagination for large result sets.

[def _trace.cli.context]
The `--context` flag displays the full extracted context for every annotation, including file path, line, and column.

[def _trace.cli.lines]
The `--lines` flag displays only file path, line, and column for each annotation, without the full context.

[def _trace.cli.context-of]
The `--context-of=type1,type2` flag expands full context only for specific annotation types.

---

## Examples

[example _trace.syntax.id]
[example _trace.syntax.id.segments]
[example _trace.syntax.id.minimum]
[example _trace.syntax.id.hierarchy]
Valid IDs:
- `foo` (single segment - satisfies minimum requirement)
- `foo.bar` (two segments - `bar` is child of `foo`)
- `foo.bar.baz-qux` (with dash in segment)
- `my_module.some_feature` (with underscores in segments)

Invalid IDs:
- `.foo` (empty first segment violates segments rule)
- `foo.` (empty last segment)
- `foo..bar` (empty middle segment)
- `foo.bar!` (invalid character - only alphanumeric, dash, underscore allowed)

[example _trace.satisfaction.modifiers]
```
[def api.endpoints +doc]          # requires impl, test, and doc
[def api.endpoints.list -test]    # inherits +doc, so requires impl and doc only
[def internal.utils -impl -test]  # requires nothing (documentation-only)
```

[example _trace.context.boundaries]
Given this file:
```
// The authentication module handles user login.
// [def auth.login]
// Users must provide valid credentials.

// [def auth.logout]
// Terminates the user session.
```

The context for `auth.login` is:
```
The authentication module handles user login.
[def auth.login]
Users must provide valid credentials.
```

The blank line separates it from `auth.logout`.

[example _trace.context.prefix-strip]
Given:
```
// [def feature.foo]
// This feature does something.
// It has multiple lines.
```

All lines share `// ` prefix, so the extracted context becomes:
```
[def feature.foo]
This feature does something.
It has multiple lines.
```

---

## CLI Enhancements

[def _trace.cli.description]
When run with no arguments (default output), the CLI includes a brief (2-3 sentence) description of what the tool is and how it works, sufficient for an AI agent to understand its purpose without additional context. The description should mention the `--install-skills` option to help AI agents discover the available skills.

[def _trace.cli.install-skills]
The CLI provides an `--install-skills` option that installs Claude Code skill files into the current project's `.claude/skills/` directory. The command fails with an error if any target files already exist, preventing accidental overwrites.

---

## Claude Code Skills

[def _trace.skills @children -test]
Requirements for Claude Code skills that help AI agents work with the requirements system. Skills are documentation for AI agents and cannot be programmatically tested.

[def _trace.skills.embedded]
Skill files are embedded in the binary at compile time and can be installed via `--install-skills`.

[def _trace.skills.self-contained]
Each skill must contain enough context about the _trace model and workflow that an AI agent can effectively use the tool with no other documentation. Skills should explain: annotation syntax, requirement hierarchy, satisfaction modes, and the iterative workflow.

[def _trace.skills.validate]
A skill for deep validation of requirements - checking whether implementations semantically satisfy their definitions, not just whether annotations exist.

[def _trace.skills.work]
A skill for autonomous work on requirements - selecting unsatisfied requirements, implementing or testing them, and iterating until satisfied.

[def _trace.skills.add]
A skill for adding or updating requirements with careful attention to clarity, proper placement in the hierarchy, and ensuring complete mutual understanding between the agent and user before committing changes.

---

## Future Considerations

The following are explicitly out of scope for the initial release but may be added later:

- **RFC 2119 keywords**: Support for MUST, MUST NOT, SHOULD, SHOULD NOT, MAY in requirement descriptions, with validation.
- **Google Docs import**: Importing requirements from Google Docs.
- **Configuration file**: Custom glob patterns, default satisfaction criteria, output format preferences.
- **Watch mode**: Continuous monitoring for changes.
- **Machine-readable output**: JSON or other structured output formats.
- **Git index mode**: A `--git-index` flag to read files from the git index/tree instead of the filesystem directly, enabling analysis of staged changes or specific commits. The current working directory is still respected—scanning starts from the corresponding subdirectory in the index tree.
