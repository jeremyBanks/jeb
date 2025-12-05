#!/usr/bin/env bash
set -euo pipefail

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Read extension info from package.json
PUBLISHER=$(jq -r '.publisher' "$SCRIPT_DIR/package.json")
NAME=$(jq -r '.name' "$SCRIPT_DIR/package.json")
VERSION=$(jq -r '.version' "$SCRIPT_DIR/package.json")

# VS Code extensions directory
EXTENSIONS_DIR="$HOME/.vscode-remote/extensions"

# Remove any existing installations
echo "Removing existing installations..."
rm -rf "${EXTENSIONS_DIR:?}/${NAME:?}" 2>/dev/null || true
rm -rf "${EXTENSIONS_DIR:?}/${PUBLISHER:?}.${NAME:?}"* 2>/dev/null || true

# Create symlink with proper naming convention
LINK_NAME="$PUBLISHER.$NAME-$VERSION"
echo "Installing $LINK_NAME -> $SCRIPT_DIR"
ln -sf "$SCRIPT_DIR" "$EXTENSIONS_DIR/$LINK_NAME"

echo "Done! Reload VS Code window to apply changes."
echo "  Ctrl+Shift+P -> 'Developer: Reload Window'"
