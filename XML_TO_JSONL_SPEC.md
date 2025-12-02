# XML to JSON Lines Conversion - Technical Specification

## Overview

This specification defines a lossless, streaming transformation from XML documents to JSON Lines format. The design flattens hierarchical XML structures into a stream of individual JSON objects, where each object represents a single XML node with complete ancestor context encoded via special attribute naming conventions.

## Design Goals

1. **Lossless Round-Tripping**: Preserve all information necessary to reconstruct the original XML document exactly, including:
   - Element hierarchy and ordering
   - All attributes and their values
   - Text content and whitespace
   - Metadata nodes (comments, processing instructions, DOCTYPE, CDATA, etc.)
   - Self-closing vs. empty tag distinction

2. **Streaming Compatibility**: Enable processing of arbitrarily large XML documents without requiring the entire document in memory. Each XML node can be converted to a JSON object and output immediately in document order.

3. **Flat Output Structure**: Avoid nested JSON objects. Each XML element produces exactly one JSON object containing all necessary context for interpretation.

4. **Unambiguous Encoding**: Use a naming scheme that cannot collide with valid XML element/attribute names, ensuring the special fields are always distinguishable.

## Input Format

**Supported**: Well-formed XML 1.0/1.1 documents, including HTML documents that can be parsed as XML.

**Character Encoding**: Implementation should accept both string and byte slice inputs, with appropriate encoding detection/handling for byte inputs.

## Output Format

**Format**: JSON Lines (one JSON object per line, newline-separated)

**Order**: Objects appear in document order (depth-first traversal)

**Encoding**: Each line is a valid, compact (non-pretty-printed) JSON object followed by a newline (`\n`)

## Transformation Rules

### Core Attribute Naming Scheme

Each JSON object uses a special naming convention to encode the XML node and its ancestral context:

| Attribute Pattern | Meaning | Example |
|------------------|---------|---------|
| `""` (empty string) | The node's tag name | `"": "div"` |
| `"-"` | Parent element's tag name | `"-": "body"` |
| `"--"` | Grandparent element's tag name | `"--": "html"` |
| `"---"`, `"----"`, etc. | Great-grandparent, great-great-grandparent, etc. | `"---": "root"` |
| `"-attributeName"` | Parent element's attribute value | `"-id": "main"` |
| `"--attributeName"` | Grandparent element's attribute value | `"--class": "page"` |
| `"---attributeName"`, etc. | Higher ancestor attribute values | `"---lang": "en"` |

**Key Properties**:
- Valid XML names cannot be empty or start with `-` or `@`, preventing collisions
- Unlimited ancestor depth supported (no hard limit on number of `-` prefixes)
- All attributes from all ancestors are included with appropriate prefix depth
- Root elements have no parent attributes (no `-`, `--`, etc. fields)

### Virtual Attributes

Three special attributes are added to every node to capture structural information:

| Attribute | Purpose | Presence Rule |
|-----------|---------|---------------|
| `@text` | Text content within the element (before any child elements) | Always present for non-self-closing tags (may be empty string `""`). Omitted entirely for self-closing tags. |
| `@tail` | Text content after the closing tag, before the next sibling | Always present (may be empty string `""`) |
| `@index` | Zero-based sibling index among nodes at the same level | Always present on all nodes (including root-level nodes) |

**Rationale**:
- `@text` presence/absence distinguishes `<tag/>` (self-closing, omitted) from `<tag></tag>` (empty but not self-closing, `@text: ""`)
- `@tail` captures inter-element whitespace and text that follows an element
- `@index` provides canonical ordering for reconstruction and disambiguates identical siblings

### Special Node Types

XML metadata and special constructs are represented as pseudo-elements with synthetic tag names:

| XML Construct | Tag Name (`""` field) | `@text` Content |
|---------------|----------------------|-----------------|
| `<!-- comment -->` | `"!--"` | `" comment "` (everything between `<!--` and `-->`) |
| `<?xml version="1.0"?>` | `"?xml"` | `" version=\"1.0\""` (everything between `<?xml` and `?>`) |
| `<?xml-stylesheet ...?>` | `"?xml-stylesheet"` | Everything between opening and closing delimiters |
| `<!DOCTYPE html>` | `"!DOCTYPE"` | `" html"` (everything between `<!DOCTYPE` and `>`) |
| `<!ENTITY name "value">` | `"!ENTITY"` | ` name "value"` |
| `<![CDATA[content]]>` | `"![CDATA["` | `"content"` (everything before `]]>`) |

**Behavior**:
- Special nodes get `@text`, `@tail`, and `@index` like regular elements
- `@text` contains the complete literal content between the pseudo-tag-name and the closing delimiter, preserving all whitespace
- These nodes have no child elements (they're leaf nodes)
- They participate in the ancestor chain (child elements get these as parent context if applicable)

### Namespace Handling

**Approach**: Literal preservation

- Namespaced element names like `<foo:bar>` → `"": "foo:bar"`
- Namespace declarations like `xmlns:foo="http://..."` → `"xmlns:foo": "http://..."`
- No special processing or normalization of namespace prefixes

### Entity and Character Reference Handling

**Built-in XML Entities** (always decoded):
- `&amp;` → `&`
- `&lt;` → `<`
- `&gt;` → `>`
- `&quot;` → `"`
- `&apos;` → `'`

**Numeric Character References** (always decoded):
- Decimal: `&#65;` → `A`
- Hexadecimal: `&#x41;` → `A`

**Named Entities from Standards** (decoded):
- All entities defined in XML 1.0 specification
- All entities defined in XML 1.1 specification
- All entities defined in HTML 4 specification
- All entities defined in HTML 5 specification

**Unsupported Entities** (replacement + warning):
- Custom/user-defined entities not in above standards
- External entity references
- Entities requiring external DTD resolution

**Error Handling**: When an unsupported entity is encountered:
1. Replace with Unicode replacement character (U+FFFD `�`)
2. Log a warning using the tracing warning macro
3. Continue processing

**External Entities**: Not resolved. External entity references are treated as unsupported entities (replacement character + warning).

## Streaming and Ordering

**Document Order**: Nodes are output in the order they appear in the XML document (depth-first, pre-order traversal):
1. Parent element
2. Parent's first child (and all its descendants)
3. Parent's second child (and all its descendants)
4. And so on...

**Example**:
```xml
<root>
  <a>
    <b/>
  </a>
  <c/>
</root>
```

Output order: `root`, `a`, `b`, `c`

## Complete Examples

### Example 1: Simple Nested Elements

**Input XML**:
```xml
<html lang="en">
  <body id="main">
    <div class="content">Hello</div>
  </body>
</html>
```

**Output JSON Lines**:
```json
{"": "html", "lang": "en", "@text": "\n  ", "@tail": "", "@index": 0}
{"": "body", "-": "html", "-lang": "en", "id": "main", "@text": "\n    ", "@tail": "\n", "@index": 0}
{"": "div", "-": "body", "--": "html", "-id": "main", "--lang": "en", "class": "content", "@text": "Hello", "@tail": "\n  ", "@index": 0}
```

**Explanation**:
- `html` element: Root node (no parent attributes), has `lang` attribute, text content is newline+spaces, no tail, index 0
- `body` element: Has parent context (`"-": "html"`, `"-lang": "en"`), has `id` attribute, text is whitespace, tail is newline
- `div` element: Has parent and grandparent context, all ancestor attributes included

### Example 2: Self-Closing vs Empty Tags

**Input XML**:
```xml
<root>
  <self-closing/>
  <empty></empty>
</root>
```

**Output JSON Lines**:
```json
{"": "root", "@text": "\n  ", "@tail": "", "@index": 0}
{"": "self-closing", "-": "root", "@tail": "\n  ", "@index": 0}
{"": "empty", "-": "root", "@text": "", "@tail": "\n", "@index": 1}
```

**Explanation**:
- `self-closing`: No `@text` field (distinguishes from empty tag)
- `empty`: Has `@text: ""` (empty string, but field is present)

### Example 3: Mixed Content (Text and Elements)

**Input XML**:
```xml
<p>Text before <em>emphasis</em> text after</p>
```

**Output JSON Lines**:
```json
{"": "p", "@text": "Text before ", "@tail": "", "@index": 0}
{"": "em", "-": "p", "@text": "emphasis", "@tail": " text after", "@index": 0}
```

**Explanation**:
- `p` element: `@text` contains text before first child element
- `em` element: `@tail` contains text after the closing `</em>` tag

### Example 4: Special Nodes and Metadata

**Input XML**:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<!-- This is a comment -->
<root><![CDATA[Some <data>]]></root>
```

**Output JSON Lines**:
```json
{"": "?xml", "@text": " version=\"1.0\" encoding=\"UTF-8\"", "@tail": "\n", "@index": 0}
{"": "!DOCTYPE", "@text": " html", "@tail": "\n", "@index": 1}
{"": "!--", "@text": " This is a comment ", "@tail": "\n", "@index": 2}
{"": "root", "@text": "", "@tail": "", "@index": 3}
{"": "![CDATA[", "-": "root", "@text": "Some <data>", "@tail": "", "@index": 0}
```

**Explanation**:
- XML declaration, DOCTYPE, and comment are root-level nodes with sequential indices
- CDATA section is a child of `root`, preserves literal content including `<` and `>`

### Example 5: Multiple Root-Level Nodes

**Input XML**:
```xml
<!-- Comment 1 -->
<!-- Comment 2 -->
<root/>
<!-- Comment 3 -->
```

**Output JSON Lines**:
```json
{"": "!--", "@text": " Comment 1 ", "@tail": "\n", "@index": 0}
{"": "!--", "@text": " Comment 2 ", "@tail": "\n", "@index": 1}
{"": "root", "@tail": "\n", "@index": 2}
{"": "!--", "@text": " Comment 3 ", "@tail": "", "@index": 3}
```

**Explanation**:
- All root-level nodes (including comments before/after root element) get sequential `@index` values
- No parent attributes for any of these nodes

### Example 6: Entity References

**Input XML**:
```xml
<p>Standard: &lt;&gt;&amp; Numeric: &#65; HTML: &nbsp; Custom: &custom;</p>
```

**Output JSON Lines**:
```json
{"": "p", "@text": "Standard: <>& Numeric: A HTML:   Custom: �", "@tail": "", "@index": 0}
```

**Explanation**:
- `&lt;`, `&gt;`, `&amp;` decoded to `<`, `>`, `&`
- `&#65;` decoded to `A`
- `&nbsp;` (HTML entity) decoded to non-breaking space
- `&custom;` (unsupported) replaced with `�` and warning logged

### Example 7: Deep Nesting with Attributes

**Input XML**:
```xml
<a id="1"><b id="2"><c id="3"><d id="4">text</d></c></b></a>
```

**Output JSON Lines** (formatted for readability, actual output is compact):
```json
{"": "a", "id": "1", "@text": "", "@tail": "", "@index": 0}
{"": "b", "-": "a", "-id": "1", "id": "2", "@text": "", "@tail": "", "@index": 0}
{"": "c", "-": "b", "--": "a", "-id": "2", "--id": "1", "id": "3", "@text": "", "@tail": "", "@index": 0}
{"": "d", "-": "c", "--": "b", "---": "a", "-id": "3", "--id": "2", "---id": "1", "id": "4", "@text": "text", "@tail": "", "@index": 0}
```

**Explanation**:
- Each level adds one more `-` prefix to ancestor tag names and attributes
- No limit on nesting depth

## Edge Cases and Special Considerations

### Whitespace Preservation

All whitespace is preserved exactly as it appears in the source XML:
- Leading/trailing whitespace in text nodes
- Whitespace-only text nodes
- Whitespace in attribute values
- Newlines and indentation

### Attribute Order

Attribute order from the source XML should be preserved in the JSON output where possible (note that JSON object key order is implementation-dependent, but modern JSON parsers typically preserve insertion order).

### Empty Documents

An empty or whitespace-only XML document produces no output (zero JSON objects).

### Malformed XML

This specification assumes well-formed XML input. Behavior for malformed XML is implementation-defined and may include:
- Error reporting and termination
- Lenient parsing with warnings
- Automatic correction attempts

The chosen approach should be documented in the implementation.

## Non-Goals / Out of Scope

The following are explicitly out of scope for this specification:

1. **Filtering**: Output includes all nodes and attributes. Filtering of `@index` when not needed for disambiguation, filtering of whitespace-only text nodes, etc., are separate post-processing concerns.

2. **JSON to XML**: This specification defines XML→JSON only. The reverse transformation is possible due to losslessness but is not specified here.

3. **Schema Validation**: No validation against XML Schema, DTD, or other schema languages.

4. **XPath/XQuery**: No query language support.

5. **Pretty Printing**: Output is compact JSON (no indentation). Pretty printing is a post-processing concern.

6. **Optimization**: Performance characteristics, memory usage patterns, and optimization strategies are implementation concerns.

7. **Integration with Other Data Models**: This specification defines a standalone transformation. Integration with application-specific data structures is implementation-specific.

## Implementation Notes (For Future Reference)

When implementing this specification:

1. **Module Structure**: Implement as a standalone module with minimal dependencies
2. **Input**: Accept both `&str` and `&[u8]` inputs
3. **Output**: Return `Vec<String>` where each string is a complete JSON line
4. **Error Handling**: Use `Result` types for error propagation
5. **Testing**: Include comprehensive examples covering all edge cases in this specification
6. **Command Integration**: Add as a command to the binary after core implementation
7. **Examples Directory**: Follow existing patterns in `examples/` directory

## Revision History

- **Version 1.0** (2025-12-02): Initial specification based on deleted IDEAS.md proposal and clarifying discussions
