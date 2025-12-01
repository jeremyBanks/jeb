#!/bin/bash
# Examples of using jeb to convert XML to JSON

echo "=== Simple XML Example ==="
cargo run --quiet -- examples/xml/simple.xml parse-xml

echo ""
echo "=== Book Example with Attributes ==="
cargo run --quiet -- examples/xml/book.xml parse-xml

echo ""
echo "=== Nested Structure Example ==="
cargo run --quiet -- examples/xml/nested.xml parse-xml

echo ""
echo "=== Mixed Content with DOCTYPE and CDATA ==="
cargo run --quiet -- examples/xml/mixed-content.xml parse-xml

echo ""
echo "=== Using stdin ==="
echo '<greeting name="World">Hello!</greeting>' | cargo run --quiet -- stdin parse-xml

echo ""
echo "=== Converting to compact JSON (pipe through jq) ==="
if command -v jq &> /dev/null; then
    cargo run --quiet -- examples/xml/simple.xml parse-xml | jq -c
else
    echo "(jq not installed, skipping compact JSON example)"
fi
