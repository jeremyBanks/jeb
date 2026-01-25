#!/bin/bash
# Comprehensive gitignore behavior tests

set -e

TEST_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$TEST_DIR/repo"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

setup_repo() {
    rm -rf "$REPO_DIR"
    mkdir -p "$REPO_DIR"
    cd "$REPO_DIR"
    git init --quiet
    git config user.email "test@test.com"
    git config user.name "Test"
}

create_test_files() {
    # Create comprehensive directory structure
    mkdir -p a/b/a/b
    mkdir -p b
    mkdir -p x/a/b
    mkdir -p x/y/a/b

    # Create files at various levels
    touch foo.txt
    touch a/foo.txt
    touch a/b/foo.txt
    touch a/b/a/b/foo.txt
    touch b/foo.txt
    touch x/a/b/foo.txt
    touch x/y/a/b/foo.txt
}

test_pattern() {
    local pattern="$1"
    local description="$2"

    setup_repo
    create_test_files

    echo "$pattern" > .gitignore

    echo ""
    echo -e "${YELLOW}=== Testing: $pattern ===${NC}"
    echo "Description: $description"
    echo "Files ignored:"

    # Use git check-ignore to see what matches
    local ignored_files=$(find . -name "foo.txt" -type f | sort | while read f; do
        if git check-ignore -q "$f" 2>/dev/null; then
            echo "  $f"
        fi
    done)

    if [ -z "$ignored_files" ]; then
        echo "  (none)"
    else
        echo "$ignored_files"
    fi

    # Also show verbose output for each file
    echo "Detailed check-ignore output:"
    find . -name "foo.txt" -type f | sort | while read f; do
        result=$(git check-ignore -v "$f" 2>/dev/null || echo "NOT IGNORED: $f")
        echo "  $result"
    done
}

echo "========================================"
echo "GITIGNORE BEHAVIOR TEST SUITE"
echo "========================================"

# Group 1: Simple patterns (no slash)
test_pattern "foo.txt" "Simple filename - should match at any level"

# Group 2: Leading slash (anchored)
test_pattern "/foo.txt" "Leading slash - should only match root level"
test_pattern "/a/b/foo.txt" "Leading slash with path - should only match that specific path"

# Group 3: THE KEY QUESTION - slash in middle
test_pattern "a/b/foo.txt" "Slash in middle - does this match ONLY /a/b/foo.txt or ALSO /x/a/b/foo.txt?"
test_pattern "b/foo.txt" "Slash in middle - does this match /b/foo.txt only, or also /a/b/foo.txt?"

# Group 4: Double-star patterns
test_pattern "**/foo.txt" "Double-star prefix - should match at any level"
test_pattern "**/a/b/foo.txt" "Double-star with path - should match a/b/foo.txt at any level"
test_pattern "**/b/foo.txt" "Double-star with partial path - should match b/foo.txt at any level"

# Group 5: Trailing slash (directories)
test_pattern "a/" "Directory pattern - what does this match?"
test_pattern "b/" "Directory pattern b/ - matches /b/ but also /a/b/?"
test_pattern "/a/" "Anchored directory - should only match root /a/"

# Group 6: Single star in path
test_pattern "*/foo.txt" "Star at start - matches one level deep?"
test_pattern "a/*/foo.txt" "Star in middle - matches a/[anything]/foo.txt?"
test_pattern "**/a/*/foo.txt" "Double-star then single star"

# Group 7: Compare behaviors
echo ""
echo -e "${YELLOW}=== COMPARISON SUMMARY ===${NC}"
echo "Testing critical difference between a/b/foo.txt and **/a/b/foo.txt"

setup_repo
create_test_files
echo "a/b/foo.txt" > .gitignore
echo "Pattern: a/b/foo.txt"
echo "Matches:"
find . -name "foo.txt" -type f | sort | while read f; do
    if git check-ignore -q "$f" 2>/dev/null; then
        echo "  ✓ $f"
    else
        echo "  ✗ $f"
    fi
done

setup_repo
create_test_files
echo "**/a/b/foo.txt" > .gitignore
echo ""
echo "Pattern: **/a/b/foo.txt"
echo "Matches:"
find . -name "foo.txt" -type f | sort | while read f; do
    if git check-ignore -q "$f" 2>/dev/null; then
        echo "  ✓ $f"
    else
        echo "  ✗ $f"
    fi
done

echo ""
echo "========================================"
echo "TESTS COMPLETE"
echo "========================================"
