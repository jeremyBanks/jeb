#!/bin/bash
# Test wildcards with anchoring

REPO_DIR="./repo"

echo "=== Testing wildcards with implicit vs explicit anchoring ==="

# Test: */b/foo.txt vs /*/b/foo.txt
echo ""
echo "--- Pattern: */b/foo.txt (implicit anchor) ---"
rm -rf "$REPO_DIR" && mkdir -p "$REPO_DIR" && cd "$REPO_DIR"
git init --quiet && git config user.email "test@test.com" && git config user.name "Test"

mkdir -p a/b x/a/b y/z/b
touch a/b/foo.txt x/a/b/foo.txt y/z/b/foo.txt b/foo.txt

echo "*/b/foo.txt" > .gitignore
find . -name foo.txt | sort | while read f; do
    result=$(git check-ignore -v "$f" 2>/dev/null || echo "NOT: $f")
    echo "  $result"
done

echo ""
echo "--- Pattern: /*/b/foo.txt (explicit anchor) ---"
cd ..
rm -rf "$REPO_DIR" && mkdir -p "$REPO_DIR" && cd "$REPO_DIR"
git init --quiet && git config user.email "test@test.com" && git config user.name "Test"

mkdir -p a/b x/a/b y/z/b
touch a/b/foo.txt x/a/b/foo.txt y/z/b/foo.txt b/foo.txt

echo "/*/b/foo.txt" > .gitignore
find . -name foo.txt | sort | while read f; do
    result=$(git check-ignore -v "$f" 2>/dev/null || echo "NOT: $f")
    echo "  $result"
done

# Test: a/*/foo.txt vs /a/*/foo.txt  
echo ""
echo "--- Pattern: a/*/foo.txt ---"
cd ..
rm -rf "$REPO_DIR" && mkdir -p "$REPO_DIR" && cd "$REPO_DIR"
git init --quiet && git config user.email "test@test.com" && git config user.name "Test"

mkdir -p a/b a/c x/a/b
touch a/b/foo.txt a/c/foo.txt x/a/b/foo.txt a/foo.txt

echo "a/*/foo.txt" > .gitignore
find . -name foo.txt | sort | while read f; do
    result=$(git check-ignore -v "$f" 2>/dev/null || echo "NOT: $f")
    echo "  $result"
done

echo ""
echo "--- Pattern: /a/*/foo.txt ---"
cd ..
rm -rf "$REPO_DIR" && mkdir -p "$REPO_DIR" && cd "$REPO_DIR"
git init --quiet && git config user.email "test@test.com" && git config user.name "Test"

mkdir -p a/b a/c x/a/b
touch a/b/foo.txt a/c/foo.txt x/a/b/foo.txt a/foo.txt

echo "/a/*/foo.txt" > .gitignore
find . -name foo.txt | sort | while read f; do
    result=$(git check-ignore -v "$f" 2>/dev/null || echo "NOT: $f")
    echo "  $result"
done
