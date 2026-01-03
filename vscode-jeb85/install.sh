#!/usr/bin/env bash
set -euo pipefail

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PACKAGE_JSON="$SCRIPT_DIR/package.json"

# Read extension info from package.json
PUBLISHER=$(jq -r '.publisher' "$PACKAGE_JSON")
NAME=$(jq -r '.name' "$PACKAGE_JSON")
OLD_VERSION=$(jq -r '.version' "$PACKAGE_JSON")

# Bump the minor version (x.Y.z -> x.Y+1.z)
IFS='.' read -r MAJOR MINOR PATCH <<< "$OLD_VERSION"
NEW_VERSION="$MAJOR.$MINOR.$((PATCH + 1))"

# Update package.json with new version
jq --arg v "$NEW_VERSION" '.version = $v' "$PACKAGE_JSON" > "$PACKAGE_JSON.tmp"
mv "$PACKAGE_JSON.tmp" "$PACKAGE_JSON"
echo "Bumped version: $OLD_VERSION -> $NEW_VERSION"

# VS Code extensions directory
EXTENSIONS_DIR="$HOME/.vscode-remote/extensions"

# Remove any existing installations
echo "Removing existing installations..."
rm -rf "${EXTENSIONS_DIR:?}/${NAME:?}" 2>/dev/null || true
rm -rf "${EXTENSIONS_DIR:?}/${PUBLISHER:?}.${NAME:?}"* 2>/dev/null || true

# Create symlink with proper naming convention
LINK_NAME="$PUBLISHER.$NAME-$NEW_VERSION"
echo "Installing $LINK_NAME -> $SCRIPT_DIR"
ln -sf "$SCRIPT_DIR" "$EXTENSIONS_DIR/$LINK_NAME"

echo "Done! Reload VS Code window to apply changes."
echo "  Ctrl+Shift+P -> 'Developer: Reload Window'"
