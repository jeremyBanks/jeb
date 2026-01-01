# Error Test Cases TODO

This document lists error conditions from the IDEA.md and IDEA-1.1.md specifications
that should be tested but require a different test infrastructure (tests that expect
parsing/validation failures).

## Implementation Note

The current test infrastructure in `fixtures.rs` expects all fixtures to parse
successfully and verify round-trip stability. To test error conditions, we would
need to add:

```rust
// Suggested approach: files ending in .error.yaml
if input_path.to_str().unwrap().contains(".error.") {
    // Expect parsing to fail
    let result = parse(&input_yaml);
    assert!(result.is_err(), "Expected parsing to fail for {:?}", input_path);
    continue;
}
```

## Required Error Test Cases

### 1. Invalid Tree Entry Names (IDEA.md lines 195-207)

**error-01-slash-in-name.int.error.yaml**
```yaml
1:
  tree:
    "path/with/slash": "invalid"
```

**error-02-backslash-in-name.int.error.yaml**
```yaml
1:
  tree:
    "path\\with\\backslash": "invalid"
```

**error-03-colon-in-name.int.error.yaml**
```yaml
1:
  tree:
    "C:invalid": "invalid"
```

**error-04-dot-name.int.error.yaml**
```yaml
1:
  tree:
    ".": "invalid"
```

**error-05-dotdot-name.int.error.yaml**
```yaml
1:
  tree:
    "..": "invalid"
```

**error-06-empty-name.int.error.yaml**
```yaml
1:
  tree:
    "": "invalid"
```

### 2. Commit Reference Cycles (IDEA.md lines 269-270)

**error-07-self-reference.int.error.yaml**
```yaml
1:
  parents: [1]
  tree:
    file.txt: "cycle"
```

**error-08-circular-parents.int.error.yaml**
```yaml
1:
  parents: [2]
  tree: {}

2:
  parents: [1]
  tree: {}
```

**error-09-three-way-cycle.int.error.yaml**
```yaml
1:
  parents: [3]
  tree: {}

2:
  parents: [1]
  tree: {}

3:
  parents: [2]
  tree: {}
```

### 3. YAML Tag Rejection (IDEA.md error conditions)

**error-10-binary-tag.int.error.yaml**
```yaml
1:
  tree:
    file.txt: !!binary "AQIDBA=="
```

**error-11-str-tag.int.error.yaml**
```yaml
1:
  tree:
    file.txt: !!str "tagged string"
```

### 4. Special YAML Values (IDEA.md)

**error-12-infinity.int.error.yaml**
```yaml
1:
  tree:
    value: .inf
```

**error-13-nan.int.error.yaml**
```yaml
1:
  tree:
    value: .nan
```

### 5. Date Validation (IDEA.md lines 125-149)

**error-14-date-before-epoch.int.error.yaml**
```yaml
1:
  author-date: 1969-12-31T23:59:59Z
  tree: {}
```

**error-15-date-with-fractional-seconds.int.error.yaml**
```yaml
1:
  author-date: 2021-01-14T08:25:36.123Z
  tree: {}
```

### 6. IDEA-1.1 Reference Errors

**error-16-path-without-commit.int.error.yaml**
```yaml
1:
  tree:
    file.txt: "content"

2:
  tree:
    broken:
      [path]: file.txt
      # Error: [path] without [commit]
```

**error-17-reference-to-nonexistent-path.int.error.yaml**
```yaml
1:
  tree:
    file.txt: "content"

2:
  tree:
    broken:
      [commit]: 1
      [path]: does/not/exist
```

**error-18-blob-with-extra-keys.int.error.yaml**
```yaml
1:
  tree:
    file.txt: "blob content"

2:
  tree:
    ref:
      [commit]: 1
      [path]: file.txt
      extra: "not allowed"
```

**error-19-dotdot-past-root.int.error.yaml**
```yaml
1:
  tree:
    file.txt: "content"

2:
  tree:
    broken:
      [commit]: 1
      [path]: ../../../past-root
```

**error-20-commit-null-with-path.int.error.yaml**
```yaml
1:
  tree:
    file.txt: "content"

2:
  tree:
    broken:
      [commit]: null
      [path]: file.txt
```

### 7. Hash Validation (IDEA-1.1)

**error-21-ambiguous-truncated-hash.hex.error.yaml**
```yaml
# Would need commits with hash collision in truncated form
# This is hard to create without brute-forcing hashes
```

**error-22-invalid-hex-characters.hex.error.yaml**
```yaml
HEAD: "gggggggggggggggggggggggggggggggggggggggg"
refs:
  heads:
    main: 1
1:
  tree: {}
```

**error-23-odd-length-hex.hex.error.yaml**
```yaml
HEAD: "abc"
refs:
  heads:
    main: 1
1:
  tree: {}
```

### 8. Type Validation

**error-24-wrong-type-parents.int.error.yaml**
```yaml
1:
  parents: "should be array"
  tree: {}
```

**error-25-wrong-type-tree.int.error.yaml**
```yaml
1:
  tree: "should be mapping"
```

**error-26-wrong-type-refs.int.error.yaml**
```yaml
HEAD: refs/heads/main
refs: "should be mapping"
```

### 9. Unknown Keys (IDEA.md)

**error-27-unknown-top-level-key.int.error.yaml**
```yaml
HEAD: refs/heads/main
refs:
  heads:
    main: 1
unknown-key: "not allowed"
1:
  tree: {}
```

**error-28-unknown-commit-key.int.error.yaml**
```yaml
1:
  tree: {}
  unknown-field: "not allowed"
```

## Summary

- **28 error test cases** identified from specifications
- All require test infrastructure modification to expect failures
- Priority: High for specification compliance validation
- Recommended approach: Add error test support to fixtures.rs first
