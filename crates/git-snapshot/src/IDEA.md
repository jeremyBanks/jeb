# git-snapshot

This is some loose plan/ideation for a library for snapshotting git repositories
for testing.

## Version 1

For version 1, we're not actually interacting with real git at all, we're only
interacting with our serialized and in-memory representations (which are quite
different from each other).

We're going to use `serde`, but we're not using any actual `Serialize` or
`Deserialize` derived implementations, we're just going to use dynamic
`serde_yaml::Value` values in our own parsing logic, and write them to disk.

### On-Disk Representation

The on-disk representation is designed to be human-readable and writeable, with
a focus on minimal duplication and sensible defaults.

In the on-disk representation, each commit must be consistently referenced by
either its full real correct 40-character hex object ID as a `Value::String`, or
a unique positive integer `Value::Number`. (The integer IDs never exist
in-memory, they're purely an on-disk serialization thing.) Our serialization
function will take a parameter controlling whether it uses full object IDs, or
whether it uses integer IDs (in which case they'll be assigned starting at 1 in
the order in which the commits appear in the file, which is specified below).
Note that we're not using any Yaml types that couldn't exist in JSON, except for
numeric map keys. Things like dates are always serialized and deserialized as
strings or numbers; if any syntax examples below imply otherwise they're
incorrect.

The top-level of the on-disk representation is `Value::Mapping` starting with
two reserved keys: 1. `HEAD`, which is either a string starting with "refs/", or
else it's a commit reference, and 2. `refs`, which is a nested mapping whose
leaves are commit references and whose key paths are their ref path. (For V1,
the only refs we include are branches under `refs/heads/`; we do not include
tags, or remote refs, or notes, or anything else like that.) For example:

```yaml
HEAD: refs/heads/trunk
refs:
  heads:
    trunk: 123
    dev: 456
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

We'll go over the specific meaning and behavior of each field one at a time.

```yaml
# ...
1:
  parents: []
  # ...
```

`parents` is an array of commit references. If not present, it defaults to an
array containing a reference to the previous commit in the list. If this is the
first commit in the list, then it's empty.

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
this may be a multi-line string and may include trailers.

```yaml
# ...
1:
  # ...
  author: "Example <example@example.com>"
  committer: "Example <example@example.com>"
# ...
```

`author` and `committer` are strings with the expected format. If `committer` is
absent, it defaults to `author`. If `author` is absent, it defaults to the
author of the first-parent commit. If there are no parents, `author` defaults to
`User <user@localhost>`.

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
defaults to `256` seconds after the maximum `author-date` among parent commits.
If absent, `commit-date` defaults to `3` seconds after the maximum of this
`author-date` and the `commit-date`s among parents commits.

```yaml
# ...
1:
  # ...
  author-date: 2021-01-14T08:25:36Z
  commit-date: 2021-01-14T14:25:36-0200
# ...
```

## Version 2

(We are NOT implementing or discussing this in detail yet. This is just for
future reference, but at this time we are focused entirely on Version 1.)

In the future, we'll have methods for reading from a `git2::Repository`, and
writing to one if it's empty, as well as for creating one in a new temporary
directory and returning a handle to it, and similar things, to make it as easy
as practical to use this for testing with real git. But that depends on us
getting Version 1 right first, before we worry about these details!
