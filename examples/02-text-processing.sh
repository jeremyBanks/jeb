#!/bin/bash
# Example 2: Text processing capabilities

echo "=== Example 1: Split text by lines ==="
echo -e 'line 1\nline 2\nline 3' | jeb split-lines join-space
echo

echo "=== Example 2: Collapse whitespace ==="
echo 'hello    world    from    jeb' | jeb collapse
echo

echo "=== Example 3: Keep first 3 lines ==="
echo -e 'a\nb\nc\nd\ne' | jeb split-lines first-3 join-lines
echo

echo "=== Example 4: Keep last 2 lines ==="
echo -e 'a\nb\nc\nd\ne' | jeb split-lines last-2 join-lines
echo

echo "=== Example 5: Join lines with spaces ==="
echo -e 'hello\nworld' | jeb split-lines join-space
echo
