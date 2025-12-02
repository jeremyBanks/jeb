# XML to JSON Lines Examples

This directory contains example XML files and their JSON Lines conversions.

## Posts.xml

Source: [Stack Exchange Data Dump - 3dprinting.meta.stackexchange.com](https://archive.org/download/stackexchange_20250630_rev2/stackexchange_20250630_rev2/3dprinting.meta.stackexchange.com.7z)

This is a real-world XML file from the Stack Exchange network containing meta posts from the 3D Printing Stack Exchange site.

### File Details

- **Posts.xml** (1.1 MB): Original XML file with 595 post entries
- **Posts.jsonlines** (207 KB): First 128 lines of JSON Lines output

### Usage

Convert the entire file:
```bash
jeb /path/to/Posts.xml xml-to-jsonlines
```

Convert and save output:
```bash
jeb /path/to/Posts.xml xml-to-jsonlines > output.jsonlines
```

### Sample Output

The XML structure like:
```xml
<?xml version="1.0" encoding="utf-8"?>
<!--
  ContentLicense
  ...
-->
<posts>
  <row Id="6" PostTypeId="1" CreationDate="2016-01-12T20:30:19.493"
       Score="5" ViewCount="114" Body="&lt;p&gt;From..."
       OwnerUserId="63" Title="What should our documentation contain?"
       Tags="&lt;discussion&gt;&lt;7-questions&gt;" AnswerCount="3" />
  ...
</posts>
```

Converts to JSON Lines where each node becomes a separate JSON object:

```json
{"":"?xml","@text":" version=\"1.0\" encoding=\"utf-8\"","@tail":"\r\n","@index":0}
{"":"!--","@text":"\r\n  ContentLicense\r\n\r\n  CC BY-SA 2.5...","@tail":"\r\n","@index":1}
{"":"posts","@text":"\r\n  ","@tail":"","@index":2}
{"":"row","-":"posts","Body":"<p>From <a href=\"...","OwnerUserId":"63","CreationDate":"2016-01-12T20:30:19.493",...,"@tail":"\r\n  ","@index":0}
```

### Key Features Demonstrated

1. **Special Nodes**: XML declaration (`?xml`), comments (`!--`)
2. **Ancestor Context**: Each `row` element includes `"-":"posts"` to show its parent
3. **Entity Decoding**: HTML entities like `&lt;` are decoded to `<` in attribute values
4. **Structural Metadata**:
   - `@text`: Text content within elements
   - `@tail`: Text after closing tag
   - `@index`: Position among siblings
5. **Lossless Conversion**: All information needed to reconstruct the original XML is preserved
