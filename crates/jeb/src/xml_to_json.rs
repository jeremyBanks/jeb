use indexmap::IndexMap;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde_json::Value as JsonValue;

/// Converts XML/HTML to JSON using the lossless transformation scheme.
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
/// - `@text`: Text content as first child (empty string for self-closing, null for no text)
/// - `@tail`: Text following the node
/// - `@index`: Sibling index for distinguishing identical adjacent parents
///
/// Metadata preservation:
/// - CDATA sections: `""` = `![CDATA[`, `@text` = content
/// - Comments: `""` = `!--`, `@text` = comment text
/// - Processing instructions: `""` = `?xml`, `@text` = PI content
/// - DOCTYPE: `""` = `!DOCTYPE`, `@text` = DOCTYPE content
pub fn xml_to_json(xml_bytes: &[u8]) -> Result<JsonValue, String> {
    xml_to_json_impl(xml_bytes, false)
}

/// Converts HTML to JSON using the lossless transformation scheme.
///
/// This function uses more lenient parsing suitable for HTML documents
/// that may not be well-formed XML.
pub fn html_to_json(html_bytes: &[u8]) -> Result<JsonValue, String> {
    xml_to_json_impl(html_bytes, true)
}

/// Auto-detects whether input is HTML or XML and converts to JSON.
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
    let mut top_level_items: Vec<JsonValue> = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => {
                // Reset the just_closed_child flag since we're starting a new element
                if let Some(parent) = stack.last_mut() {
                    parent.just_closed_child = false;
                }

                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                let mut node = IndexMap::new();
                node.insert("".to_string(), JsonValue::String(tag_name.clone()));

                // Add parent information
                if !stack.is_empty() {
                    add_parent_info(&mut node, &stack);
                }

                // Add attributes
                for attr in e.attributes() {
                    let attr = attr.map_err(|e| e.to_string())?;
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    node.insert(key, JsonValue::String(value));
                }

                // Initialize virtual attributes
                node.insert("@text".to_string(), JsonValue::Null);
                node.insert("@tail".to_string(), JsonValue::Null);
                node.insert("@index".to_string(), JsonValue::Number(0.into()));

                let context = NodeContext {
                    tag_name,
                    node,
                    text_content: String::new(),
                    has_element_children: false,
                    children: Vec::new(),
                    just_closed_child: false,
                };

                stack.push(context);
            }

            Ok(Event::End(_)) => {
                if let Some(mut context) = stack.pop() {
                    // Set text content
                    if !context.text_content.is_empty() {
                        context.node.insert(
                            "@text".to_string(),
                            JsonValue::String(context.text_content.clone()),
                        );
                    } else if !context.has_element_children {
                        // Self-closing tag
                        context.node.insert("@text".to_string(), JsonValue::String("".to_string()));
                    }

                    // Add children if any
                    if !context.children.is_empty() {
                        context.node.insert(
                            "children".to_string(),
                            JsonValue::Array(context.children.clone()),
                        );
                        context.has_element_children = true;
                    }

                    let node_value = JsonValue::Object(
                        context.node.into_iter().collect()
                    );

                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(node_value);
                        parent.has_element_children = true;
                        parent.just_closed_child = true;
                    } else {
                        top_level_items.push(node_value);
                    }
                }
            }

            Ok(Event::Empty(e)) => {
                // Reset the just_closed_child flag since we're starting a new element
                if let Some(parent) = stack.last_mut() {
                    parent.just_closed_child = false;
                }

                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                let mut node = IndexMap::new();
                node.insert("".to_string(), JsonValue::String(tag_name.clone()));

                // Add parent information
                if !stack.is_empty() {
                    add_parent_info(&mut node, &stack);
                }

                // Add attributes
                for attr in e.attributes() {
                    let attr = attr.map_err(|e| e.to_string())?;
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    node.insert(key, JsonValue::String(value));
                }

                // Self-closing tag has empty string for @text
                node.insert("@text".to_string(), JsonValue::String("".to_string()));
                node.insert("@tail".to_string(), JsonValue::Null);
                node.insert("@index".to_string(), JsonValue::Number(0.into()));

                let node_value = JsonValue::Object(
                    node.into_iter().collect()
                );

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node_value);
                    parent.has_element_children = true;
                    parent.just_closed_child = true;
                } else {
                    top_level_items.push(node_value);
                }
            }

            Ok(Event::Text(e)) => {
                let text = e.unescape().map_err(|e| e.to_string())?;
                if let Some(context) = stack.last_mut() {
                    if context.just_closed_child && !context.children.is_empty() {
                        // Text after a child element - this is tail text
                        // Modify the last child to set its @tail
                        if let Some(JsonValue::Object(last_child)) = context.children.last_mut() {
                            let current_tail = last_child.get("@tail").cloned();
                            let new_tail = match current_tail {
                                Some(JsonValue::String(s)) => {
                                    // Append to existing tail
                                    JsonValue::String(format!("{}{}", s, text))
                                }
                                _ => {
                                    // Set new tail
                                    JsonValue::String(text.to_string())
                                }
                            };
                            last_child.insert("@tail".to_string(), new_tail);
                        }
                        // Don't reset just_closed_child here - multiple text nodes can follow
                    } else {
                        // Text inside parent, before any child elements
                        context.text_content.push_str(&text);
                    }
                }
            }

            Ok(Event::CData(e)) => {
                // Reset the just_closed_child flag since we're processing a new node
                if let Some(parent) = stack.last_mut() {
                    parent.just_closed_child = false;
                }

                let content = String::from_utf8_lossy(&e.into_inner()).to_string();

                let mut node = IndexMap::new();
                node.insert("".to_string(), JsonValue::String("![CDATA[".to_string()));
                node.insert("@text".to_string(), JsonValue::String(content));
                node.insert("@tail".to_string(), JsonValue::Null);
                node.insert("@index".to_string(), JsonValue::Number(0.into()));

                // Add parent information
                if !stack.is_empty() {
                    add_parent_info(&mut node, &stack);
                }

                let node_value = JsonValue::Object(
                    node.into_iter().collect()
                );

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node_value);
                    parent.has_element_children = true;
                    parent.just_closed_child = true;
                } else {
                    top_level_items.push(node_value);
                }
            }

            Ok(Event::Comment(e)) => {
                // Reset the just_closed_child flag since we're processing a new node
                if let Some(parent) = stack.last_mut() {
                    parent.just_closed_child = false;
                }

                let comment = String::from_utf8_lossy(&e.into_inner()).to_string();

                let mut node = IndexMap::new();
                node.insert("".to_string(), JsonValue::String("!--".to_string()));
                node.insert("@text".to_string(), JsonValue::String(format!(" {} ", comment)));
                node.insert("@tail".to_string(), JsonValue::Null);
                node.insert("@index".to_string(), JsonValue::Number(0.into()));

                // Add parent information
                if !stack.is_empty() {
                    add_parent_info(&mut node, &stack);
                }

                let node_value = JsonValue::Object(
                    node.into_iter().collect()
                );

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node_value);
                    parent.has_element_children = true;
                    parent.just_closed_child = true;
                } else {
                    top_level_items.push(node_value);
                }
            }

            Ok(Event::Decl(e)) => {
                // Reset the just_closed_child flag since we're processing a new node
                if let Some(parent) = stack.last_mut() {
                    parent.just_closed_child = false;
                }

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
                node.insert("@text".to_string(), JsonValue::String(content));
                node.insert("@tail".to_string(), JsonValue::Null);
                node.insert("@index".to_string(), JsonValue::Number(0.into()));

                // Add parent information
                if !stack.is_empty() {
                    add_parent_info(&mut node, &stack);
                }

                let node_value = JsonValue::Object(
                    node.into_iter().collect()
                );

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node_value);
                    parent.has_element_children = true;
                    parent.just_closed_child = true;
                } else {
                    top_level_items.push(node_value);
                }
            }

            Ok(Event::DocType(e)) => {
                // Reset the just_closed_child flag since we're processing a new node
                if let Some(parent) = stack.last_mut() {
                    parent.just_closed_child = false;
                }

                let content = String::from_utf8_lossy(&e.into_inner()).to_string();

                let mut node = IndexMap::new();
                node.insert("".to_string(), JsonValue::String("!DOCTYPE".to_string()));
                node.insert("@text".to_string(), JsonValue::String(format!(" {}", content)));
                node.insert("@tail".to_string(), JsonValue::Null);
                node.insert("@index".to_string(), JsonValue::Number(0.into()));

                // Add parent information
                if !stack.is_empty() {
                    add_parent_info(&mut node, &stack);
                }

                let node_value = JsonValue::Object(
                    node.into_iter().collect()
                );

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node_value);
                    parent.has_element_children = true;
                    parent.just_closed_child = true;
                } else {
                    top_level_items.push(node_value);
                }
            }

            Ok(Event::PI(e)) => {
                // Reset the just_closed_child flag since we're processing a new node
                if let Some(parent) = stack.last_mut() {
                    parent.just_closed_child = false;
                }

                let content = String::from_utf8_lossy(&e.into_inner()).to_string();
                let parts: Vec<&str> = content.splitn(2, ' ').collect();
                let target = parts.get(0).unwrap_or(&"");
                let data = parts.get(1).unwrap_or(&"");

                let mut node = IndexMap::new();
                node.insert("".to_string(), JsonValue::String(format!("?{}", target)));
                node.insert("@text".to_string(), JsonValue::String(format!(" {}", data)));
                node.insert("@tail".to_string(), JsonValue::Null);
                node.insert("@index".to_string(), JsonValue::Number(0.into()));

                // Add parent information
                if !stack.is_empty() {
                    add_parent_info(&mut node, &stack);
                }

                let node_value = JsonValue::Object(
                    node.into_iter().collect()
                );

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node_value);
                    parent.has_element_children = true;
                    parent.just_closed_child = true;
                } else {
                    top_level_items.push(node_value);
                }
            }

            Err(e) => return Err(format!("XML parsing error at position {}: {}", reader.buffer_position(), e)),
        }

        buf.clear();
    }

    // If we have multiple top-level items (declaration, comments, root element),
    // return them all wrapped in an array. If only one, return it directly.
    match top_level_items.len() {
        0 => Err("No root element found".to_string()),
        1 => Ok(top_level_items.into_iter().next().unwrap()),
        _ => Ok(JsonValue::Array(top_level_items)),
    }
}

struct NodeContext {
    tag_name: String,
    node: IndexMap<String, JsonValue>,
    text_content: String,
    has_element_children: bool,
    children: Vec<JsonValue>,
    just_closed_child: bool,
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
        for (key, value) in &ancestor.node {
            if !key.is_empty() && !key.starts_with('@') && !key.starts_with('-') && key != "children" {
                node.insert(
                    format!("{}{}", prefix, key),
                    value.clone(),
                );
            }
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
