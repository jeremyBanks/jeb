use indexmap::IndexMap;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde_json::Value;

/// Converts XML data to JSON Lines format following the specification.
///
/// Each XML node is converted to a JSON object on a single line, with ancestor
/// context encoded via special attribute naming conventions using `-` prefixes.
pub fn xml_to_jsonlines(xml_data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_reader(xml_data);
    reader.trim_text(false);
    reader.expand_empty_elements(false);

    let mut events = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(event) => events.push(event),
            Err(e) => {
                eprintln!("Warning: XML parsing error: {}", e);
                break;
            }
        }
    }

    // Build intermediate representation and collect JSON lines
    let mut json_lines = Vec::new();
    let mut ancestor_stack: Vec<AncestorInfo> = Vec::new();
    let mut sibling_counters: Vec<usize> = Vec::new();

    process_events(&events, &mut json_lines, &mut ancestor_stack, &mut sibling_counters)?;

    // Output
    let output = json_lines
        .into_iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(output)
}

#[derive(Clone, Debug)]
struct AncestorInfo {
    tag: String,
    attributes: IndexMap<String, String>,
}

fn process_events(
    events: &[Event],
    json_lines: &mut Vec<Value>,
    ancestor_stack: &mut Vec<AncestorInfo>,
    sibling_counters: &mut Vec<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut i = 0;

    while i < events.len() {

        match &events[i] {
            Event::Decl(decl) => {
                let content = String::from_utf8_lossy(&decl).to_string();
                let obj = create_special_node("?xml", &content, "", ancestor_stack)?;
                json_lines.push(obj);
                increment_sibling_index(sibling_counters);
                i += 1;
            }
            Event::DocType(doctype) => {
                let content = String::from_utf8_lossy(doctype).to_string();
                let obj = create_special_node("!DOCTYPE", &content, "", ancestor_stack)?;
                json_lines.push(obj);
                increment_sibling_index(sibling_counters);
                i += 1;
            }
            Event::Comment(bytes) => {
                let text = String::from_utf8_lossy(bytes).to_string();
                let obj = create_special_node("!--", &text, "", ancestor_stack)?;
                json_lines.push(obj);
                increment_sibling_index(sibling_counters);
                i += 1;
            }
            Event::Start(start) => {
                let tag_name = String::from_utf8_lossy(start.name().as_ref()).to_string();
                let mut attributes = IndexMap::new();

                for attr in start.attributes() {
                    let attr = attr?;
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = attr.decode_and_unescape_value(&Reader::from_reader(&b""[..]))?;
                    attributes.insert(key, value.to_string());
                }

                // Collect text content between start tag and first child/end tag
                let text_content = collect_text_until_element(&events[i + 1..]);

                // Create the element object
                let obj = create_element_node(
                    &tag_name,
                    &text_content,
                    "",
                    ancestor_stack,
                    &attributes,
                    get_current_index(sibling_counters),
                )?;
                json_lines.push(obj);

                // Push to ancestor stack
                ancestor_stack.push(AncestorInfo {
                    tag: tag_name,
                    attributes,
                });
                sibling_counters.push(0);

                i += 1;
            }
            Event::Empty(empty) => {
                let tag_name = String::from_utf8_lossy(empty.name().as_ref()).to_string();
                let mut attributes = IndexMap::new();

                for attr in empty.attributes() {
                    let attr = attr?;
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = attr.decode_and_unescape_value(&Reader::from_reader(&b""[..]))?;
                    attributes.insert(key, value.to_string());
                }

                // Self-closing element: no @text field
                let obj = create_empty_element_node(
                    &tag_name,
                    "",
                    ancestor_stack,
                    &attributes,
                    get_current_index(sibling_counters),
                )?;
                json_lines.push(obj);
                increment_sibling_index(sibling_counters);

                // Collect and apply tail text after empty element
                let tail_text = collect_text_until_element(&events[i + 1..]);
                if let Some(Value::Object(map)) = json_lines.last_mut() {
                    map.insert("@tail".to_string(), Value::String(tail_text));
                }

                i += 1;
            }
            Event::End(_end) => {
                ancestor_stack.pop();
                if !sibling_counters.is_empty() {
                    sibling_counters.pop();
                }
                increment_sibling_index(sibling_counters);

                // Collect and apply tail text after this end tag
                let tail_text = collect_text_until_element(&events[i + 1..]);
                if let Some(Value::Object(map)) = json_lines.last_mut() {
                    map.insert("@tail".to_string(), Value::String(tail_text));
                }

                i += 1;
            }
            Event::CData(cdata) => {
                let text = String::from_utf8_lossy(&cdata).to_string();
                let obj = create_special_node("![CDATA[", &text, "", ancestor_stack)?;
                json_lines.push(obj);
                increment_sibling_index(sibling_counters);

                // Collect and apply tail text after CDATA
                let tail_text = collect_text_until_element(&events[i + 1..]);
                if let Some(Value::Object(map)) = json_lines.last_mut() {
                    map.insert("@tail".to_string(), Value::String(tail_text));
                }

                i += 1;
            }
            Event::PI(pi) => {
                let pi_data = String::from_utf8_lossy(&pi).to_string();
                let target = if let Some(space_pos) = pi_data.find(' ') {
                    &pi_data[..space_pos]
                } else {
                    &pi_data
                };
                let obj = create_special_node(&format!("?{}", target), &pi_data, "", ancestor_stack)?;
                json_lines.push(obj);
                increment_sibling_index(sibling_counters);

                // Collect and apply tail text after PI
                let tail_text = collect_text_until_element(&events[i + 1..]);
                if let Some(Value::Object(map)) = json_lines.last_mut() {
                    map.insert("@tail".to_string(), Value::String(tail_text));
                }

                i += 1;
            }
            Event::Text(_) => {
                // Text is handled when processing Start/End elements
                i += 1;
            }
            _ => {
                // Handle other events
                i += 1;
            }
        }
    }

    Ok(())
}

fn collect_text_until_element(events: &[Event]) -> String {
    let mut text = String::new();

    for event in events {
        match event {
            Event::Text(bytes) => {
                if let Ok(decoded) = bytes.unescape() {
                    text.push_str(&decoded);
                }
            }
            Event::Start(_) | Event::Empty(_) | Event::End(_) => break,
            _ => {}
        }
    }

    text
}

fn get_current_index(sibling_counters: &[usize]) -> usize {
    sibling_counters.last().copied().unwrap_or(0)
}

fn increment_sibling_index(sibling_counters: &mut Vec<usize>) {
    if let Some(last) = sibling_counters.last_mut() {
        *last += 1;
    }
}

fn create_element_node(
    tag: &str,
    text: &str,
    tail: &str,
    ancestor_stack: &[AncestorInfo],
    attributes: &IndexMap<String, String>,
    index: usize,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut obj = serde_json::Map::new();

    obj.insert("".to_string(), Value::String(tag.to_string()));

    // Add ancestor tags and attributes (in reverse order, closest ancestor first)
    let len = ancestor_stack.len();
    for (idx, ancestor) in ancestor_stack.iter().enumerate() {
        let depth = len - idx; // Distance from current element (1 = immediate parent)
        let prefix = "-".repeat(depth);
        obj.insert(prefix.clone(), Value::String(ancestor.tag.clone()));

        for (attr_name, attr_value) in &ancestor.attributes {
            let attr_key = format!("{}{}", prefix, attr_name);
            obj.insert(attr_key, Value::String(attr_value.clone()));
        }
    }

    // Add element's own attributes
    for (key, value) in attributes {
        obj.insert(key.clone(), Value::String(value.clone()));
    }

    // Add virtual attributes
    obj.insert("@text".to_string(), Value::String(text.to_string()));
    obj.insert("@tail".to_string(), Value::String(tail.to_string()));
    obj.insert("@index".to_string(), Value::Number(index.into()));

    Ok(Value::Object(obj))
}

fn create_empty_element_node(
    tag: &str,
    _tail: &str,
    ancestor_stack: &[AncestorInfo],
    attributes: &IndexMap<String, String>,
    index: usize,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut obj = serde_json::Map::new();

    obj.insert("".to_string(), Value::String(tag.to_string()));

    // Add ancestor tags and attributes (in reverse order, closest ancestor first)
    let len = ancestor_stack.len();
    for (idx, ancestor) in ancestor_stack.iter().enumerate() {
        let depth = len - idx; // Distance from current element (1 = immediate parent)
        let prefix = "-".repeat(depth);
        obj.insert(prefix.clone(), Value::String(ancestor.tag.clone()));

        for (attr_name, attr_value) in &ancestor.attributes {
            let attr_key = format!("{}{}", prefix, attr_name);
            obj.insert(attr_key, Value::String(attr_value.clone()));
        }
    }

    // Add element's own attributes
    for (key, value) in attributes {
        obj.insert(key.clone(), Value::String(value.clone()));
    }

    // Self-closing: no @text field, @tail will be set after by caller
    obj.insert("@tail".to_string(), Value::String(String::new()));
    obj.insert("@index".to_string(), Value::Number(index.into()));

    Ok(Value::Object(obj))
}

fn create_special_node(
    tag: &str,
    text: &str,
    tail: &str,
    ancestor_stack: &[AncestorInfo],
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut obj = serde_json::Map::new();

    obj.insert("".to_string(), Value::String(tag.to_string()));

    // Add ancestor tags and attributes (in reverse order, closest ancestor first)
    let len = ancestor_stack.len();
    for (idx, ancestor) in ancestor_stack.iter().enumerate() {
        let depth = len - idx; // Distance from current element (1 = immediate parent)
        let prefix = "-".repeat(depth);
        obj.insert(prefix.clone(), Value::String(ancestor.tag.clone()));

        for (attr_name, attr_value) in &ancestor.attributes {
            let attr_key = format!("{}{}", prefix, attr_name);
            obj.insert(attr_key, Value::String(attr_value.clone()));
        }
    }

    let index = if ancestor_stack.is_empty() { 0 } else { 0 }; // Root level nodes get index 0
    obj.insert("@text".to_string(), Value::String(text.to_string()));
    obj.insert("@tail".to_string(), Value::String(tail.to_string()));
    obj.insert("@index".to_string(), Value::Number(index.into()));

    Ok(Value::Object(obj))
}
