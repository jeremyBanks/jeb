> ⚠️ **please note** that this is mostly just brainstorming, not describing what
> actually exists.

---

next goals:

- [ ] priorities: queries with NO dependencies, so we don't have to worry about
      context doing much complicated.
- [ ] sqlite storage layer with separate blobs and queries, but not much smarts.
      (maybe we merge later, but maybe not.)
- [ ] cache configuration can come later
- [ ] queries for web caching
- [ ] and royal road ingestion
- [ ] and per-chapter text-to-speech production of ogg files
- [ ] nothing else for now, get this working, merge it

---

## 📖 Fiction app

i want a tool to convert eBooks to audiobooks, packaged as private podcasts, for
my personal convenience.

## features

- Reads eBooks from
  - EPUB files
  - archiveofourown.org
  - fanfiction.net
  - royalroad.com
- Generates:
  - Opus audio using Microsoft's text-to-speech and FFMPEG.
  - EPUB with embedded aligned synchronized audio.
  - MKV/WebM files, with embedded synchronized text, both individually
    per-chapter and as a single file with embedded chapter markers.
  - postcast RSS feeds indexing the generated media files.

## ⚙️ Incremental engine

fiction's engine is a sort-of incremental-computation content-based-addressing
query-driven caching engine. Built on top of SQLite and Postcard for now, maybe
using Protocol Buffers later. The main thing distinguishing it from alternatives
like Salsa and Turbo is that it sucks. Secondary to that, fiction is focused on
data that can be cached long-term, to disk, while Salsa seems to be focused on
data that's stored in-memory, and Turbo confused me. Aiming for some kind of
Turbo extension in the future might make sense.

Every operation in the engine is represented by a Query, which has a Request
type and a Response type. The database stores Request-Response pairs, with
associated Metadata indicating what dependencies the response had when being
built.

### Storage Layers

An implementation of the Storage layer will provide one or both of:

1. **Blobs** Content-addressed storage of byte strings, between 32B and 1 GiB in
   size, each identified by its 32B BLAKE3 hash digest.
2. **Queries** Storage of Request-Response pairs, each 32B (addressing a stored
   blob) or less (a short inlined value). Each pair should include `inserted_at`
   and `validated_at` fields, indicating the first and most-recent times that a
   given Request-Response pair was inserted. Should be indexed by `Request`
   hashed, followed by the `validated_at` timestamp descending, so that the most
   recent known `Response` can quickly be found for a given `Request`.

Depending on the desired behaviour, a storage implementation may drop items, or
only save the most recent, or similar. They may be read-only and not support
writing.

The built-in storage layer implementations include:

- entirely in-memory
- SQLite (in-memory or on disk)
- Web (read-only via HTTP(S))
- Static (read-only compiled-in with databake)

and a meta layer type that can be used to read/write from multiple different
storage implementations with customizable behaviour.

### Requests and Responses

They're serialized with Postcard now but I'd like to do something that's
serde-compatible while at least being forward-compatible with protocol buffers.

Request and Response types should themselves be relatively small. Any large data
should be serialized as a Blob and the Request or Response should include the
Blob IDs instead.

## 👑 Ser Proto serialization

a **ser**de serialization implementation using a subset of **proto**col buffers
wire format (though not its schemas).

The encoding is canonical/deterministic, with the following rules:

- Field IDs are always sorted in an object.
- All non-repeated integers types larger than `u8`/`i8` are encoded are varints.
- Signed values are
- Repeated integers are encoded as fixed-length packed (not varints!) values
  only if that would be shorter than the non-packed varint representation.
- Missing values

Field IDs are automatically generated assigned based on field order, starting
at 1. However, they can also be manually specified by naming/renaming a field to
the number prefixed with an underscore, such as `_0`, `_2`, or `_9001`.

Unit enum values or default/zero enum values are represented by a varint
encoding their discriminant. Non-default/non-unit enum values are encoded as a
submessage with a single field whose ID corresponds to 1 + the discriminant
value (post-zig-zag encoding if signed (which is the case for custom
discriminant values with the default representation)). If the serde data model
allows this. It might not.

Hmm. But you can't generate a proto schema for that, since varints and
fixed-length integers are different types. I guess we have to be consistent...
maybe varints for all integers, and the fixed-length types are only used for
`f32` and `f64`. Or we could also use them for 32- and 64-bit integer types,
while using varints for 8-, 16-, and 128-bit integers. But if we want to allow
integers sizes to be expanded, they should all be varints... but doing that for
u8 data seems really damn stupid. Maybe that's where we use serde_bytes instead,
actually!

Per postcard, we should store f32 as little-endian bytes explicitly for
endian-agnosticism.

I'd like to drop zero-valued fields in encoding, but I'm not sure if
deserializing those values works with serde without manually using the
default-related attributes. If it can be done, I'd like to, _instead_ of
supporting the serde `#[default]` attribute at all. (I don't think formats like
Postcard support it.) Unit values should not be encoded.

### Protocol Buffers Incompatibilities

Ser Proto doesn't enforce some requirements for Protocol Buffers messages, so
you may create incompatible messages from the following:

- Protocol Buffers' variable-length integers are limited to 64-bit values. We
  also allow you to also use them for 128-bit values (`u128`/`i128`).

- Protocol Buffers length-prefixed data is limited to lengths that can fit in 31
  bits, which is equal to 2GiB. We do not enforce enforce this limit.

- Protocol Buffers requires field IDs in the range `1-18,999` or
  `20,000-536,870,911`. We allow you to (manually) specify any non-negative
  field ID that fits in a u128, including `0`.

### Related

- https://postcard.jamesmunns.com/wire-format.html
- https://developers.google.com/protocol-buffers/docs/encoding
- https://postcard.jamesmunns.com/serde-data-model.html
