#!/bin/bash
# Check that version in Cargo.toml follows project conventions
# Called by CI and can be run locally
# Supports both workspace and single-package layouts

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if this is a workspace
if grep -q '^\[workspace\]' Cargo.toml; then
    echo "Detected workspace structure"

    # Check if workspace has a shared version
    WORKSPACE_VERSION=$(grep '^\[workspace\.package\]' -A 10 Cargo.toml | grep '^version = ' | head -1 | cut -d'"' -f2 || echo "")

    if [[ -n "$WORKSPACE_VERSION" ]]; then
        echo "Workspace version: $WORKSPACE_VERSION"
        VERSION=$WORKSPACE_VERSION

        if [[ ! "$VERSION" =~ ^0\.0\.[0-9]+$ ]]; then
            echo -e "${RED}ERROR: Workspace version must be 0.0.x format (major=0, minor=0)${NC}"
            echo "Found: $VERSION"
            exit 1
        fi

        echo -e "${GREEN}✓ Workspace version $VERSION is valid (0.0.x format)${NC}"

        # For workspace with shared version, use root Cargo.toml for comparison
        CARGO_PATH="Cargo.toml"
    else
        # Check jeb crate
        if [ -f "crates/jeb/Cargo.toml" ]; then
            JEB_VERSION=$(grep '^version = ' crates/jeb/Cargo.toml | head -1 | cut -d'"' -f2)
            echo "jeb version: $JEB_VERSION"

            if [[ ! "$JEB_VERSION" =~ ^0\.0\.[0-9]+$ ]]; then
                echo -e "${RED}ERROR: jeb version must be 0.0.x format (major=0, minor=0)${NC}"
                echo "Found: $JEB_VERSION"
                exit 1
            fi

            echo -e "${GREEN}✓ jeb version $JEB_VERSION is valid (0.0.x format)${NC}"
        fi

        # Check json-encoded-binary crate
        if [ -f "crates/json-encoded-binary/Cargo.toml" ]; then
            JEB85_VERSION=$(grep '^version = ' crates/json-encoded-binary/Cargo.toml | head -1 | cut -d'"' -f2)
            echo "json-encoded-binary version: $JEB85_VERSION"

            if [[ ! "$JEB85_VERSION" =~ ^0\.0\.[0-9]+$ ]]; then
                echo -e "${RED}ERROR: json-encoded-binary version must be 0.0.x format (major=0, minor=0)${NC}"
                echo "Found: $JEB85_VERSION"
                exit 1
            fi

            echo -e "${GREEN}✓ json-encoded-binary version $JEB85_VERSION is valid (0.0.x format)${NC}"
        fi

        # For workspace without shared version, check if jeb version differs from base (primary crate)
        VERSION=$JEB_VERSION
        CARGO_PATH="crates/jeb/Cargo.toml"
    fi
else
    # Single package layout
    VERSION=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
    echo "Current version: $VERSION"

    # Check if version matches 0.0.x pattern
    if [[ ! "$VERSION" =~ ^0\.0\.[0-9]+$ ]]; then
        echo -e "${RED}ERROR: Version must be 0.0.x format (major=0, minor=0)${NC}"
        echo "Found: $VERSION"
        exit 1
    fi

    echo -e "${GREEN}✓ Version $VERSION is valid (0.0.x format)${NC}"
    CARGO_PATH="Cargo.toml"
fi

# If we're on a branch (not main/master/trunk), check if version differs from base
CURRENT_BRANCH=$(git branch --show-current)

if [[ "$CURRENT_BRANCH" != "main" && "$CURRENT_BRANCH" != "master" && "$CURRENT_BRANCH" != "trunk" ]]; then
    # Try to find and fetch base branch
    BASE_FOUND=false

    for BASE_BRANCH in main master trunk; do
        # Check if branch exists locally
        if git show-ref --verify --quiet "refs/heads/$BASE_BRANCH"; then
            BASE_FOUND=true
            echo "Found local $BASE_BRANCH branch"
        # Check if branch exists on remote
        elif git ls-remote --exit-code --heads origin "$BASE_BRANCH" &>/dev/null; then
            echo "Fetching $BASE_BRANCH from origin..."
            if git fetch origin "$BASE_BRANCH" &>/dev/null; then
                BASE_FOUND=true
                echo "Fetched origin/$BASE_BRANCH"
            fi
        fi

        if [ "$BASE_FOUND" = true ]; then
            # Try to get version from base branch - handle missing file gracefully
            BASE_VERSION=""

            # First check if our Cargo.toml exists in the base branch
            if git cat-file -e "origin/$BASE_BRANCH:$CARGO_PATH" 2>/dev/null || \
               git cat-file -e "$BASE_BRANCH:$CARGO_PATH" 2>/dev/null; then

                # Get the version - check for workspace.package or direct version
                BASE_CONTENT=$(git show "origin/$BASE_BRANCH:$CARGO_PATH" 2>/dev/null || \
                              git show "$BASE_BRANCH:$CARGO_PATH" 2>/dev/null || \
                              echo "")

                # Try workspace.package version first
                BASE_VERSION=$(echo "$BASE_CONTENT" | grep '^\[workspace\.package\]' -A 10 | grep '^version = ' | head -1 | cut -d'"' -f2 || echo "")

                # If not found, try direct version
                if [[ -z "$BASE_VERSION" ]]; then
                    BASE_VERSION=$(echo "$BASE_CONTENT" | grep '^version = ' | head -1 | cut -d'"' -f2 || echo "")
                fi
            else
                # Try the other layout (workspace vs single-package)
                if [[ "$CARGO_PATH" == "Cargo.toml" ]]; then
                    ALT_PATH="crates/jeb/Cargo.toml"
                else
                    ALT_PATH="Cargo.toml"
                fi

                if git cat-file -e "origin/$BASE_BRANCH:$ALT_PATH" 2>/dev/null || \
                   git cat-file -e "$BASE_BRANCH:$ALT_PATH" 2>/dev/null; then

                    BASE_CONTENT=$(git show "origin/$BASE_BRANCH:$ALT_PATH" 2>/dev/null || \
                                  git show "$BASE_BRANCH:$ALT_PATH" 2>/dev/null || \
                                  echo "")

                    # Try workspace.package version first
                    BASE_VERSION=$(echo "$BASE_CONTENT" | grep '^\[workspace\.package\]' -A 10 | grep '^version = ' | head -1 | cut -d'"' -f2 || echo "")

                    # If not found, try direct version
                    if [[ -z "$BASE_VERSION" ]]; then
                        BASE_VERSION=$(echo "$BASE_CONTENT" | grep '^version = ' | head -1 | cut -d'"' -f2 || echo "")
                    fi

                    if [[ -n "$BASE_VERSION" ]]; then
                        echo -e "${YELLOW}⚠ Project structure changed between branches${NC}"
                        echo "Base used: $ALT_PATH, current uses: $CARGO_PATH"
                    fi
                else
                    echo -e "${YELLOW}⚠ Cargo.toml does not exist in $BASE_BRANCH branch${NC}"
                    echo "This is okay if the project structure has changed"
                    echo -e "${GREEN}✓ Version check complete (no comparison possible)${NC}"
                    exit 0
                fi
            fi

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
            else
                echo -e "${YELLOW}⚠ Could not extract version from $BASE_BRANCH:$CARGO_PATH${NC}"
            fi
        fi
    done

    if [ "$BASE_FOUND" = false ]; then
        echo -e "${YELLOW}⚠ Could not find base branch to compare version${NC}"
        echo "This is okay for new repositories or if working offline"
    fi
fi

echo -e "${GREEN}✓ Version check complete${NC}"
