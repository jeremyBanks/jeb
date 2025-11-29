#!/bin/bash
# Example 4: Pipeline architecture and visualization

echo "=== Example 1: Visualize a simple pipeline (dry-run) ==="
jeb --dry-run parse-json sort to-json
echo

echo "=== Example 2: Detailed pipeline visualization ==="
jeb --visualize-detailed parse-json to-json
echo

echo "=== Example 3: Complex pipeline with multiple stages ==="
echo '{"data": [3,1,2]}' | jeb --dry-run parse-json split-array sort join-array to-json
echo

echo "=== Example 4: Pipeline with implicit stdin and stdout ==="
# Note how stdin and stdout are automatically added (shown in red)
echo '{"x":1}' | jeb parse-json to-json
echo

echo "=== Example 5: Quiet mode (no visualization) ==="
echo '{"silent":true}' | jeb -q parse-json to-json
echo

echo "=== Example 6: Chain multiple files ==="
# Create sample files
echo '{"file":1}' > /tmp/file1.json
echo '{"file":2}' > /tmp/file2.json
jeb /tmp/file1.json /tmp/file2.json parse-json chain to-json
echo
