#!/bin/bash
# Example 1: Basic JSON processing

echo "=== Example 1: Parse and pretty-print JSON ==="
echo '{"name":"Alice","age":30}' | jeb parse-json to-json
echo

echo "=== Example 2: Process JSON from file ==="
echo '[{"id":3},{"id":1},{"id":2}]' > /tmp/data.json
jeb /tmp/data.json parse-json sort to-json
echo

echo "=== Example 3: Multiple JSON objects (JSON Lines) ==="
echo -e '{"a":1}\n{"a":2}\n{"a":3}' | jeb parse-json join-array to-json
echo

echo "=== Example 4: Split JSON array into individual items ==="
echo '[1,2,3,4,5]' | jeb parse-json split-array to-json
echo
