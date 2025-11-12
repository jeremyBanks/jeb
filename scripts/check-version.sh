#!/bin/bash
# Check that version in Cargo.toml follows project conventions
# This replicates the version-check.yml workflow

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get current version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
echo "Current version: $VERSION"

# Check if version matches 0.0.x pattern
if [[ ! "$VERSION" =~ ^0\.0\.[0-9]+$ ]]; then
    echo -e "${RED}ERROR: Version must be 0.0.x format (major=0, minor=0)${NC}"
    echo "Found: $VERSION"
    exit 1
fi

echo -e "${GREEN}✓ Version $VERSION is valid (0.0.x format)${NC}"

# If we're on a branch (not main/master/trunk), check if version differs from main
CURRENT_BRANCH=$(git branch --show-current)

if [[ "$CURRENT_BRANCH" != "main" && "$CURRENT_BRANCH" != "master" && "$CURRENT_BRANCH" != "trunk" ]]; then
    # Try to get version from main branch (or master/trunk as fallback)
    for BASE_BRANCH in main master trunk; do
        if git show-ref --verify --quiet "refs/heads/$BASE_BRANCH" || \
           git show-ref --verify --quiet "refs/remotes/origin/$BASE_BRANCH"; then

            # Get version from base branch
            BASE_VERSION=$(git show "origin/$BASE_BRANCH:Cargo.toml" 2>/dev/null | grep '^version = ' | head -1 | cut -d'"' -f2 || echo "")

            if [[ -n "$BASE_VERSION" ]]; then
                echo "Base branch ($BASE_BRANCH) version: $BASE_VERSION"

                if [[ "$VERSION" == "$BASE_VERSION" ]]; then
                    echo -e "${RED}ERROR: Version must be bumped for this branch${NC}"
                    echo "Current: $VERSION"
                    echo "Base: $BASE_VERSION"
                    echo ""
                    echo "Please increment the patch version (e.g., 0.0.3 -> 0.0.4)"
                    exit 1
                fi

                echo -e "${GREEN}✓ Version has been updated from $BASE_VERSION to $VERSION${NC}"
                exit 0
            fi
        fi
    done

    echo -e "${YELLOW}⚠ Could not find base branch to compare version${NC}"
    echo "This is okay for new repositories or if you're working offline"
fi

echo -e "${GREEN}✓ Version check complete${NC}"
