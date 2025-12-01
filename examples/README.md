# JEB Examples

This directory contains examples demonstrating the capabilities of the `jeb` binary.

## XML to JSON Conversion

The `parse-xml` command converts XML to JSON using a lossless transformation scheme that preserves all structure, attributes, and metadata.

### Sample XML Files

- `xml/simple.xml` - Basic XML with repeated elements
- `xml/book.xml` - XML with attributes and nested structure
- `xml/nested.xml` - Deeply nested XML demonstrating parent attribute propagation
- `xml/mixed-content.xml` - HTML-style document with DOCTYPE, CDATA, and comments

### Running Examples

#### Using the shell script:
```bash
./examples/xml-to-json.sh
```

#### Manual usage:
```bash
# Convert a file
cargo run -- examples/xml/book.xml parse-xml stdout

# Use stdin
echo '<root><item>test</item></root>' | cargo run -- stdin parse-xml stdout

# Chain with other commands
cargo run -- examples/xml/simple.xml parse-xml split-lines filter stdout
```

### Transformation Features

The XML to JSON conversion uses a special naming scheme for lossless round-tripping:

**Tag and Parent Information:**
- `""` - The node's tag name
- `"-"` - Parent tag name
- `"--"` - Grandparent tag name (continues with more hyphens for ancestors)
- `"-attribute"` - Parent's attribute values
- `"--attribute"` - Grandparent's attribute values

**Virtual Attributes (always present):**
- `@text` - Text content of the node (empty string for self-closing tags, null for container-only tags)
- `@tail` - Text immediately following the node
- `@index` - Sibling index for disambiguation

**Metadata Preservation:**
- CDATA sections: `""` = `![CDATA[`, `@text` = content
- Comments: `""` = `!--`, `@text` = comment text
- Processing instructions: `""` = `?xml`, `@text` = PI content
- DOCTYPE: `""` = `!DOCTYPE`, `@text` = DOCTYPE content

### Example Output

Input (`examples/xml/book.xml`):
```xml
<?xml version="1.0" encoding="UTF-8"?>
<book id="123">
  <title>The Art of Programming</title>
  <author name="Jane Doe">
    <email>jane@example.com</email>
  </author>
</book>
```

Output:
```json
{
  "": "book",
  "id": "123",
  "@text": "\n  \n  \n",
  "@tail": null,
  "@index": 0,
  "children": [
    {
      "": "title",
      "-": "book",
      "--": null,
      "-id": "123",
      "@text": "The Art of Programming",
      "@tail": null,
      "@index": 0
    },
    {
      "": "author",
      "-": "book",
      "--": null,
      "-id": "123",
      "name": "Jane Doe",
      "@text": "\n    \n  ",
      "@tail": null,
      "@index": 0,
      "children": [
        {
          "": "email",
          "-": "author",
          "--": "book",
          "---": null,
          "-name": "Jane Doe",
          "--id": "123",
          "@text": "jane@example.com",
          "@tail": null,
          "@index": 0
        }
      ]
    }
  ]
}
```

Notice how:
- Each node knows its parent's tag name and attributes (via `"-"` prefix)
- Grandparent information is available (via `"--"` prefix)
- All text content, including whitespace, is preserved
- Attributes are accessible both on the node itself and via parent references
