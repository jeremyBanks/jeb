#!/bin/bash
# Direct comparison: a/b/foo.txt vs /a/b/foo.txt

REPO_DIR="./repo"

echo "=== Testing if 'a/b/foo.txt' and '/a/b/foo.txt' are truly equivalent ==="

# Test 1: Basic comparison
echo ""
echo "--- Test 1: Root gitignore, basic structure ---"
rm -rf "$REPO_DIR" && mkdir -p "$REPO_DIR" && cd "$REPO_DIR"
git init --quiet && git config user.email "test@test.com" && git config user.name "Test"

mkdir -p a/b x/a/b
touch a/b/foo.txt x/a/b/foo.txt foo.txt

echo "a/b/foo.txt" > .gitignore
echo "Pattern: a/b/foo.txt (no leading slash)"
git check-ignore -v a/b/foo.txt x/a/b/foo.txt foo.txt 2>/dev/null || echo "(no matches)"

cd ..
rm -rf "$REPO_DIR" && mkdir -p "$REPO_DIR" && cd "$REPO_DIR"
git init --quiet && git config user.email "test@test.com" && git config user.name "Test"

mkdir -p a/b x/a/b
touch a/b/foo.txt x/a/b/foo.txt foo.txt

echo "/a/b/foo.txt" > .gitignore
echo ""
echo "Pattern: /a/b/foo.txt (with leading slash)"
git check-ignore -v a/b/foo.txt x/a/b/foo.txt foo.txt 2>/dev/null || echo "(no matches)"

# Test 2: What about in a subdirectory gitignore?
echo ""
echo "--- Test 2: Subdirectory gitignore ---"
cd ..
rm -rf "$REPO_DIR" && mkdir -p "$REPO_DIR" && cd "$REPO_DIR"
git init --quiet && git config user.email "test@test.com" && git config user.name "Test"

mkdir -p sub/a/b sub/x/a/b
touch sub/a/b/foo.txt sub/x/a/b/foo.txt sub/foo.txt

echo "a/b/foo.txt" > sub/.gitignore
echo "Pattern in sub/.gitignore: a/b/foo.txt"
git check-ignore -v sub/a/b/foo.txt sub/x/a/b/foo.txt sub/foo.txt 2>/dev/null || echo "(no matches)"

cd ..
rm -rf "$REPO_DIR" && mkdir -p "$REPO_DIR" && cd "$REPO_DIR"
git init --quiet && git config user.email "test@test.com" && git config user.name "Test"

mkdir -p sub/a/b sub/x/a/b
touch sub/a/b/foo.txt sub/x/a/b/foo.txt sub/foo.txt

echo "/a/b/foo.txt" > sub/.gitignore
echo ""
echo "Pattern in sub/.gitignore: /a/b/foo.txt"
git check-ignore -v sub/a/b/foo.txt sub/x/a/b/foo.txt sub/foo.txt 2>/dev/null || echo "(no matches)"

# Test 3: What if we have both root AND nested matching paths?
echo ""
echo "--- Test 3: Complex nesting ---"
cd ..
rm -rf "$REPO_DIR" && mkdir -p "$REPO_DIR" && cd "$REPO_DIR"
git init --quiet && git config user.email "test@test.com" && git config user.name "Test"

mkdir -p a/b a/b/a/b x/a/b
touch a/b/foo.txt a/b/a/b/foo.txt x/a/b/foo.txt

echo "a/b/foo.txt" > .gitignore
echo "Pattern: a/b/foo.txt"
echo "All foo.txt files:"
find . -name foo.txt | sort | while read f; do
    result=$(git check-ignore -v "$f" 2>/dev/null || echo "NOT IGNORED: $f")
    echo "  $result"
done

# Test 4: Does the implicit anchoring apply to the FIRST component only?
echo ""
echo "--- Test 4: Pattern 'b/foo.txt' - where does 'b' anchor? ---"
cd ..
rm -rf "$REPO_DIR" && mkdir -p "$REPO_DIR" && cd "$REPO_DIR"
git init --quiet && git config user.email "test@test.com" && git config user.name "Test"

mkdir -p b a/b x/y/b
touch b/foo.txt a/b/foo.txt x/y/b/foo.txt

echo "b/foo.txt" > .gitignore
echo "Pattern: b/foo.txt"
find . -name foo.txt | sort | while read f; do
    result=$(git check-ignore -v "$f" 2>/dev/null || echo "NOT IGNORED: $f")
    echo "  $result"
done
