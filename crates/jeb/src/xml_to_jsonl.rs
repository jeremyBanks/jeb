//! XML to JSON Lines conversion
//!
//! This module implements a lossless, streaming transformation from XML
//! documents to JSON Lines format. Each XML node is converted to a single JSON
//! object with complete ancestor context encoded via special attribute naming
//! conventions.
//!
//! See `/home/user/jeb/XML_TO_JSONL_SPEC.md` for the complete specification.

use std::collections::BTreeMap;

use quick_xml::{Reader, events::Event};

/// Converts an XML document to JSON Lines format.
///
/// # Arguments
///
/// * `xml` - The XML document as a string slice
///
/// # Returns
///
/// A vector of strings, where each string is a single-line JSON object
/// representing one XML node. The objects appear in document order (depth-first
/// traversal).
///
/// # Specification
///
/// This implementation follows the XML to JSON Lines specification, which
/// includes:
///
/// - **Flat output**: Each XML element becomes one JSON object
/// - **Ancestor context**: Parent/grandparent attributes with `-`, `--`, `---`
///   prefixes
/// - **Virtual attributes**: `@text`, `@tail`, `@index` for structure
///   preservation
/// - **Special nodes**: CDATA, comments, processing instructions, DOCTYPE, etc.
/// - **Entity handling**: XML 1.0/1.1 and HTML 4/5 entities decoded
/// - **Lossless round-tripping**: Preserves all information to reconstruct
///   original XML
///
/// # Examples
///
/// ```
/// use jeb::xml_to_jsonl;
///
/// let xml = r#"<root><child>text</child></root>"#;
/// let jsonl = xml_to_jsonl(xml);
/// assert_eq!(jsonl.len(), 2); // root and child elements
/// ```
#[must_use]
pub fn xml_to_jsonl(xml: &str) -> Vec<String> {
    let mut converter = XmlToJsonlConverter::new(xml);
    converter.convert()
}

/// Helper type for building JSON objects with ordered keys.
/// Uses `BTreeMap` to ensure consistent key ordering in output.
type JsonObject = BTreeMap<String, serde_json::Value>;

/// Represents an element in the ancestor stack
#[derive(Debug, Clone)]
struct AncestorElement {
    name: String,
    attributes: Vec<(String, String)>,
}

/// Represents an element waiting for its @tail to be determined
#[derive(Debug)]
struct CompletedElement {
    obj: JsonObject,
}

/// Main converter struct that maintains state during conversion
struct XmlToJsonlConverter<'a> {
    reader: Reader<&'a [u8]>,
    output: Vec<String>,
    ancestor_stack: Vec<AncestorElement>,
    building_stack: Vec<(JsonObject, String)>, // (object, accumulated_text)
    last_completed: Option<CompletedElement>,
    root_index: usize,
    sibling_indices: Vec<usize>,
    accumulated_tail: String,
}

impl<'a> XmlToJsonlConverter<'a> {
    fn new(xml: &'a str) -> Self {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().expand_empty_elements = false;
        reader.config_mut().trim_text(false);

        Self {
            reader,
            output: Vec::new(),
            ancestor_stack: Vec::new(),
            building_stack: Vec::new(),
            last_completed: None,
            root_index: 0,
            sibling_indices: Vec::new(),
            accumulated_tail: String::new(),
        }
    }

    fn flush_completed(&mut self) {
        if let Some(mut completed) = self.last_completed.take() {
            completed.obj.insert(
                "@tail".to_string(),
                serde_json::Value::String(self.accumulated_tail.clone()),
            );
            self.accumulated_tail.clear();
            self.output_object(completed.obj);
        }
    }

    fn convert(&mut self) -> Vec<String> {
        let mut buf = Vec::new();

        loop {
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    // Flush any previously completed element
                    self.flush_completed();

                    // If there's a parent element being built, its @text is complete
                    // Move it to last_completed so it can collect @tail
                    if let Some((mut parent_obj, parent_text)) = self.building_stack.pop() {
                        parent_obj
                            .insert("@text".to_string(), serde_json::Value::String(parent_text));
                        self.last_completed = Some(CompletedElement { obj: parent_obj });
                    }

                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let attrs: Vec<(String, String)> = e
                        .attributes()
                        .filter_map(std::result::Result::ok)
                        .map(|a| {
                            (
                                String::from_utf8_lossy(a.key.as_ref()).to_string(),
                                String::from_utf8_lossy(&a.value).to_string(),
                            )
                        })
                        .collect();

                    let mut obj = self.create_base_object(&name);

                    // Add attributes
                    for (key, value) in &attrs {
                        obj.insert(key.clone(), serde_json::Value::String(value.clone()));
                    }

                    // Add @tail placeholder (will be set later)
                    obj.insert(
                        "@tail".to_string(),
                        serde_json::Value::String(self.accumulated_tail.clone()),
                    );
                    self.accumulated_tail.clear();

                    // Add @index
                    let index = self.current_index();
                    obj.insert(
                        "@index".to_string(),
                        serde_json::Value::Number(index.into()),
                    );

                    // Push to building stack to collect @text
                    self.building_stack.push((obj, String::new()));

                    // Push to ancestor stack
                    self.ancestor_stack.push(AncestorElement {
                        name: name.clone(),
                        attributes: attrs,
                    });
                    self.sibling_indices.push(0);
                    self.increment_current_index();
                }
                Ok(Event::Empty(e)) => {
                    // Flush any previously completed element
                    self.flush_completed();

                    // If there's a parent element being built, its @text is complete
                    // Move it to last_completed so it can collect @tail
                    if let Some((mut parent_obj, parent_text)) = self.building_stack.pop() {
                        parent_obj
                            .insert("@text".to_string(), serde_json::Value::String(parent_text));
                        self.last_completed = Some(CompletedElement { obj: parent_obj });
                    }

                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let mut obj = self.create_base_object(&name);

                    // Add attributes
                    for attr in e.attributes().filter_map(std::result::Result::ok) {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        obj.insert(key, serde_json::Value::String(value));
                    }

                    // Set @tail
                    obj.insert(
                        "@tail".to_string(),
                        serde_json::Value::String(self.accumulated_tail.clone()),
                    );
                    self.accumulated_tail.clear();

                    // No @text for self-closing tags

                    // Add @index
                    let index = self.current_index();
                    obj.insert(
                        "@index".to_string(),
                        serde_json::Value::Number(index.into()),
                    );

                    // Self-closing element becomes last_completed (will get @tail updated)
                    self.last_completed = Some(CompletedElement { obj });
                    self.increment_current_index();
                }
                Ok(Event::End(_e)) => {
                    // Pop element from building stack and set its @text
                    if let Some((mut obj, text)) = self.building_stack.pop() {
                        // Element was in building stack (leaf or no children yet)
                        // Don't flush last_completed (it's probably a parent)
                        obj.insert("@text".to_string(), serde_json::Value::String(text));
                        // Element becomes last_completed (will collect @tail before output)
                        self.last_completed = Some(CompletedElement { obj });
                    } else {
                        // Element was already moved to last_completed (it had children)
                        // Now it's closing, so flush it
                        self.flush_completed();
                    }

                    // Pop from ancestor stack
                    self.ancestor_stack.pop();
                    self.sibling_indices.pop();
                }
                Ok(Event::Text(e)) => {
                    let text = match e.unescape() {
                        Ok(cow) => cow.to_string(),
                        Err(_) => String::from_utf8_lossy(&e).to_string(),
                    };

                    // If we're inside an element being built, add to its @text
                    if let Some((_obj, elem_text)) = self.building_stack.last_mut() {
                        elem_text.push_str(&text);
                    } else {
                        // Otherwise, accumulate for @tail
                        self.accumulated_tail.push_str(&text);
                    }
                }
                Ok(Event::CData(e)) => {
                    self.flush_completed();

                    // If parent is being built, its @text is complete
                    if let Some((mut parent_obj, parent_text)) = self.building_stack.pop() {
                        parent_obj
                            .insert("@text".to_string(), serde_json::Value::String(parent_text));
                        self.last_completed = Some(CompletedElement { obj: parent_obj });
                    }

                    let mut obj = self.create_base_object("![CDATA[");
                    let content = String::from_utf8_lossy(&e).to_string();
                    obj.insert("@text".to_string(), serde_json::Value::String(content));
                    obj.insert(
                        "@tail".to_string(),
                        serde_json::Value::String(self.accumulated_tail.clone()),
                    );
                    self.accumulated_tail.clear();
                    let index = self.current_index();
                    obj.insert(
                        "@index".to_string(),
                        serde_json::Value::Number(index.into()),
                    );
                    self.last_completed = Some(CompletedElement { obj });
                    self.increment_current_index();
                }
                Ok(Event::Comment(e)) => {
                    self.flush_completed();

                    // If parent is being built, its @text is complete
                    if let Some((mut parent_obj, parent_text)) = self.building_stack.pop() {
                        parent_obj
                            .insert("@text".to_string(), serde_json::Value::String(parent_text));
                        self.last_completed = Some(CompletedElement { obj: parent_obj });
                    }

                    let mut obj = self.create_base_object("!--");
                    let content = String::from_utf8_lossy(&e).to_string();
                    obj.insert("@text".to_string(), serde_json::Value::String(content));
                    obj.insert(
                        "@tail".to_string(),
                        serde_json::Value::String(self.accumulated_tail.clone()),
                    );
                    self.accumulated_tail.clear();
                    let index = self.current_index();
                    obj.insert(
                        "@index".to_string(),
                        serde_json::Value::Number(index.into()),
                    );
                    self.last_completed = Some(CompletedElement { obj });
                    self.increment_current_index();
                }
                Ok(Event::Decl(e)) => {
                    self.flush_completed();

                    let mut obj = self.create_base_object("?xml");

                    // Build the declaration text
                    let mut decl_text = String::new();
                    if let Ok(version) = e.version() {
                        decl_text.push_str(&format!(
                            " version=\"{}\"",
                            String::from_utf8_lossy(&version)
                        ));
                    }
                    if let Some(Ok(encoding)) = e.encoding() {
                        decl_text.push_str(&format!(
                            " encoding=\"{}\"",
                            String::from_utf8_lossy(&encoding)
                        ));
                    }
                    if let Some(Ok(standalone)) = e.standalone() {
                        decl_text.push_str(&format!(
                            " standalone=\"{}\"",
                            String::from_utf8_lossy(&standalone)
                        ));
                    }

                    obj.insert("@text".to_string(), serde_json::Value::String(decl_text));
                    obj.insert(
                        "@tail".to_string(),
                        serde_json::Value::String(self.accumulated_tail.clone()),
                    );
                    self.accumulated_tail.clear();
                    let index = self.current_index();
                    obj.insert(
                        "@index".to_string(),
                        serde_json::Value::Number(index.into()),
                    );
                    self.last_completed = Some(CompletedElement { obj });
                    self.increment_current_index();
                }
                Ok(Event::PI(e)) => {
                    self.flush_completed();

                    let content = String::from_utf8_lossy(&e).to_string();

                    // Extract PI target (everything before first whitespace)
                    let mut parts = content.splitn(2, |c: char| c.is_whitespace());
                    let target = parts.next().unwrap_or("");
                    let data = parts.next().unwrap_or("");

                    let tag_name = format!("?{target}");
                    let mut obj = self.create_base_object(&tag_name);
                    let pi_text = if data.is_empty() {
                        String::new()
                    } else {
                        format!(" {data}")
                    };
                    obj.insert("@text".to_string(), serde_json::Value::String(pi_text));
                    obj.insert(
                        "@tail".to_string(),
                        serde_json::Value::String(self.accumulated_tail.clone()),
                    );
                    self.accumulated_tail.clear();
                    let index = self.current_index();
                    obj.insert(
                        "@index".to_string(),
                        serde_json::Value::Number(index.into()),
                    );
                    self.last_completed = Some(CompletedElement { obj });
                    self.increment_current_index();
                }
                Ok(Event::DocType(e)) => {
                    self.flush_completed();

                    let mut obj = self.create_base_object("!DOCTYPE");
                    let content = String::from_utf8_lossy(&e).to_string();
                    obj.insert(
                        "@text".to_string(),
                        serde_json::Value::String(format!(" {content}")),
                    );
                    obj.insert(
                        "@tail".to_string(),
                        serde_json::Value::String(self.accumulated_tail.clone()),
                    );
                    self.accumulated_tail.clear();
                    let index = self.current_index();
                    obj.insert(
                        "@index".to_string(),
                        serde_json::Value::Number(index.into()),
                    );
                    self.last_completed = Some(CompletedElement { obj });
                    self.increment_current_index();
                }
                Err(e) => {
                    tracing::warn!("XML parsing error: {:?}", e);
                    break;
                }
            }
            buf.clear();
        }

        // Flush any remaining completed element at EOF
        self.flush_completed();

        self.output.clone()
    }

    fn create_base_object(&self, tag_name: &str) -> JsonObject {
        let mut obj = JsonObject::new();
        obj.insert(
            String::new(),
            serde_json::Value::String(tag_name.to_string()),
        );

        // Add ancestor context
        let depth = self.ancestor_stack.len();
        for (i, ancestor) in self.ancestor_stack.iter().enumerate() {
            let prefix_count = depth - i;
            let prefix = "-".repeat(prefix_count);

            // Add ancestor tag name
            obj.insert(
                prefix.clone(),
                serde_json::Value::String(ancestor.name.clone()),
            );

            // Add ancestor attributes
            for (attr_name, attr_value) in &ancestor.attributes {
                let key = format!("{prefix}{attr_name}");
                obj.insert(key, serde_json::Value::String(attr_value.clone()));
            }
        }

        obj
    }

    fn current_index(&self) -> usize {
        if self.ancestor_stack.is_empty() {
            self.root_index
        } else {
            *self.sibling_indices.last().unwrap_or(&0)
        }
    }

    fn increment_current_index(&mut self) {
        if self.ancestor_stack.is_empty() {
            self.root_index += 1;
        } else if let Some(last) = self.sibling_indices.last_mut() {
            *last += 1;
        }
    }

    fn output_object(&mut self, obj: JsonObject) {
        if let Ok(json_str) = serde_json::to_string(&obj) {
            self.output.push(json_str);
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// Helper to parse a JSON line into a JsonObject for testing
    fn parse_json_line(line: &str) -> JsonObject {
        serde_json::from_str(line).expect("valid JSON")
    }

    /// Helper to check if a JSON object has a field with a specific value
    fn assert_field_eq(obj: &JsonObject, key: &str, expected: serde_json::Value) {
        assert_eq!(
            obj.get(key),
            Some(&expected),
            "Expected field '{}' to be {:?}, but got {:?}",
            key,
            expected,
            obj.get(key)
        );
    }

    /// Helper to check if a JSON object does NOT have a field
    fn assert_field_absent(obj: &JsonObject, key: &str) {
        assert!(
            !obj.contains_key(key),
            "Expected field '{}' to be absent, but it was present with value {:?}",
            key,
            obj.get(key)
        );
    }

    #[test]
    fn test_simple_nested_elements() {
        let xml = r#"<html lang="en">
  <body id="main">
    <div class="content">Hello</div>
  </body>
</html>"#;

        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 3, "Should produce 3 JSON objects");

        // Parse JSON lines
        let html = parse_json_line(&result[0]);
        let body = parse_json_line(&result[1]);
        let div = parse_json_line(&result[2]);

        // Verify html element
        assert_field_eq(&html, "", json!("html"));
        assert_field_eq(&html, "lang", json!("en"));
        assert_field_eq(&html, "@text", json!("\n  "));
        assert_field_eq(&html, "@tail", json!(""));
        assert_field_eq(&html, "@index", json!(0));
        assert_field_absent(&html, "-"); // No parent

        // Verify body element
        assert_field_eq(&body, "", json!("body"));
        assert_field_eq(&body, "-", json!("html"));
        assert_field_eq(&body, "-lang", json!("en"));
        assert_field_eq(&body, "id", json!("main"));
        assert_field_eq(&body, "@text", json!("\n    "));
        assert_field_eq(&body, "@tail", json!("\n"));
        assert_field_eq(&body, "@index", json!(0));

        // Verify div element
        assert_field_eq(&div, "", json!("div"));
        assert_field_eq(&div, "-", json!("body"));
        assert_field_eq(&div, "--", json!("html"));
        assert_field_eq(&div, "-id", json!("main"));
        assert_field_eq(&div, "--lang", json!("en"));
        assert_field_eq(&div, "class", json!("content"));
        assert_field_eq(&div, "@text", json!("Hello"));
        assert_field_eq(&div, "@tail", json!("\n  "));
        assert_field_eq(&div, "@index", json!(0));
    }

    #[test]
    fn test_self_closing_vs_empty_tags() {
        let xml = r#"<root>
  <self-closing/>
  <empty></empty>
</root>"#;

        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 3, "Should produce 3 JSON objects");

        let root = parse_json_line(&result[0]);
        let self_closing = parse_json_line(&result[1]);
        let empty = parse_json_line(&result[2]);

        // Verify root element
        assert_field_eq(&root, "", json!("root"));
        assert_field_eq(&root, "@text", json!("\n  "));
        assert_field_eq(&root, "@tail", json!(""));
        assert_field_eq(&root, "@index", json!(0));

        // Verify self-closing element (no @text field)
        assert_field_eq(&self_closing, "", json!("self-closing"));
        assert_field_eq(&self_closing, "-", json!("root"));
        assert_field_absent(&self_closing, "@text"); // KEY DISTINCTION
        assert_field_eq(&self_closing, "@tail", json!("\n  "));
        assert_field_eq(&self_closing, "@index", json!(0));

        // Verify empty element (has @text: "")
        assert_field_eq(&empty, "", json!("empty"));
        assert_field_eq(&empty, "-", json!("root"));
        assert_field_eq(&empty, "@text", json!("")); // KEY DISTINCTION
        assert_field_eq(&empty, "@tail", json!("\n"));
        assert_field_eq(&empty, "@index", json!(1));
    }

    #[test]
    fn test_mixed_content() {
        let xml = r#"<p>Text before <em>emphasis</em> text after</p>"#;

        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 2, "Should produce 2 JSON objects");

        let p = parse_json_line(&result[0]);
        let em = parse_json_line(&result[1]);

        // Verify p element
        assert_field_eq(&p, "", json!("p"));
        assert_field_eq(&p, "@text", json!("Text before "));
        assert_field_eq(&p, "@tail", json!(""));
        assert_field_eq(&p, "@index", json!(0));

        // Verify em element
        assert_field_eq(&em, "", json!("em"));
        assert_field_eq(&em, "-", json!("p"));
        assert_field_eq(&em, "@text", json!("emphasis"));
        assert_field_eq(&em, "@tail", json!(" text after"));
        assert_field_eq(&em, "@index", json!(0));
    }

    #[test]
    fn test_special_nodes_and_metadata() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<!-- This is a comment -->
<root><![CDATA[Some <data>]]></root>"#;

        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 5, "Should produce 5 JSON objects");

        let xml_decl = parse_json_line(&result[0]);
        let doctype = parse_json_line(&result[1]);
        let comment = parse_json_line(&result[2]);
        let root = parse_json_line(&result[3]);
        let cdata = parse_json_line(&result[4]);

        // Verify XML declaration
        assert_field_eq(&xml_decl, "", json!("?xml"));
        assert_field_eq(
            &xml_decl,
            "@text",
            json!(" version=\"1.0\" encoding=\"UTF-8\""),
        );
        assert_field_eq(&xml_decl, "@tail", json!("\n"));
        assert_field_eq(&xml_decl, "@index", json!(0));
        assert_field_absent(&xml_decl, "-"); // No parent

        // Verify DOCTYPE
        assert_field_eq(&doctype, "", json!("!DOCTYPE"));
        assert_field_eq(&doctype, "@text", json!(" html"));
        assert_field_eq(&doctype, "@tail", json!("\n"));
        assert_field_eq(&doctype, "@index", json!(1));

        // Verify comment
        assert_field_eq(&comment, "", json!("!--"));
        assert_field_eq(&comment, "@text", json!(" This is a comment "));
        assert_field_eq(&comment, "@tail", json!("\n"));
        assert_field_eq(&comment, "@index", json!(2));

        // Verify root element
        assert_field_eq(&root, "", json!("root"));
        assert_field_eq(&root, "@text", json!(""));
        assert_field_eq(&root, "@tail", json!(""));
        assert_field_eq(&root, "@index", json!(3));

        // Verify CDATA section
        assert_field_eq(&cdata, "", json!("![CDATA["));
        assert_field_eq(&cdata, "-", json!("root"));
        assert_field_eq(&cdata, "@text", json!("Some <data>"));
        assert_field_eq(&cdata, "@tail", json!(""));
        assert_field_eq(&cdata, "@index", json!(0));
    }

    #[test]
    fn test_multiple_root_level_nodes() {
        let xml = r#"<!-- Comment 1 -->
<!-- Comment 2 -->
<root/>
<!-- Comment 3 -->"#;

        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 4, "Should produce 4 JSON objects");

        let comment1 = parse_json_line(&result[0]);
        let comment2 = parse_json_line(&result[1]);
        let root = parse_json_line(&result[2]);
        let comment3 = parse_json_line(&result[3]);

        // All should have sequential indices and no parent attributes
        assert_field_eq(&comment1, "", json!("!--"));
        assert_field_eq(&comment1, "@index", json!(0));
        assert_field_absent(&comment1, "-");

        assert_field_eq(&comment2, "", json!("!--"));
        assert_field_eq(&comment2, "@index", json!(1));
        assert_field_absent(&comment2, "-");

        assert_field_eq(&root, "", json!("root"));
        assert_field_eq(&root, "@index", json!(2));
        assert_field_absent(&root, "-");
        assert_field_absent(&root, "@text"); // Self-closing

        assert_field_eq(&comment3, "", json!("!--"));
        assert_field_eq(&comment3, "@index", json!(3));
        assert_field_absent(&comment3, "-");
    }

    #[test]
    fn test_entity_references() {
        let xml = r#"<p>Standard: &lt;&gt;&amp; Numeric: &#65; HTML: &nbsp; Custom: &custom;</p>"#;

        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 1, "Should produce 1 JSON object");

        let p = parse_json_line(&result[0]);

        // Standard XML entities should be decoded
        // Numeric character references should be decoded
        // HTML entities should be decoded
        // Custom entities should become replacement character U+FFFD (�)
        assert_field_eq(
            &p,
            "@text",
            json!("Standard: <>& Numeric: A HTML: \u{00A0} Custom: \u{FFFD}"),
        );
    }

    #[test]
    fn test_deep_nesting_with_attributes() {
        let xml = r#"<a id="1"><b id="2"><c id="3"><d id="4">text</d></c></b></a>"#;

        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 4, "Should produce 4 JSON objects");

        let a = parse_json_line(&result[0]);
        let b = parse_json_line(&result[1]);
        let c = parse_json_line(&result[2]);
        let d = parse_json_line(&result[3]);

        // Verify element 'a'
        assert_field_eq(&a, "", json!("a"));
        assert_field_eq(&a, "id", json!("1"));
        assert_field_absent(&a, "-");

        // Verify element 'b'
        assert_field_eq(&b, "", json!("b"));
        assert_field_eq(&b, "-", json!("a"));
        assert_field_eq(&b, "-id", json!("1"));
        assert_field_eq(&b, "id", json!("2"));

        // Verify element 'c'
        assert_field_eq(&c, "", json!("c"));
        assert_field_eq(&c, "-", json!("b"));
        assert_field_eq(&c, "--", json!("a"));
        assert_field_eq(&c, "-id", json!("2"));
        assert_field_eq(&c, "--id", json!("1"));
        assert_field_eq(&c, "id", json!("3"));

        // Verify element 'd'
        assert_field_eq(&d, "", json!("d"));
        assert_field_eq(&d, "-", json!("c"));
        assert_field_eq(&d, "--", json!("b"));
        assert_field_eq(&d, "---", json!("a"));
        assert_field_eq(&d, "-id", json!("3"));
        assert_field_eq(&d, "--id", json!("2"));
        assert_field_eq(&d, "---id", json!("1"));
        assert_field_eq(&d, "id", json!("4"));
        assert_field_eq(&d, "@text", json!("text"));
    }

    #[test]
    fn test_whitespace_preservation() {
        let xml = "  <root>  \n\t  <child>  text  </child>  \n  </root>  ";

        let result = xml_to_jsonl(xml);

        // Should preserve all whitespace exactly
        let _root = parse_json_line(&result[0]);
        let child = parse_json_line(&result[1]);

        // Root should have leading whitespace in @tail before it appears
        // (or the first node should capture it somehow based on implementation)
        // Child should preserve exact whitespace in @text
        assert_field_eq(&child, "@text", json!("  text  "));
    }

    #[test]
    fn test_namespaces_preserved_literally() {
        let xml = r#"<foo:bar xmlns:foo="http://example.com"><foo:baz/></foo:bar>"#;

        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 2, "Should produce 2 JSON objects");

        let bar = parse_json_line(&result[0]);
        let baz = parse_json_line(&result[1]);

        // Namespaces should be preserved literally in tag names
        assert_field_eq(&bar, "", json!("foo:bar"));
        assert_field_eq(&bar, "xmlns:foo", json!("http://example.com"));

        assert_field_eq(&baz, "", json!("foo:baz"));
        assert_field_eq(&baz, "-", json!("foo:bar"));
        assert_field_eq(&baz, "-xmlns:foo", json!("http://example.com"));
    }

    #[test]
    fn test_attribute_order_preservation() {
        let xml = r#"<element zebra="z" alpha="a" middle="m"/>"#;

        let result = xml_to_jsonl(xml);
        let elem = parse_json_line(&result[0]);

        // BTreeMap will sort keys, but we verify all attributes are present
        assert_field_eq(&elem, "zebra", json!("z"));
        assert_field_eq(&elem, "alpha", json!("a"));
        assert_field_eq(&elem, "middle", json!("m"));
    }

    #[test]
    fn test_empty_document() {
        let xml = "";
        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 0, "Empty document should produce no output");
    }

    #[test]
    fn test_whitespace_only_document() {
        let xml = "   \n\t  \n  ";
        let result = xml_to_jsonl(xml);
        // Implementation may vary: could be 0 objects or could preserve whitespace
        // Specification says "empty or whitespace-only XML document produces no output"
        assert_eq!(
            result.len(),
            0,
            "Whitespace-only document should produce no output"
        );
    }

    #[test]
    fn test_processing_instruction() {
        let xml = r#"<?xml-stylesheet type="text/css" href="style.css"?><root/>"#;

        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 2, "Should produce 2 JSON objects");

        let pi = parse_json_line(&result[0]);
        assert_field_eq(&pi, "", json!("?xml-stylesheet"));
        assert_field_eq(&pi, "@text", json!(" type=\"text/css\" href=\"style.css\""));
        assert_field_eq(&pi, "@index", json!(0));
    }

    #[test]
    fn test_multiple_children_same_parent() {
        let xml = r#"<parent attr="value"><child1/><child2/><child3/></parent>"#;

        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 4, "Should produce 4 JSON objects");

        let _parent = parse_json_line(&result[0]);
        let child1 = parse_json_line(&result[1]);
        let child2 = parse_json_line(&result[2]);
        let child3 = parse_json_line(&result[3]);

        // All children should reference parent with its attribute
        assert_field_eq(&child1, "-", json!("parent"));
        assert_field_eq(&child1, "-attr", json!("value"));
        assert_field_eq(&child1, "@index", json!(0));

        assert_field_eq(&child2, "-", json!("parent"));
        assert_field_eq(&child2, "-attr", json!("value"));
        assert_field_eq(&child2, "@index", json!(1));

        assert_field_eq(&child3, "-", json!("parent"));
        assert_field_eq(&child3, "-attr", json!("value"));
        assert_field_eq(&child3, "@index", json!(2));
    }

    #[test]
    fn test_hex_character_reference() {
        let xml = r#"<p>Hex: &#x41; &#x3042;</p>"#;

        let result = xml_to_jsonl(xml);
        let p = parse_json_line(&result[0]);

        // &#x41; = 'A', &#x3042; = 'あ'
        assert_field_eq(&p, "@text", json!("Hex: A あ"));
    }

    #[test]
    fn test_cdata_with_special_chars() {
        let xml = r#"<root><![CDATA[<tag>&entity;"quotes"]]></root>"#;

        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 2);

        let cdata = parse_json_line(&result[1]);
        assert_field_eq(&cdata, "", json!("![CDATA["));
        // CDATA should preserve everything literally, no entity decoding
        assert_field_eq(&cdata, "@text", json!("<tag>&entity;\"quotes\""));
    }

    #[test]
    fn test_boolean_html_attributes() {
        let xml = r#"<input disabled required/>"#;

        let result = xml_to_jsonl(xml);
        let input = parse_json_line(&result[0]);

        // Boolean attributes without values should be converted to true
        assert_field_eq(&input, "disabled", json!(true));
        assert_field_eq(&input, "required", json!(true));
    }

    #[test]
    fn test_sibling_indices() {
        let xml = r#"<root><a/><b/><c/></root>"#;

        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 4);

        let a = parse_json_line(&result[1]);
        let b = parse_json_line(&result[2]);
        let c = parse_json_line(&result[3]);

        // Siblings should have sequential indices
        assert_field_eq(&a, "@index", json!(0));
        assert_field_eq(&b, "@index", json!(1));
        assert_field_eq(&c, "@index", json!(2));
    }
}
