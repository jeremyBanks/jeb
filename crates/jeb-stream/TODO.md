- as_value
- as_bytes
- as_text
- chunk(delimiter?: "\n", length?: 64KiB)
  - alias: lines = chunk(delimiter: "\n", length: null)
  - alias: paragraphs = = chunk(delimiter: "\n\n", length: null)

- box => is not value, make value, if value, wrap in {"":...}
- unbox => if {"":...} or Value::Text or Value::Bytes, unwrap, else error

- flat_map(f (lifts each item to a stream then applies a stream transform))
- map(f (single-item transform - maybe these names are too similar?))
- fail_fast (closes after first error)
- serde(f) => maps through another stream using serde to convert to and from the
  types it actually expects
- command(Command)

We _might_ want to define everything as both free functions and as methods on
our extensions of StreamExt/TryStreamExt.

---
