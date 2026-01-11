We search for the annotations in all files matching **/*.md or src/**/*.

Annotations are always wrapped in [...] (we're not using a suffix like `r` any more).
Annotations are not preceded by `]` or `)`,  nor followed by `[` or `(`.
Annotations always contain a period-separated list as their last space-separated component (this is the requirement identifier / ID).
An ID's (period-separated) components must each consist of at least one character, all of which must be alphanumeric or dashes or underscores.
An ID must have at least one component (if it's just one that implies no periods).
ID components represent a hierarchy (so order matters).
Each annotation's context consists of the line it's defined on and every line above or below until reaching a line that has no alphanumeric characters (this implies markdown paragraphs but is more general). If there is a common prefix of non-alphanumeric/dash/underscores/period characters for every line in the context, that is stripped from each line in the description.
There may be multiple annotations on the same line; there is no conflict. They will have the same description, that's fine.
Each annotations we record the file path, line (1-indexed), and column (1-indexed) that it's defined on (where the initial `[` is placed.

The following are recognized annotations types:

`def` defines a requirement. Each requirement ID may only be defined once. A def also implies the existence of its ancestor requirements (if no definition for an ancestor is found, it is treated as  if it exists with a simple default `[def some.ancestor.name]` with no associated description text.
It is a non-fatal error to define the same ID more than once. (For determinism, we'll pick the one whose path comes first lexicographically.)
Each definition has certain annotations types that it must be found with the same ID for it to be satisfied.
By default these are inherited from their parent. If no ancestors modify them, the default required annotation types are `impl` and `test`. Annotation types may be any string of at least one alphanumeric/underscore/dash characters (requiring a `def` is a bit silly because this _is_ the def, so it's a non-fatal error and we ignore it after emitting the message). Required annotations types are added or removed in a `def` for an requirement and its descendants by appending them, joined, with `+` and `-` indicators, as another component. It is a non-fatal error to attempt to remove a type that didn't exist or add one which already did.
[def some.id +impl+verify]
The final possible part is the special-case `@child`, `@self`, or `@either` (default if not defined by ancestors) as a final of the space-delimited components, defines whether the criteria need to be explicitly satisfied for this ID, or if they need to be satisfied for all children (and there needs to be at least one child), or the default: either one works.
It is a non-fatal error to add an annotation of a type that's not required.

We limit output to 32 items by default, but if there are more we end with a message telling the user they can use --skip and --limit to get more.
`_trace [optional.component.prefix or.more.than.one ...]`
lists requirements (always in order as a nested numbered list), each followed whether they're satisifed something like
---
- root: done with impl, test.
  - root.system-a: done with impl, test.
- alternate: has test, doc but missing impl
  - alternate.a: done with impl, test, doc
  - alternate.b: has test, doc but missing impl

Showing X of Y definitions. You may use `--limit` and `--skip` etc etc etc.

You may use `_trace --context` to see the context for every annotation, including paths/line/col. Or `_trace --lines` to just show paths/line/col, not context.  Or `_trace --context-of=def,test` to only expand the full context for specific types.
---
something like that
