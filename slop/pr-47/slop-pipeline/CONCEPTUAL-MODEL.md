# jeb: Conceptual Model

jeb is a command-line tool for processing collections of loosely structured data. It follows the Unix philosophy of composing small operations, but extends it with a richer data model and more forgiving execution semantics.

## Pipeline Structure

A jeb command is a sequence of subcommands. Data flows left-to-right through the pipeline:

```
jeb stdin from-base64 from-json sort to-json stdout
```

The command line is parsed synchronously to build a directed acyclic graph (DAG) of nodes. Each node is a subcommand with a statically known number of inputs and outputs. Once the graph is constructed, nodes execute asynchronously as independent tasks.

### Connection Rules

Most commands have one input and one output, forming a simple linear chain. Commands with multiple inputs or outputs create branching structures. Connections are resolved using stack-like semantics during parsing: when a multi-input command appears, it consumes unconnected outputs from right to left.

```
jeb ./foo.xml parse-xml ./bar.json parse-json chain to-json stdout
```

Here, `./foo.xml` and `./bar.json` are sources (no inputs, one output each). Their outputs remain unconnected until `chain`, which accepts any number of inputs and consumes both. The order is left-to-right: foo's output comes before bar's in the combined stream.

### Implicit Commands

jeb inserts implicit commands to ensure pipelines are well-formed:

- If no source exists, `stdin` is prepended
- If no sink exists, `stdout` is appended
- If unconnected outputs remain before the final sink, `chain` is inserted to combine them

This means `jeb sort` is equivalent to `jeb stdin sort stdout`.

## Data Model

Data flows between nodes as streams of items. Each item has one of three top-level types.

### Text

A Unicode string (UTF-8). Represents unparsed textual data—the kind of thing you might grep or split by lines. Text captures the notion of "raw character data that hasn't been interpreted as a data structure yet."

### Bytes

A binary string (byte array). Represents unparsed binary data. This is the rawest form of data in the system.

### Structured

A value in an extended JSON data model:

- Primitives: null, true, false
- Numbers: 64-bit signed integers, 64-bit unsigned integers, 64-bit finite floats
- Strings: text (Unicode) or binary (byte array)
- Arrays: ordered sequences of any Structured values
- Maps: ordered key-value collections. Each map is either text-keyed or binary-keyed—key types are homogeneous within a map, but nested maps may use different key types

Maps preserve insertion order. This matters for serialization, comparison, and merging operations—all of which are order-sensitive by default.

### The Parsed/Unparsed Distinction

Text and Bytes represent data that hasn't been parsed into structure. A Structured text string and a top-level Text item might contain the same characters, but they carry different semantic meaning: one is "a string value within structured data," the other is "raw text waiting to be interpreted."

This distinction drives coercion behavior. When a command expecting Structured input receives Text, jeb will typically wrap it as a Structured string, emitting a warning and noting the implicit conversion.

## Chunking and Boundaries

For Structured data, item boundaries are unambiguous: one value, one item.

For Text and Bytes, boundaries must be established explicitly. The `stdin` source emits arbitrarily-sized byte chunks (reflecting how real I/O works). Commands like `by-lines` and `by-null` re-chunk the stream by a delimiter, preserving the delimiter at the end of each chunk. The `split-lines` and `split-null` variants consume the delimiter.

```
jeb stdin by-lines           # chunks ending with \n
jeb stdin split-lines        # chunks with \n removed
```

Parser commands like `parse-json` are designed to reassemble arbitrarily-chunked input internally. `parse-json` accepts one or more JSON values with optional whitespace between them, naturally supporting both single documents and JSON Lines format.

Commands that produce multiple items from one (like `split-array`, which emits each element of an array as a separate item) and commands that combine items (like `join-array`, which collects a stream into a single array) control how data aggregates or disperses through the pipeline.

## Pipeline Visualization

At startup, jeb displays the pipeline as parsed. The stack-based connection rules guarantee that stream connections never cross, making ASCII visualization straightforward.

The display uses line art that hard-wraps at the terminal width and continues on the left below. Output streams appear above nodes in dark blue, error streams below nodes in dark red, with error-handling nodes below that. This vertical separation keeps the main data flow visually primary while making the error pipeline available when relevant.

When implicit coercions occur, jeb can show the pipeline with equivalent explicit coercions highlighted, revealing what actually happened:

```
jeb stdin sort stdout
# might display:
# stdin [to-text-lines] sort stdout
# if sort received Bytes and coerced them to Text lines
```

This teaches users the explicit form they could write to avoid warnings in the future.

The error pipeline is only displayed if the user has customized it or if errors actually occurred during execution. Fully implicit error handling with no errors produces no extra output.

## Execution Philosophy

jeb tries to produce useful output even when inputs are unexpected. Rather than failing on the first type mismatch or malformed data, it attempts coercion, wrapping, or skipping to keep the stream flowing.

### Warnings and Coercions

When jeb performs an implicit coercion or handles unexpected input, it emits a warning and sets the exit status based on the node index (63 + index, capped at 96).

### Error Handling

Errors during execution (malformed input, unexpected values) generally do not abort the stream. A command encountering bad data will typically:

1. Emit what it can (possibly coerced or wrapped)
1. Skip to the next item
1. Send an error to the error stream

Only truly unrecoverable situations terminate output. File system errors and similar external failures are outside this model and will crash the program.

### Memory

Most nodes do not buffer, allowing bounded memory usage even on large streams. Some operations inherently require buffering (sorting, for example), but the default philosophy is to stream when possible.

## Error Pipeline

Errors are not printed directly to stderr. Instead, each node has a hidden error output carrying structured error values. These are maps with text keys containing information like node index, message, and context.

The error stream follows the same implicit-completion rules as the main pipeline. Unconnected error outputs are gathered by an implicit `chain-errors`, formatted by `format-errors` (which converts structured error values to human-readable text), and sent to `stderr`.

This design means error handling is composable. Just like any other stream, future commands could filter, format, or route errors.

### Exit Status

Exit status encodes where the first error occurred:

- 0: success, no warnings
- 63: warnings or errors from the first node (index 0)
- 64: first error from node at index 1
- And so on, up to 96

This lets scripts distinguish "the parser failed" from "the serializer failed" without parsing error messages.

## Example Commands

These illustrate the model rather than prescribe exact behavior.

**Sources** (no inputs, one output): `stdin`, `./path` (file literal)

**Sinks** (one input, no outputs): `stdout`, `stderr`

**Parsers**: `parse-json`, `parse-xml`, `from-base64` — consume Bytes/Text, emit Structured

**Serializers**: `to-json`, `to-base64` — consume Structured, emit Text/Bytes

**Chunking**: `by-lines`, `split-lines`, `by-null`, `split-null`, `join-lines`, `join-null`

**Aggregation**: `join-array` (stream → single array), `split-array` (array → stream of elements)

**Combining streams**: `chain` (concatenates streams in order), `merge` (interleaves by a global item ordering)

**Transforms**: `sort`, `filter`, `map` — operate on streams, behavior varies by item type with coercion as needed

## Summary

jeb extends Unix pipelines with a three-type data model (Text, Bytes, Structured), stream-oriented DAG execution, and forgiving semantics that prefer coercion over failure. The command line maps to a graph via stack-based connection rules, with implicit commands ensuring well-formed pipelines. Errors flow through a parallel composable pipeline. The system is designed for exploring and transforming loosely structured data, surfacing implicit behavior so users can learn and refine their pipelines over time.
