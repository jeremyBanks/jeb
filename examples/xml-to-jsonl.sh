#!/bin/bash
# Example: Convert StackExchange-style Posts.xml to JSON Lines
# This demonstrates the xml-to-jsonl command with a sample dataset
# mimicking the format from archive.org StackExchange data dumps.
#
# The output is limited to 128 lines (or all lines if fewer).

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
XML_FILE="$SCRIPT_DIR/stackexchange-posts.xml"

# Check if xml file exists
if [ ! -f "$XML_FILE" ]; then
    echo "Error: $XML_FILE not found"
    exit 1
fi

# Convert XML to JSON Lines, limiting output to 128 lines
cargo run -q -- "$XML_FILE" xml-to-jsonl stdout | head -128
