# git-snapshot

This is some loose plan/ideation for a library for snapshotting git repositories
for testing.

## Version 1

For version 1, we're not actually interacting with real git at all, we're only
interacting with our serialized and in-memory representations (which are quite
different from each other).

We're going to use `serde_yaml`, but we're not using any actual `Serialize` or
`Deserialize` derived implementations, we're just going to use dynamic
`serde_yaml::Value` values in our own parsing logic, and write them to disk.

### On-Disk Representation

The on-disk representation is designed to be human-readable and writeable, with
a focus on minimal duplication and sensible defaults. It will be a YAML 1.2 file
using UTF-8 encoding (but no `%` header or anything, we'll use parsers that are
configured to use 1.2 without requiring it).

In the on-disk representation, each commit must be consistently referenced by
either its full real correct 40-character hex object ID as a `Value::String`, or
a unique positive integer `Value::Number`. (The integer IDs never exist
in-memory, they're purely an on-disk serialization thing. We may mix integer and
hex styles in the same file, as long as each commit is consistently referenced
with one or the other.) Our serialization function will take a parameter
controlling whether it uses full object IDs, or whether it uses integer IDs (in
which case they'll be assigned starting at 1 in the order in which the commits
appear in the file, which is specified below). Note that we're not using any
Yaml types that couldn't exist in JSON, except for numeric map keys. Things like
dates are always serialized and deserialized as strings or numbers; if any
syntax examples below imply otherwise they're incorrect.

The top-level of the on-disk representation is `Value::Mapping` starting with
two reserved keys: 1. `HEAD`, which is either a string starting with "refs/", or
else it's a commit reference. 2. `refs`, which is a nested mapping whose leaves
are commit references and whose key paths are their ref path. (For V1, the only
refs we include are branches under `refs/heads/`; we do not include tags, or
remote refs, or notes, or anything else like that. If HEAD specifies a path that
does not exist under `refs`, that implies it's an unborn branch, like in Git's
normal model.) For example:

```yaml
HEAD: refs/heads/trunk
refs:
  heads:
    dev: 456
    trunk: 123
# ...
```

Every remaining item in the map is a commit, with the key being its commit
reference (so either a 40-character hex string or a positive integer), and the
value being a mapping representing the contents of the commit. We have many
defaults and options to help make this representation more compact by omitting
redundant details. The idea is that, anywhere that our deserializer's default
would already do the right thing, we omit the value in our serialized
representation to save space.

The most basic default: if a value in a mapping is itself an empty mapping or
empty array, then we just omit the value (so instead of `x: []` we just have
`x:`, which YAML interprets as `null`, which we interpret during parsing as an
empty object or array, which is _NOT_ the same as the item not being present in
the mapping, for which we often define different behavior.) If we're expecting
something other than an array or a mapping (such a string or a number), this is
_not_ allowed and a null or missing value is an error during deserialization.
YAML `!!` tags are _not_ permitted and always trigger an error if present. (YAML
anchors/aliases, we're neutral on: we will never produce them in our serializer,
and we won't go out of our way to support them, but if they end up being
implicitly supported due to the behavior of the YAML library we choose to use,
that's fine, we don't need to actively prevent it.)

We'll go over the specific meaning and behavior of each field one at a time.

```yaml
# ...
1:
  parents: []
  # ...
```

`parents` is an array of commit references. If not present, it defaults to an
array containing a reference to the previous commit in document order. If this
is the first commit in the list, then it's empty.

```yaml
# ...
1:
  # ...
  message: "initial commit"
# ...
```

`message` is a string. If absent, then if the commit is referenced by an integer
then the commit message is "commit N" where `N` is that integer. Otherwise, the
default is "commit at <commit date as ISO 8601 string>". As you would expect,
this may be a multi-line string and may include trailers. The explicit empty
string is also valid, meaning a commit with no message (NOT the default
message).

```yaml
# ...
1:
  # ...
  author: "Example <example@example.com>"
  committer: "Example <example@example.com>"
# ...
```

`author` and `committer` are strings with the expected format (effectively
`/^[^<]+[ ]<[^@]+@[^>]+>$/` although we don't actually need regexes to handle
that). If `committer` is absent, it defaults to `author`. If `author` is absent,
it defaults to the author of the first-parent commit. If there are no parents,
`author` defaults to `User <user@localhost>`.

```yaml
# ...
1:
  # ...
  author-date: 2021-01-14T08:25:36Z
  commit-date: 2021-01-14T14:25:36-0200
# ...
```

`author-date` and `committer-date` are strings representing the commit timestamp
as ISO 8601 strings (with no sub-second component, because git timestamps only
have full-second resolution). If absent, then for the initial commit
`author-date` defaults to `2021-01-14T08:25:36Z`. For non-initial commits, it
defaults to `256` seconds after the maximum `author-date` among parent commits,
copying the timestamp from that maximum (or from the first tying parent if
multiple parents have the same timestamp but different offsets). If absent,
`commit-date` defaults to `3` seconds after the maximum of this `author-date`
and the `commit-date`s among parents commits (also copying from the first tying
parent unless the `author-date` is tied for first, in which case we use it).
When serializing use `Z` for the zero/UTC offset, and the colon form like
`-02:00` for nonzero offsets, separate date components with `-`, time components
with `:`, and join them with a `T`.

When parsing, we accept any valid ISO 8601 date or datetime (so we accept other
valid delimiters), as long as it's a time git can support (so it can't be before
the Unix epoch, and it can't have nonzero fractional seconds). At minimum, the
year needs to be specified, but any number of other trailing components can be
omitted (as long as they're all trailing, with no gaps in between). We default
to month 02, day 04, hour 08, minute 16, second 32, and offset Z (UTC / 0).

```yaml
# ...
1:
  # ...
  tree:
    README: "# example"
    src:
      target:
      main.bash: "echo hello world"
# ...
```

`tree` is the most complicated case to avoid redundancy. It defaults to the same
tree as the first parent, or the empty tree if this is the first commit, but
even if a value is specified it _usually_ represents a change that is applied on
top of that default, rather than entirely replacing it as our other defaults do.
This expression is evaluated recursively.

The root `tree` is a mapping from strings to non-empty recursive mappings (to
represent nested directories/trees), or strings (to represent blob contents), or
`null`/absent/empty mappings (to represent deletion). When serializing, we
prefer the absent format where there's a key but not value, to represent
deletion, instead of `null` or `{}`, but they're both supported when parsing.

A value of `null`/missing/the empty object `{}`, that represents deletion. If
it's the root tree for the commit, it means the commit contains no files/has the
empty tree as its root. (The empty tree cannot exist anywhere but the root, so
this unambiguously represents deletion anywhere else.) If it's a nested tree
entry, it means that the file or tree at that path is deleted, it if even
existed.

A value which is a string represents the contents of a blob (must be UTF-8). If
any blob or tree already existed there, it's replaced with this new blob. The
empty string represents the empty blob. (We don't support tags like `!!binary`
so we don't support binary blobs at all: they must all be valid UTF-8.)

A non-empty object represents a directory/tree. Any existing entries which are
not named in the new object are _left intact/as-is/not-modified_. (Hence the
empty object is a special case, because normally we're not deleting unmentioned
keys.) Any entries that are modified will be listed here, as either an
empty/`null`/`{}` for deletion, a string for a blob, or a mapping with the
changes to apply to a tree. (If we have mapping object in the path where a blob
previously was, the mapping replaces the blob.)

Tree entry names (file/directory names) must not contain `/`, `\`, `:`, null
bytes, or be equal to `"."`, `".."`, or `""`, or we return an error. Names will
always be serialized as strings. For convenience to humans authors, when
parsing, we also accept _integer_ numbers by converted to their integer string
representations (not fractional/float values because we don't want to deal with
the ambiguities of floating point stringification), and name values which are
equal to `true`, `false`, or `null` are interpreted as the corresponding strings
`"true"`, `"false"` or `"null"`. (Due to the YAML parser, this implicitly means
that `True` will become `"true"`, and `~` will become `"null"`, and we tolerate
that as an unfortunate edge case, which will never occur in data we've
serialized ourselves.) Other special YAML symbols/keywords such as `.inf` and
`.nan` are not supported and result in errors. All of this same logic is also
applied to keys in our `refs` mappings, too.

This scheme has no way to store non-default flags, such as whether a files is
executable or a symlink. Those files are not supported.

This scheme can produce the cleanest results if the files are well-ordered, so
when serializing we will sort them according to the following rules (although
note that we do _not_ enforce this while deserializing, we need to be able to
handle arbitrary ordering).

We perform a topological sort, with the topologically "highest" commits coming
later, further down in the file (so a commit will always occur later than its
parents). However, topological orderings are not total orderings, and we require
a deterministic total ordering, so for each node we're also going to calculate a
"tiebreaking key" which is used to determine the ordering when topology gives us
multiple options. The tiebreaking key is an array of values (compared in the
typical way: one after another, lexicographically).

Let the tiebreaking key arrays start empty for every commit. Then take all of
our head commits (any commits directly referenced under `refs:` or `HEAD:`, with
the `HEAD` (unless unborn) coming first followed by the refs in lexicographic
order by their paths). Deduplicate them then, one by one, walk their ancestors,
depth first. For each commit we visit, we append N to its tiebreaking key, where
N is the index of this commit among the parents of the commit we reached it
from. If we find ourselves following a parent reference to a commit we've
already visited, then we _do_ record that parent reference index in its
tiebreaking key, but we don't visit further. (The idea is that we _do_ want to
consider every _edge_ in the graph, and if we're coming in from a new parent
that's a new edge, but if we keep walking up parents from _there_, we'll just be
visiting edges we've already considered.)

After we've fully walked all ancestors, then finally go through all commits and
append the final tiebreaking components: their commit timestamp, followed by
their authoring timestamp. This gives us a topological sort which will also
reflect the git structure in a way that prioritizes the HEAD branch but includes
all of them, with the most-recent commits appearing after their ancestors, and
merged branches usually appearing after the first-children chains they branched
off of.

Our in-memory representation will not preserve details of how the input is
formatted. It won't preserve the order in which commits were listed, or the
order of keys in mappings. It can't distinguish a value with an inferred default
from a case where it was hard-coded, because the default will have already been
filled-in. Those kinds of details only exist in the on-disk representation, so
our serializer needs to be able to recreate/normalize whatever of them we want
in our output.

During serializing, all mappings should have their keys sorted
lexicographically.

During serialization, we omit all fields where the default inferred value is the
same as the value we were going to explicitly set. This is critical to the
overall intent and design of this approach. If the author is the same on every
commit, you only need to specify it on the initial commit, not repeat it every
time. If you've only edited one file in a tree, we just want to indicate the new
contents for that file, not reserialize the entire tree.

This format must never include references to commits/objects that are not also
included, i.e. it can't represent shallow clones which could give us
locally-dangling references.

Cycles and self-references should be detected for controlled errors, rather than
a potential infinite loop or stack overflow.

When we're serializing, we implicitly only include commits which are reachable
from our `refs` or `HEAD`. When we're deserializing, non-reachable commits are
ignored (they do not raise an error, but they're not preserved).

During serialization, unexpected object entries (such as a top-level key that's
not `HEAD`, `refs`, an integer, or a 40-character hex string, or a value of the
wrong type, or a top-level like `this-is-not-defined` under a commit), or values
of unexpected types (beyond explicitly-described edge case/lenience handling
above) result in an error.

When serializing or deserializing, if anything that we say above must not or
cannot or will not happen, does happen, that's an error.

### In-Memory Representation

TODO

## Version 2

(We are NOT implementing or discussing this in detail yet. This is just for
future reference, but at this time we are focused entirely on Version 1.)

In the future, we'll have methods for reading from a `git2::Repository`, and
writing to one if it's empty, as well as for creating one in a new temporary
directory and returning a handle to it, and similar things, to make it as easy
as practical to use this for testing with real git. But that depends on us
getting Version 1 right first, before we worry about these details!

If we encounter any data data that is not supported by our on-disk
representation, then we ignore it if we can (for example, we can omit tags and
nothing else is corrupted) or raise an error (for example, if we encounter a
file that's marked executable, we can't add it without losing data, so we need
to raise an error and abort).
