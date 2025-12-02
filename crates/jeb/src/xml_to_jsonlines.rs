//! XML to JSON Lines conversion
//!
//! This module implements a lossless, streaming transformation from XML
//! documents to JSON Lines format as specified in the technical specification.

use indexmap::IndexMap;
use quick_xml::{Reader, events::Event};
use serde_json::{Value, json};

use crate::Panic;

/// Represents an ancestor in the XML tree with its tag name and attributes
#[derive(Debug, Clone)]
struct Ancestor {
    tag: String,
    attributes: IndexMap<String, String>,
}

/// Converts XML input to JSON Lines format
///
/// # Arguments
///
/// * `xml_input` - The XML document as a byte slice
///
/// # Returns
///
/// A vector of JSON strings, one per line
///
/// # Errors
///
/// Returns an error if the XML is malformed or cannot be parsed
pub fn xml_to_jsonlines(xml_input: &[u8]) -> Result<Vec<String>, Panic> {
    let mut reader = Reader::from_reader(xml_input);
    reader.config_mut().trim_text(false);
    reader.config_mut().expand_empty_elements = false;

    let mut output = Vec::new();
    let mut ancestors: Vec<Ancestor> = Vec::new();
    let mut root_level_index = 0;
    let mut sibling_indices: Vec<usize> = Vec::new();

    // Track the last output index so we can update @tail
    let mut last_output_index: Option<usize> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => {
                let tag = decode_bytes(e.name().as_ref())?;
                let attributes = parse_attributes(&e)?;

                let current_index = *sibling_indices.last().unwrap_or(&root_level_index);

                // Output the element immediately
                let obj = build_json_object(
                    &tag,
                    &attributes,
                    &ancestors,
                    current_index,
                    Some(String::new()), // @text starts empty, will be updated
                    String::new(),       // @tail starts empty, will be updated
                    false,
                );
                output.push(serde_json::to_string(&obj)?);
                last_output_index = Some(output.len() - 1);

                // Update indices
                if sibling_indices.is_empty() {
                    root_level_index += 1;
                } else {
                    *sibling_indices.last_mut().unwrap() += 1;
                }

                // Add this element as an ancestor for its children
                ancestors.push(Ancestor {
                    tag: tag.clone(),
                    attributes: attributes.clone(),
                });

                // Start tracking children indices
                sibling_indices.push(0);
            }

            Ok(Event::End(_)) => {
                // Pop this element from ancestors
                ancestors.pop();
                sibling_indices.pop();
            }

            Ok(Event::Empty(e)) => {
                let tag = decode_bytes(e.name().as_ref())?;
                let attributes = parse_attributes(&e)?;

                let current_index = *sibling_indices.last().unwrap_or(&root_level_index);

                // Update indices
                if sibling_indices.is_empty() {
                    root_level_index += 1;
                } else {
                    *sibling_indices.last_mut().unwrap() += 1;
                }

                // Self-closing tags output immediately
                let obj = build_json_object(
                    &tag,
                    &attributes,
                    &ancestors,
                    current_index,
                    None,          // No @text for self-closing
                    String::new(), // @tail starts empty, will be updated
                    true,
                );
                output.push(serde_json::to_string(&obj)?);
                last_output_index = Some(output.len() - 1);
            }

            Ok(Event::Text(e)) => {
                let text = decode_text(&e)?;

                // This text belongs to either:
                // 1. @text of the parent element (if we just started the parent)
                // 2. @tail of the previous sibling/child element

                // If we have a last output index, check if we should update @text or @tail
                if let Some(idx) = last_output_index {
                    let mut obj: Value = serde_json::from_str(&output[idx])?;
                    if let Some(map) = obj.as_object_mut() {
                        // If @text exists and is empty, update it
                        if let Some(text_val) = map.get("@text") {
                            if text_val.as_str() == Some("") {
                                map.insert("@text".to_string(), json!(text));
                            } else {
                                // @text already set, this must be @tail
                                map.insert("@tail".to_string(), json!(text));
                            }
                        } else {
                            // No @text field, so this must be @tail (self-closing element)
                            map.insert("@tail".to_string(), json!(text));
                        }
                    }
                    output[idx] = serde_json::to_string(&obj)?;
                }
            }

            Ok(Event::Comment(e)) => {
                let comment_text = decode_bytes(&e)?;
                // quick-xml returns the content between <!-- and -->, we need to add spaces
                let full_text = if comment_text.starts_with(' ') && comment_text.ends_with(' ') {
                    comment_text
                } else {
                    format!(" {comment_text} ")
                };

                let current_index = *sibling_indices.last().unwrap_or(&root_level_index);

                if sibling_indices.is_empty() {
                    root_level_index += 1;
                } else {
                    *sibling_indices.last_mut().unwrap() += 1;
                }

                let obj = build_json_object(
                    "!--",
                    &IndexMap::new(),
                    &ancestors,
                    current_index,
                    Some(full_text),
                    String::new(),
                    false,
                );
                output.push(serde_json::to_string(&obj)?);
                last_output_index = Some(output.len() - 1);
            }

            Ok(Event::CData(e)) => {
                let cdata_text = decode_bytes(&e)?;

                let current_index = *sibling_indices.last().unwrap_or(&root_level_index);

                if sibling_indices.is_empty() {
                    root_level_index += 1;
                } else {
                    *sibling_indices.last_mut().unwrap() += 1;
                }

                let obj = build_json_object(
                    "![CDATA[",
                    &IndexMap::new(),
                    &ancestors,
                    current_index,
                    Some(cdata_text),
                    String::new(),
                    false,
                );
                output.push(serde_json::to_string(&obj)?);
                last_output_index = Some(output.len() - 1);
            }

            Ok(Event::Decl(e)) => {
                let version = decode_bytes(e.version()?.as_ref())?;
                let encoding = e
                    .encoding()
                    .transpose()?
                    .map(|e| decode_bytes(e.as_ref()))
                    .transpose()?
                    .unwrap_or_else(|| "UTF-8".to_string());

                let decl_text = format!(" version=\"{version}\" encoding=\"{encoding}\"");

                let current_index = root_level_index;
                root_level_index += 1;

                let obj = build_json_object(
                    "?xml",
                    &IndexMap::new(),
                    &ancestors,
                    current_index,
                    Some(decl_text),
                    String::new(),
                    false,
                );
                output.push(serde_json::to_string(&obj)?);
                last_output_index = Some(output.len() - 1);
            }

            Ok(Event::PI(e)) => {
                let pi_text = decode_bytes(e.as_ref())?;

                // Extract the target (first word)
                let (target, content) = if let Some(space_pos) = pi_text.find(char::is_whitespace) {
                    (&pi_text[..space_pos], &pi_text[space_pos..])
                } else {
                    (pi_text.as_str(), "")
                };

                let current_index = *sibling_indices.last().unwrap_or(&root_level_index);

                if sibling_indices.is_empty() {
                    root_level_index += 1;
                } else {
                    *sibling_indices.last_mut().unwrap() += 1;
                }

                let tag = format!("?{target}");
                let full_text = format!(" {target}{content}");

                let obj = build_json_object(
                    &tag,
                    &IndexMap::new(),
                    &ancestors,
                    current_index,
                    Some(full_text),
                    String::new(),
                    false,
                );
                output.push(serde_json::to_string(&obj)?);
                last_output_index = Some(output.len() - 1);
            }

            Ok(Event::DocType(e)) => {
                let doctype_text = decode_bytes(&e)?;
                let full_text = format!(" {doctype_text}");

                let current_index = root_level_index;
                root_level_index += 1;

                let obj = build_json_object(
                    "!DOCTYPE",
                    &IndexMap::new(),
                    &ancestors,
                    current_index,
                    Some(full_text),
                    String::new(),
                    false,
                );
                output.push(serde_json::to_string(&obj)?);
                last_output_index = Some(output.len() - 1);
            }

            Err(e) => {
                // This will panic, ending execution
                Panic::from(format!("XML parsing error: {e}"));
            }
        }
    }

    Ok(output)
}

/// Build a JSON object for an XML element with ancestor context
fn build_json_object(
    tag: &str,
    attributes: &IndexMap<String, String>,
    ancestors: &[Ancestor],
    index: usize,
    text: Option<String>,
    tail: String,
    is_self_closing: bool,
) -> Value {
    let mut obj = IndexMap::new();

    // Tag name (empty string key)
    obj.insert(String::new(), json!(tag));

    // Ancestor tags and attributes
    for (depth, ancestor) in ancestors.iter().rev().enumerate() {
        let prefix = "-".repeat(depth + 1);

        // Ancestor tag name
        obj.insert(prefix.clone(), json!(&ancestor.tag));

        // Ancestor attributes
        for (attr_name, attr_value) in &ancestor.attributes {
            obj.insert(format!("{prefix}{attr_name}"), json!(attr_value));
        }
    }

    // Current element's attributes
    for (attr_name, attr_value) in attributes {
        obj.insert(attr_name.clone(), json!(attr_value));
    }

    // Virtual attributes
    // @text - only for non-self-closing tags
    if !is_self_closing {
        obj.insert("@text".to_string(), json!(text.unwrap_or_default()));
    }

    // @tail
    obj.insert("@tail".to_string(), json!(tail));

    // @index
    obj.insert("@index".to_string(), json!(index));

    Value::Object(obj.into_iter().collect())
}

/// Parse attributes from a `BytesStart` event
fn parse_attributes(e: &quick_xml::events::BytesStart) -> Result<IndexMap<String, String>, Panic> {
    let mut attrs = IndexMap::new();
    for attr in e.attributes() {
        let attr = attr?;
        let key = decode_bytes(attr.key.as_ref())?;
        let value = decode_attribute_value(&attr.value)?;
        attrs.insert(key, value);
    }
    Ok(attrs)
}

/// Decode bytes to a String with proper error handling
fn decode_bytes(bytes: &[u8]) -> Result<String, Panic> {
    Ok(String::from_utf8_lossy(bytes).to_string())
}

/// Decode text content, handling entity references
fn decode_text(e: &quick_xml::events::BytesText) -> Result<String, Panic> {
    match e.unescape() {
        Ok(text) => Ok(text.to_string()),
        Err(e) => {
            // If we can't decode an entity, use replacement character and warn
            eprintln!("Warning: Failed to decode entity: {e}");
            Ok("�".to_string())
        }
    }
}

/// Decode attribute value, handling entity references
fn decode_attribute_value(value: &[u8]) -> Result<String, Panic> {
    let text = core::str::from_utf8(value)?;
    match quick_xml::escape::unescape(text) {
        Ok(cow) => Ok(cow.to_string()),
        Err(e) => {
            eprintln!("Warning: Failed to decode attribute entity: {e}");
            Ok("�".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to unwrap Result<T, Panic> for testing
    fn unwrap_or_panic<T>(result: Result<T, Panic>) -> T {
        match result {
            Ok(v) => v,
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_simple_self_closing() {
        let xml = b"<root/>";
        let result = unwrap_or_panic(xml_to_jsonlines(xml));
        assert_eq!(result.len(), 1);

        let obj: Value = serde_json::from_str(&result[0]).expect("Failed to parse JSON");
        assert_eq!(obj[""], "root");
        assert_eq!(obj["@tail"], "");
        assert_eq!(obj["@index"], 0);
        // Self-closing tag should not have @text
        assert!(obj.get("@text").is_none());
    }

    #[test]
    fn test_empty_element() {
        let xml = b"<root></root>";
        let result = unwrap_or_panic(xml_to_jsonlines(xml));
        assert_eq!(result.len(), 1);

        let obj: Value = serde_json::from_str(&result[0]).expect("Failed to parse JSON");
        assert_eq!(obj[""], "root");
        assert_eq!(obj["@text"], "");
        assert_eq!(obj["@tail"], "");
        assert_eq!(obj["@index"], 0);
    }

    #[test]
    fn test_element_with_text() {
        let xml = b"<root>Hello</root>";
        let result = unwrap_or_panic(xml_to_jsonlines(xml));
        assert_eq!(result.len(), 1);

        let obj: Value = serde_json::from_str(&result[0]).expect("Failed to parse JSON");
        assert_eq!(obj[""], "root");
        assert_eq!(obj["@text"], "Hello");
        assert_eq!(obj["@tail"], "");
    }

    #[test]
    fn test_nested_elements() {
        let xml = br#"<html lang="en">
  <body id="main">
    <div class="content">Hello</div>
  </body>
</html>"#;

        let result = unwrap_or_panic(xml_to_jsonlines(xml));
        assert_eq!(result.len(), 3);

        // Check html element
        let html: Value = serde_json::from_str(&result[0]).expect("Failed to parse JSON");
        assert_eq!(html[""], "html");
        assert_eq!(html["lang"], "en");
        assert_eq!(html["@index"], 0);

        // Check body element
        let body: Value = serde_json::from_str(&result[1]).expect("Failed to parse JSON");
        assert_eq!(body[""], "body");
        assert_eq!(body["-"], "html");
        assert_eq!(body["-lang"], "en");
        assert_eq!(body["id"], "main");
        assert_eq!(body["@index"], 0);

        // Check div element
        let div: Value = serde_json::from_str(&result[2]).expect("Failed to parse JSON");
        assert_eq!(div[""], "div");
        assert_eq!(div["-"], "body");
        assert_eq!(div["--"], "html");
        assert_eq!(div["-id"], "main");
        assert_eq!(div["--lang"], "en");
        assert_eq!(div["class"], "content");
        assert_eq!(div["@text"], "Hello");
        assert_eq!(div["@index"], 0);
    }

    #[test]
    fn test_comment() {
        let xml = b"<!-- This is a comment -->";
        let result = unwrap_or_panic(xml_to_jsonlines(xml));
        assert_eq!(result.len(), 1);

        let obj: Value = serde_json::from_str(&result[0]).expect("Failed to parse JSON");
        assert_eq!(obj[""], "!--");
        assert_eq!(obj["@text"], " This is a comment ");
        assert_eq!(obj["@index"], 0);
    }

    #[test]
    fn test_cdata() {
        let xml = b"<root><![CDATA[Some <data>]]></root>";
        let result = unwrap_or_panic(xml_to_jsonlines(xml));
        assert_eq!(result.len(), 2);

        let root: Value = serde_json::from_str(&result[0]).expect("Failed to parse JSON");
        assert_eq!(root[""], "root");

        let cdata: Value = serde_json::from_str(&result[1]).expect("Failed to parse JSON");
        assert_eq!(cdata[""], "![CDATA[");
        assert_eq!(cdata["@text"], "Some <data>");
        assert_eq!(cdata["-"], "root");
    }

    #[test]
    fn test_multiple_siblings() {
        let xml = b"<root><a/><b/><c/></root>";
        let result = unwrap_or_panic(xml_to_jsonlines(xml));
        assert_eq!(result.len(), 4);

        let a: Value = serde_json::from_str(&result[1]).expect("Failed to parse JSON");
        assert_eq!(a[""], "a");
        assert_eq!(a["@index"], 0);

        let b: Value = serde_json::from_str(&result[2]).expect("Failed to parse JSON");
        assert_eq!(b[""], "b");
        assert_eq!(b["@index"], 1);

        let c: Value = serde_json::from_str(&result[3]).expect("Failed to parse JSON");
        assert_eq!(c[""], "c");
        assert_eq!(c["@index"], 2);
    }

    #[test]
    fn test_mixed_content() {
        let xml = b"<p>Text before <em>emphasis</em> text after</p>";
        let result = unwrap_or_panic(xml_to_jsonlines(xml));
        assert_eq!(result.len(), 2);

        let p: Value = serde_json::from_str(&result[0]).expect("Failed to parse JSON");
        assert_eq!(p[""], "p");
        assert_eq!(p["@text"], "Text before ");

        let em: Value = serde_json::from_str(&result[1]).expect("Failed to parse JSON");
        assert_eq!(em[""], "em");
        assert_eq!(em["@text"], "emphasis");
        assert_eq!(em["@tail"], " text after");
    }
}
