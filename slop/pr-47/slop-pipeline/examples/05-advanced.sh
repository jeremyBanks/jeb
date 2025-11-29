#!/bin/bash
# Example 5: Advanced pipeline features

echo "=== Example 1: Filter and transform data ==="
echo -e 'hello\n\nworld\n\nfrom jeb' | jeb split-lines filter join-space
echo

echo "=== Example 2: First and last items ==="
echo -e 'a\nb\nc\nd\ne' | jeb split-lines first last join-space
echo

echo "=== Example 3: Working with structured data ==="
cat <<EOF | jeb parse-json split-array first to-json
[
  {"name": "Alice", "age": 30},
  {"name": "Bob", "age": 25},
  {"name": "Charlie", "age": 35}
]
EOF
echo

echo "=== Example 4: Combining multiple operations ==="
echo -e '  spaced   text  \n  more   spaces  ' | jeb split-lines collapse join-lines
echo

echo "=== Example 5: Exit status demonstration ==="
# This will show warnings/errors if JSON is malformed
echo 'invalid json' | jeb parse-json to-json 2>&1
echo "Exit code: $?"
echo

echo "=== Example 6: Processing with sort ==="
echo -e 'zebra\napple\nmango\nbanana' | jeb split-lines sort join-lines
echo
