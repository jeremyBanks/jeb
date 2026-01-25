#!/bin/bash
# Test patterns in subdirectory gitignore

REPO_DIR="./repo"
rm -rf "$REPO_DIR"
mkdir -p "$REPO_DIR"
cd "$REPO_DIR"
git init --quiet
git config user.email "test@test.com"
git config user.name "Test"

# Create structure
mkdir -p a/b/c
touch foo.txt
touch a/foo.txt
touch a/b/foo.txt
touch a/b/c/foo.txt

echo ""
echo "=== Test: Pattern 'foo.txt' in a/b/.gitignore ==="
echo "foo.txt" > a/b/.gitignore
find . -name "foo.txt" -type f | sort | while read f; do
    result=$(git check-ignore -v "$f" 2>/dev/null || echo "NOT IGNORED: $f")
    echo "  $result"
done

echo ""
echo "=== Test: Pattern '/foo.txt' in a/b/.gitignore ==="
echo "/foo.txt" > a/b/.gitignore
find . -name "foo.txt" -type f | sort | while read f; do
    result=$(git check-ignore -v "$f" 2>/dev/null || echo "NOT IGNORED: $f")
    echo "  $result"
done

echo ""
echo "=== Test: Pattern 'c/foo.txt' in a/b/.gitignore ==="
echo "c/foo.txt" > a/b/.gitignore
find . -name "foo.txt" -type f | sort | while read f; do
    result=$(git check-ignore -v "$f" 2>/dev/null || echo "NOT IGNORED: $f")
    echo "  $result"
done
