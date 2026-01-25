#!/bin/bash
# Test: If root has **/b/foo.txt, what patterns do we need in a/b/.gitignore 
# to achieve the same effect?

REPO_DIR="./repo"

echo "=== ROOT with **/b/foo.txt: What files are ignored under /a/b/? ==="
rm -rf "$REPO_DIR"
mkdir -p "$REPO_DIR"
cd "$REPO_DIR"
git init --quiet
git config user.email "test@test.com"
git config user.name "Test"

mkdir -p a/b/b
mkdir -p a/b/x/b
touch a/b/foo.txt        # /a/b/foo.txt
touch a/b/b/foo.txt      # /a/b/b/foo.txt
touch a/b/x/b/foo.txt    # /a/b/x/b/foo.txt
touch a/b/other.txt      # control file

echo "**/b/foo.txt" > .gitignore
echo "Root pattern: **/b/foo.txt"
echo "Files under a/b/ that are ignored:"
find a/b -name "*.txt" -type f | sort | while read f; do
    if git check-ignore -q "$f" 2>/dev/null; then
        echo "  ✓ IGNORED: $f"
    else
        echo "  ✗ not ignored: $f"
    fi
done

echo ""
echo "=== Now testing: a/b/.gitignore with '/foo.txt' alone ==="
rm -rf "$REPO_DIR"
mkdir -p "$REPO_DIR"
cd "$REPO_DIR"
git init --quiet
git config user.email "test@test.com"
git config user.name "Test"

mkdir -p a/b/b
mkdir -p a/b/x/b
touch a/b/foo.txt
touch a/b/b/foo.txt
touch a/b/x/b/foo.txt
touch a/b/other.txt

mkdir -p a/b
echo "/foo.txt" > a/b/.gitignore
echo "Pattern in a/b/.gitignore: /foo.txt"
echo "Files under a/b/ that are ignored:"
find a/b -name "*.txt" -type f | sort | while read f; do
    if git check-ignore -q "$f" 2>/dev/null; then
        echo "  ✓ IGNORED: $f"
    else
        echo "  ✗ not ignored: $f"
    fi
done

echo ""
echo "=== Now testing: a/b/.gitignore with '**/b/foo.txt' alone ==="
rm -rf "$REPO_DIR"
mkdir -p "$REPO_DIR"
cd "$REPO_DIR"
git init --quiet
git config user.email "test@test.com"
git config user.name "Test"

mkdir -p a/b/b
mkdir -p a/b/x/b
touch a/b/foo.txt
touch a/b/b/foo.txt
touch a/b/x/b/foo.txt
touch a/b/other.txt

mkdir -p a/b
echo "**/b/foo.txt" > a/b/.gitignore
echo "Pattern in a/b/.gitignore: **/b/foo.txt"
echo "Files under a/b/ that are ignored:"
find a/b -name "*.txt" -type f | sort | while read f; do
    if git check-ignore -q "$f" 2>/dev/null; then
        echo "  ✓ IGNORED: $f"
    else
        echo "  ✗ not ignored: $f"
    fi
done

echo ""
echo "=== Now testing: a/b/.gitignore with BOTH '/foo.txt' AND '**/b/foo.txt' ==="
rm -rf "$REPO_DIR"
mkdir -p "$REPO_DIR"
cd "$REPO_DIR"
git init --quiet
git config user.email "test@test.com"
git config user.name "Test"

mkdir -p a/b/b
mkdir -p a/b/x/b
touch a/b/foo.txt
touch a/b/b/foo.txt
touch a/b/x/b/foo.txt
touch a/b/other.txt

mkdir -p a/b
printf "/foo.txt\n**/b/foo.txt\n" > a/b/.gitignore
echo "Patterns in a/b/.gitignore: /foo.txt AND **/b/foo.txt"
echo "Files under a/b/ that are ignored:"
find a/b -name "*.txt" -type f | sort | while read f; do
    if git check-ignore -q "$f" 2>/dev/null; then
        echo "  ✓ IGNORED: $f"
    else
        echo "  ✗ not ignored: $f"
    fi
done
