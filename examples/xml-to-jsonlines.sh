#!/usr/bin/env bash

# Example: Convert Stack Exchange XML Posts data to JSON Lines format
#
# This example demonstrates the xml-to-jsonlines transformation on a large
# real-world XML file (3D Printing Meta Stack Exchange posts). The output is
# limited to 128 lines for readability.
#
# The transformation converts XML hierarchies into a flat JSON Lines format,
# with each line representing a single XML node. Ancestor context is encoded
# using special attribute naming conventions.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
JEB="${SCRIPT_DIR}/../target/debug/jeb"

if [[ ! -f "$JEB" ]]; then
    echo "Error: jeb binary not found at $JEB" >&2
    echo "Please run 'cargo build' first" >&2
    exit 1
fi

if [[ ! -f "$SCRIPT_DIR/3dprinting-posts-sample.xml" ]]; then
    echo "Error: Sample XML file not found at $SCRIPT_DIR/3dprinting-posts-sample.xml" >&2
    exit 1
fi

# Convert the XML file to JSON Lines and limit output to 128 lines
"$JEB" "$SCRIPT_DIR/3dprinting-posts-sample.xml" xml-to-jsonlines 2>&1 | head -128

echo ""
echo "--- Output limited to 128 lines ---"
echo "This example demonstrates XML to JSON Lines conversion on a real Stack Exchange XML export."
echo "Each JSON object on a single line represents an XML node with its ancestors encoded in prefixed attributes."
