use indexmap::IndexMap;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde_json::Value as JsonValue;

/// Converts XML/HTML to JSON Lines using the lossless transformation scheme.
///
/// This function can parse both well-formed XML and lenient HTML.
///
/// The transformation uses a special naming scheme for lossless round-tripping:
/// - `""` (empty string): The node's tag name
/// - `"-"`: Parent tag name
/// - `"--"`: Grandparent tag name (and so on)
/// - `"-attribute-name"`: Parent's attribute values
/// - `"--attribute-name"`: Grandparent's attribute values
///
/// Virtual attributes (always present):
/// - `@text`: Text content (empty string for self-closing, null for no text)
/// - `@tail`: Text following the node's closing tag
/// - `@index`: Sibling index for distinguishing identical adjacent parents
///
/// Metadata preservation:
/// - CDATA sections: `""` = `![CDATA[`, `@text` = content
/// - Comments: `""` = `!--`, `@text` = comment text
/// - Processing instructions: `""` = `?xml`, `@text` = PI content
/// - DOCTYPE: `""` = `!DOCTYPE`, `@text` = DOCTYPE content
///
/// Returns an array of JSON objects (one per element), suitable for JSON Lines output.
pub fn xml_to_json(xml_bytes: &[u8]) -> Result<JsonValue, String> {
    xml_to_json_impl(xml_bytes, false)
}

/// Converts HTML to JSON Lines using the lossless transformation scheme.
///
/// This function uses more lenient parsing suitable for HTML documents
/// that may not be well-formed XML.
pub fn html_to_json(html_bytes: &[u8]) -> Result<JsonValue, String> {
    xml_to_json_impl(html_bytes, true)
}

/// Auto-detects whether input is HTML or XML and converts to JSON Lines.
///
/// Detection logic:
/// - Starts with `<!DOCTYPE html>` (case-insensitive) → HTML
/// - Starts with `<html` (case-insensitive) → HTML
/// - Starts with `<?xml` → XML
/// - Otherwise → XML (default)
pub fn parse_markup(bytes: &[u8]) -> Result<JsonValue, String> {
    let is_html = detect_html(bytes);
    xml_to_json_impl(bytes, is_html)
}

fn detect_html(bytes: &[u8]) -> bool {
    let trimmed = bytes.iter()
        .skip_while(|&&b| b.is_ascii_whitespace())
        .copied()
        .take(200)
        .collect::<Vec<u8>>();

    let lower = trimmed.to_ascii_lowercase();

    // Check for HTML DOCTYPE
    if lower.starts_with(b"<!doctype html") {
        return true;
    }

    // Check for <html tag
    if lower.starts_with(b"<html") {
        return true;
    }

    false
}

struct NodeContext {
    tag_name: String,
    attributes: IndexMap<String, String>,
    text_content: String,
    index: usize,
}

fn xml_to_json_impl(xml_bytes: &[u8], lenient: bool) -> Result<JsonValue, String> {
    let mut reader = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(false);
    reader.config_mut().expand_empty_elements = true;

    if lenient {
        // More lenient settings for HTML
        reader.config_mut().check_end_names = false;
        reader.config_mut().check_comments = false;
        reader.config_mut().allow_unmatched_ends = true;
    }

    let mut stack: Vec<NodeContext> = Vec::new();
    let mut output: Vec<JsonValue> = Vec::new();
    let mut last_closed_element_index: Option<usize> = None;
    let mut sibling_counts: Vec<usize> = vec![0];
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => {
                // Starting a new element, so clear the last closed element flag
                last_closed_element_index = None;

                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                // Collect attributes
                let mut attributes = IndexMap::new();
                for attr in e.attributes() {
                    let attr = attr.map_err(|e| e.to_string())?;
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    attributes.insert(key, value);
                }

                let index = *sibling_counts.last().unwrap_or(&0);

                stack.push(NodeContext {
                    tag_name,
                    attributes,
                    text_content: String::new(),
                    index,
                });

                sibling_counts.push(0);
            }

            Ok(Event::End(_)) => {
                if let Some(context) = stack.pop() {
                    sibling_counts.pop();

                    let mut node = IndexMap::new();

                    // Tag name
                    node.insert("".to_string(), JsonValue::String(context.tag_name.clone()));

                    // Add parent information
                    add_parent_info(&mut node, &stack);

                    // Add element's own attributes
                    for (key, value) in context.attributes {
                        node.insert(key, JsonValue::String(value));
                    }

                    // @text
                    if context.text_content.is_empty() {
                        node.insert("@text".to_string(), JsonValue::Null);
                    } else {
                        node.insert("@text".to_string(), JsonValue::String(context.text_content));
                    }

                    // @tail placeholder - will be updated if next event is text
                    node.insert("@tail".to_string(), JsonValue::Null);

                    // @index
                    node.insert("@index".to_string(), JsonValue::Number(context.index.into()));

                    let element_index = output.len();
                    output.push(JsonValue::Object(node.into_iter().collect()));

                    // Remember this element so we can set its @tail if needed
                    last_closed_element_index = Some(element_index);

                    // Increment sibling count for parent
                    if let Some(count) = sibling_counts.last_mut() {
                        *count += 1;
                    }
                }
            }

            Ok(Event::Empty(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                let mut node = IndexMap::new();
                node.insert("".to_string(), JsonValue::String(tag_name));

                // Add parent information
                add_parent_info(&mut node, &stack);

                // Add attributes
                for attr in e.attributes() {
                    let attr = attr.map_err(|e| e.to_string())?;
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    node.insert(key, JsonValue::String(value));
                }

                // Self-closing tag has empty string for @text
                node.insert("@text".to_string(), JsonValue::String("".to_string()));

                // @tail placeholder - will be updated if next event is text
                node.insert("@tail".to_string(), JsonValue::Null);

                let index = *sibling_counts.last().unwrap_or(&0);
                node.insert("@index".to_string(), JsonValue::Number(index.into()));

                let element_index = output.len();
                output.push(JsonValue::Object(node.into_iter().collect()));

                // Remember this element so we can set its @tail if needed
                last_closed_element_index = Some(element_index);

                // Increment sibling count
                if let Some(count) = sibling_counts.last_mut() {
                    *count += 1;
                }
            }

            Ok(Event::Text(e)) => {
                let text = e.unescape().map_err(|e| e.to_string())?;

                if let Some(element_index) = last_closed_element_index {
                    // We just closed an element, so this text is its tail
                    if let Some(JsonValue::Object(obj)) = output.get_mut(element_index) {
                        let current_tail = obj.get("@tail").cloned();
                        let new_tail = match current_tail {
                            Some(JsonValue::String(s)) => {
                                JsonValue::String(format!("{}{}", s, text))
                            }
                            _ => JsonValue::String(text.to_string()),
                        };
                        obj.insert("@tail".to_string(), new_tail);
                    }
                    // Don't clear last_closed_element_index yet - multiple text nodes can follow
                } else if let Some(context) = stack.last_mut() {
                    // Text inside an element (no element was just closed)
                    context.text_content.push_str(&text);
                }
            }

            Ok(Event::CData(e)) => {
                let content = String::from_utf8_lossy(&e.into_inner()).to_string();

                let mut node = IndexMap::new();
                node.insert("".to_string(), JsonValue::String("![CDATA[".to_string()));

                // Add parent information
                add_parent_info(&mut node, &stack);

                node.insert("@text".to_string(), JsonValue::String(content));
                node.insert("@tail".to_string(), JsonValue::Null);

                let index = *sibling_counts.last().unwrap_or(&0);
                node.insert("@index".to_string(), JsonValue::Number(index.into()));

                let element_index = output.len();
                output.push(JsonValue::Object(node.into_iter().collect()));

                // Remember this element so we can set its @tail if needed
                last_closed_element_index = Some(element_index);

                if let Some(count) = sibling_counts.last_mut() {
                    *count += 1;
                }
            }

            Ok(Event::Comment(e)) => {
                let comment = String::from_utf8_lossy(&e.into_inner()).to_string();

                let mut node = IndexMap::new();
                node.insert("".to_string(), JsonValue::String("!--".to_string()));

                // Add parent information
                add_parent_info(&mut node, &stack);

                node.insert("@text".to_string(), JsonValue::String(format!(" {} ", comment)));
                node.insert("@tail".to_string(), JsonValue::Null);

                let index = *sibling_counts.last().unwrap_or(&0);
                node.insert("@index".to_string(), JsonValue::Number(index.into()));

                let element_index = output.len();
                output.push(JsonValue::Object(node.into_iter().collect()));

                // Remember this element so we can set its @tail if needed
                last_closed_element_index = Some(element_index);

                if let Some(count) = sibling_counts.last_mut() {
                    *count += 1;
                }
            }

            Ok(Event::Decl(e)) => {
                let version = e.version().map_err(|e| e.to_string())?;
                let encoding = e.encoding().transpose().map_err(|e| e.to_string())?;
                let standalone = e.standalone().transpose().map_err(|e| e.to_string())?;

                let mut content = format!(" version=\"{}\"", String::from_utf8_lossy(&version));
                if let Some(enc) = encoding {
                    content.push_str(&format!(" encoding=\"{}\"", String::from_utf8_lossy(&enc)));
                }
                if let Some(sa) = standalone {
                    content.push_str(&format!(" standalone=\"{}\"", String::from_utf8_lossy(&sa)));
                }

                let mut node = IndexMap::new();
                node.insert("".to_string(), JsonValue::String("?xml".to_string()));

                // Add parent information
                add_parent_info(&mut node, &stack);

                node.insert("@text".to_string(), JsonValue::String(content));
                node.insert("@tail".to_string(), JsonValue::Null);

                let index = *sibling_counts.last().unwrap_or(&0);
                node.insert("@index".to_string(), JsonValue::Number(index.into()));

                let element_index = output.len();
                output.push(JsonValue::Object(node.into_iter().collect()));

                // Remember this element so we can set its @tail if needed
                last_closed_element_index = Some(element_index);

                if let Some(count) = sibling_counts.last_mut() {
                    *count += 1;
                }
            }

            Ok(Event::DocType(e)) => {
                let content = String::from_utf8_lossy(&e.into_inner()).to_string();

                let mut node = IndexMap::new();
                node.insert("".to_string(), JsonValue::String("!DOCTYPE".to_string()));

                // Add parent information
                add_parent_info(&mut node, &stack);

                node.insert("@text".to_string(), JsonValue::String(format!(" {}", content)));
                node.insert("@tail".to_string(), JsonValue::Null);

                let index = *sibling_counts.last().unwrap_or(&0);
                node.insert("@index".to_string(), JsonValue::Number(index.into()));

                let element_index = output.len();
                output.push(JsonValue::Object(node.into_iter().collect()));

                // Remember this element so we can set its @tail if needed
                last_closed_element_index = Some(element_index);

                if let Some(count) = sibling_counts.last_mut() {
                    *count += 1;
                }
            }

            Ok(Event::PI(e)) => {
                let content = String::from_utf8_lossy(&e.into_inner()).to_string();
                let parts: Vec<&str> = content.splitn(2, ' ').collect();
                let target = parts.get(0).unwrap_or(&"");
                let data = parts.get(1).unwrap_or(&"");

                let mut node = IndexMap::new();
                node.insert("".to_string(), JsonValue::String(format!("?{}", target)));

                // Add parent information
                add_parent_info(&mut node, &stack);

                node.insert("@text".to_string(), JsonValue::String(format!(" {}", data)));
                node.insert("@tail".to_string(), JsonValue::Null);

                let index = *sibling_counts.last().unwrap_or(&0);
                node.insert("@index".to_string(), JsonValue::Number(index.into()));

                let element_index = output.len();
                output.push(JsonValue::Object(node.into_iter().collect()));

                // Remember this element so we can set its @tail if needed
                last_closed_element_index = Some(element_index);

                if let Some(count) = sibling_counts.last_mut() {
                    *count += 1;
                }
            }

            Err(e) => return Err(format!("XML parsing error at position {}: {}", reader.buffer_position(), e)),
        }

        buf.clear();
    }

    if output.is_empty() {
        return Err("No elements found".to_string());
    }

    Ok(JsonValue::Array(output))
}

fn add_parent_info(node: &mut IndexMap<String, JsonValue>, stack: &[NodeContext]) {
    for (depth, ancestor) in stack.iter().rev().enumerate() {
        let prefix = "-".repeat(depth + 1);

        // Add parent tag name
        node.insert(
            prefix.clone(),
            JsonValue::String(ancestor.tag_name.clone()),
        );

        // Add parent's attributes with prefix
        for (key, value) in &ancestor.attributes {
            node.insert(
                format!("{}{}", prefix, key),
                JsonValue::String(value.clone()),
            );
        }
    }

    // Add null for ancestor beyond tree
    let depth = stack.len();
    node.insert(
        "-".repeat(depth + 1),
        JsonValue::Null,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_xml() {
        let xml = br#"<root><child>text</child></root>"#;
        let result = xml_to_json(xml);
        assert!(result.is_ok());
        println!("{}", serde_json::to_string_pretty(&result.unwrap()).unwrap());
    }

    #[test]
    fn test_xml_with_attributes() {
        let xml = br#"<book id="123"><title>Example</title></book>"#;
        let result = xml_to_json(xml);
        assert!(result.is_ok());
        println!("{}", serde_json::to_string_pretty(&result.unwrap()).unwrap());
    }

    #[test]
    fn test_xml_with_comment() {
        let xml = br#"<root><!-- comment --><child/></root>"#;
        let result = xml_to_json(xml);
        assert!(result.is_ok());
        println!("{}", serde_json::to_string_pretty(&result.unwrap()).unwrap());
    }

    #[test]
    fn test_xml_with_cdata() {
        let xml = br#"<root><![CDATA[some data]]></root>"#;
        let result = xml_to_json(xml);
        assert!(result.is_ok());
        println!("{}", serde_json::to_string_pretty(&result.unwrap()).unwrap());
    }
}
