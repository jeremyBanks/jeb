//! XML to JSON Lines Conversion
//!
//! This module implements a lossless, streaming transformation from XML documents
//! to JSON Lines format. The design flattens hierarchical XML structures into a
//! stream of individual JSON objects, where each object represents a single XML
//! node with complete ancestor context encoded via special attribute naming
//! conventions.

use indexmap::IndexMap;
use quick_xml::{
    Reader,
    escape::{resolve_html5_entity, unescape_with},
    events::{BytesCData, BytesDecl, BytesPI, BytesStart, BytesText, Event},
};
use serde_json::Value;

/// HTML5 named character entities mapped to their decoded characters.
/// This includes the most common HTML entities.
#[expect(clippy::too_many_lines)]
fn html_entity(name: &str) -> Option<&'static str> {
    // Common HTML entities - this covers the most frequently used ones
    // A complete implementation would include all ~2000+ HTML5 entities
    match name {
        // Special characters
        "nbsp" => Some("\u{00A0}"),
        "iexcl" => Some("¡"),
        "cent" => Some("¢"),
        "pound" => Some("£"),
        "curren" => Some("¤"),
        "yen" => Some("¥"),
        "brvbar" => Some("¦"),
        "sect" => Some("§"),
        "uml" => Some("¨"),
        "copy" => Some("©"),
        "ordf" => Some("ª"),
        "laquo" => Some("«"),
        "not" => Some("¬"),
        "shy" => Some("\u{00AD}"),
        "reg" => Some("®"),
        "macr" => Some("¯"),
        "deg" => Some("°"),
        "plusmn" => Some("±"),
        "sup2" => Some("²"),
        "sup3" => Some("³"),
        "acute" => Some("´"),
        "micro" => Some("µ"),
        "para" => Some("¶"),
        "middot" => Some("·"),
        "cedil" => Some("¸"),
        "sup1" => Some("¹"),
        "ordm" => Some("º"),
        "raquo" => Some("»"),
        "frac14" => Some("¼"),
        "frac12" => Some("½"),
        "frac34" => Some("¾"),
        "iquest" => Some("¿"),
        // Latin letters with diacritics
        "Agrave" => Some("À"),
        "Aacute" => Some("Á"),
        "Acirc" => Some("Â"),
        "Atilde" => Some("Ã"),
        "Auml" => Some("Ä"),
        "Aring" => Some("Å"),
        "AElig" => Some("Æ"),
        "Ccedil" => Some("Ç"),
        "Egrave" => Some("È"),
        "Eacute" => Some("É"),
        "Ecirc" => Some("Ê"),
        "Euml" => Some("Ë"),
        "Igrave" => Some("Ì"),
        "Iacute" => Some("Í"),
        "Icirc" => Some("Î"),
        "Iuml" => Some("Ï"),
        "ETH" => Some("Ð"),
        "Ntilde" => Some("Ñ"),
        "Ograve" => Some("Ò"),
        "Oacute" => Some("Ó"),
        "Ocirc" => Some("Ô"),
        "Otilde" => Some("Õ"),
        "Ouml" => Some("Ö"),
        "times" => Some("×"),
        "Oslash" => Some("Ø"),
        "Ugrave" => Some("Ù"),
        "Uacute" => Some("Ú"),
        "Ucirc" => Some("Û"),
        "Uuml" => Some("Ü"),
        "Yacute" => Some("Ý"),
        "THORN" => Some("Þ"),
        "szlig" => Some("ß"),
        "agrave" => Some("à"),
        "aacute" => Some("á"),
        "acirc" => Some("â"),
        "atilde" => Some("ã"),
        "auml" => Some("ä"),
        "aring" => Some("å"),
        "aelig" => Some("æ"),
        "ccedil" => Some("ç"),
        "egrave" => Some("è"),
        "eacute" => Some("é"),
        "ecirc" => Some("ê"),
        "euml" => Some("ë"),
        "igrave" => Some("ì"),
        "iacute" => Some("í"),
        "icirc" => Some("î"),
        "iuml" => Some("ï"),
        "eth" => Some("ð"),
        "ntilde" => Some("ñ"),
        "ograve" => Some("ò"),
        "oacute" => Some("ó"),
        "ocirc" => Some("ô"),
        "otilde" => Some("õ"),
        "ouml" => Some("ö"),
        "divide" => Some("÷"),
        "oslash" => Some("ø"),
        "ugrave" => Some("ù"),
        "uacute" => Some("ú"),
        "ucirc" => Some("û"),
        "uuml" => Some("ü"),
        "yacute" => Some("ý"),
        "thorn" => Some("þ"),
        "yuml" => Some("ÿ"),
        // Greek letters
        "Alpha" => Some("Α"),
        "Beta" => Some("Β"),
        "Gamma" => Some("Γ"),
        "Delta" => Some("Δ"),
        "Epsilon" => Some("Ε"),
        "Zeta" => Some("Ζ"),
        "Eta" => Some("Η"),
        "Theta" => Some("Θ"),
        "Iota" => Some("Ι"),
        "Kappa" => Some("Κ"),
        "Lambda" => Some("Λ"),
        "Mu" => Some("Μ"),
        "Nu" => Some("Ν"),
        "Xi" => Some("Ξ"),
        "Omicron" => Some("Ο"),
        "Pi" => Some("Π"),
        "Rho" => Some("Ρ"),
        "Sigma" => Some("Σ"),
        "Tau" => Some("Τ"),
        "Upsilon" => Some("Υ"),
        "Phi" => Some("Φ"),
        "Chi" => Some("Χ"),
        "Psi" => Some("Ψ"),
        "Omega" => Some("Ω"),
        "alpha" => Some("α"),
        "beta" => Some("β"),
        "gamma" => Some("γ"),
        "delta" => Some("δ"),
        "epsilon" => Some("ε"),
        "zeta" => Some("ζ"),
        "eta" => Some("η"),
        "theta" => Some("θ"),
        "iota" => Some("ι"),
        "kappa" => Some("κ"),
        "lambda" => Some("λ"),
        "mu" => Some("μ"),
        "nu" => Some("ν"),
        "xi" => Some("ξ"),
        "omicron" => Some("ο"),
        "pi" => Some("π"),
        "rho" => Some("ρ"),
        "sigmaf" => Some("ς"),
        "sigma" => Some("σ"),
        "tau" => Some("τ"),
        "upsilon" => Some("υ"),
        "phi" => Some("φ"),
        "chi" => Some("χ"),
        "psi" => Some("ψ"),
        "omega" => Some("ω"),
        // Math/technical symbols
        "forall" => Some("∀"),
        "part" => Some("∂"),
        "exist" => Some("∃"),
        "empty" => Some("∅"),
        "nabla" => Some("∇"),
        "isin" => Some("∈"),
        "notin" => Some("∉"),
        "ni" => Some("∋"),
        "prod" => Some("∏"),
        "sum" => Some("∑"),
        "minus" => Some("−"),
        "lowast" => Some("∗"),
        "radic" => Some("√"),
        "prop" => Some("∝"),
        "infin" => Some("∞"),
        "ang" => Some("∠"),
        "and" => Some("∧"),
        "or" => Some("∨"),
        "cap" => Some("∩"),
        "cup" => Some("∪"),
        "int" => Some("∫"),
        "there4" => Some("∴"),
        "sim" => Some("∼"),
        "cong" => Some("≅"),
        "asymp" => Some("≈"),
        "ne" => Some("≠"),
        "equiv" => Some("≡"),
        "le" => Some("≤"),
        "ge" => Some("≥"),
        "sub" => Some("⊂"),
        "sup" => Some("⊃"),
        "nsub" => Some("⊄"),
        "sube" => Some("⊆"),
        "supe" => Some("⊇"),
        "oplus" => Some("⊕"),
        "otimes" => Some("⊗"),
        "perp" => Some("⊥"),
        "sdot" => Some("⋅"),
        // Punctuation and typography
        "bull" => Some("•"),
        "hellip" => Some("…"),
        "prime" => Some("′"),
        "Prime" => Some("″"),
        "oline" => Some("‾"),
        "frasl" => Some("⁄"),
        "ensp" => Some("\u{2002}"),
        "emsp" => Some("\u{2003}"),
        "thinsp" => Some("\u{2009}"),
        "zwnj" => Some("\u{200C}"),
        "zwj" => Some("\u{200D}"),
        "lrm" => Some("\u{200E}"),
        "rlm" => Some("\u{200F}"),
        "ndash" => Some("–"),
        "mdash" => Some("—"),
        "lsquo" => Some("\u{2018}"),
        "rsquo" => Some("\u{2019}"),
        "sbquo" => Some("\u{201A}"),
        "ldquo" => Some("\u{201C}"),
        "rdquo" => Some("\u{201D}"),
        "bdquo" => Some("\u{201E}"),
        "dagger" => Some("†"),
        "Dagger" => Some("‡"),
        "permil" => Some("‰"),
        "lsaquo" => Some("\u{2039}"),
        "rsaquo" => Some("\u{203A}"),
        "euro" => Some("€"),
        // Arrows
        "larr" => Some("←"),
        "uarr" => Some("↑"),
        "rarr" => Some("→"),
        "darr" => Some("↓"),
        "harr" => Some("↔"),
        "crarr" => Some("↵"),
        "lArr" => Some("⇐"),
        "uArr" => Some("⇑"),
        "rArr" => Some("⇒"),
        "dArr" => Some("⇓"),
        "hArr" => Some("⇔"),
        // Card suits
        "spades" => Some("♠"),
        "clubs" => Some("♣"),
        "hearts" => Some("♥"),
        "diams" => Some("♦"),
        // Additional common entities
        "OElig" => Some("Œ"),
        "oelig" => Some("œ"),
        "Scaron" => Some("Š"),
        "scaron" => Some("š"),
        "Yuml" => Some("Ÿ"),
        "fnof" => Some("ƒ"),
        "circ" => Some("ˆ"),
        "tilde" => Some("˜"),
        "trade" => Some("™"),
        _ => None,
    }
}

/// Decode entity references in text content using quick-xml's unescape function.
/// Handles built-in XML entities, numeric character references, and HTML named entities.
/// For unsupported entities, logs a warning and replaces with the replacement character.
fn decode_entities(text: &str) -> String {
    // Use quick-xml's unescape_with to handle entities
    // The escape-html feature provides HTML5 entity support
    // For any entity not handled, we provide a custom resolver
    match unescape_with(text, custom_entity_resolver) {
        Ok(decoded) => decoded.into_owned(),
        Err(e) => {
            tracing::warn!("Entity decoding error: {:?}", e);
            // Return original text if decoding fails
            text.to_string()
        }
    }
}

/// Custom entity resolver for entities not handled by quick-xml
fn custom_entity_resolver(entity: &str) -> Option<&'static str> {
    // First check our custom HTML entity table
    if let Some(value) = html_entity(entity) {
        return Some(value);
    }

    // If we get here, it's an unknown entity - log warning and return replacement char
    tracing::warn!("Unsupported entity reference: &{};", entity);
    Some("\u{FFFD}")
}

/// Resolve a single entity reference to its expanded value.
fn resolve_entity(entity: &str) -> String {
    // Built-in XML entities
    match entity {
        "lt" => return "<".to_string(),
        "gt" => return ">".to_string(),
        "amp" => return "&".to_string(),
        "quot" => return "\"".to_string(),
        "apos" => return "'".to_string(),
        _ => {}
    }

    // Numeric character references
    if let Some(hex) = entity.strip_prefix("#x").or_else(|| entity.strip_prefix("#X")) {
        // Hexadecimal: &#xNN;
        if let Ok(code) = u32::from_str_radix(hex, 16)
            && let Some(c) = char::from_u32(code) {
                return c.to_string();
            }
    } else if let Some(decimal) = entity.strip_prefix('#') {
        // Decimal: &#NN;
        if let Ok(code) = decimal.parse::<u32>()
            && let Some(c) = char::from_u32(code) {
                return c.to_string();
            }
    }

    // Try HTML5 entities via quick-xml's resolver
    if let Some(value) = resolve_html5_entity(entity) {
        return value.to_string();
    }

    // Try our custom HTML entity table
    if let Some(value) = html_entity(entity) {
        return value.to_string();
    }

    // Unknown entity - log warning and return replacement character
    tracing::warn!("Unsupported entity reference: &{};", entity);
    "\u{FFFD}".to_string()
}

/// Information about an ancestor element for encoding parent context.
#[derive(Clone, Debug)]
struct AncestorInfo {
    /// The tag name of the element.
    tag_name: String,
    /// Attributes of the element (name -> value).
    attributes: IndexMap<String, String>,
}

/// Tracks state during XML parsing for streaming output.
struct XmlToJsonlState {
    /// Stack of ancestor elements (parent at index 0, grandparent at 1, etc.).
    /// Actually stored in reverse: current parent is last element.
    ancestors: Vec<AncestorInfo>,
    /// Stack of sibling counters for each nesting level.
    /// `sibling_indices`[i] is the current sibling index at depth i.
    sibling_indices: Vec<usize>,
    /// Pending tail text to be added to the next element.
    pending_tail: Option<String>,
    /// Output JSON lines.
    output: Vec<String>,
}

impl XmlToJsonlState {
    fn new() -> Self {
        Self {
            ancestors: Vec::new(),
            sibling_indices: vec![0], // Start with root level counter
            pending_tail: None,
            output: Vec::new(),
        }
    }

    /// Get the current sibling index and increment it.
    fn get_and_increment_sibling_index(&mut self) -> usize {
        let depth = self.ancestors.len();
        // Ensure we have a counter for this depth
        while self.sibling_indices.len() <= depth {
            self.sibling_indices.push(0);
        }
        let index = self.sibling_indices[depth];
        self.sibling_indices[depth] += 1;
        index
    }

    /// Build the prefix of dashes for the given ancestor depth.
    fn ancestor_prefix(depth: usize) -> String {
        "-".repeat(depth)
    }

    /// Create a JSON object for a node.
    fn create_node_object(
        &self,
        tag_name: &str,
        own_attributes: &IndexMap<String, String>,
        text: Option<&str>,
        tail: &str,
        index: usize,
    ) -> IndexMap<String, Value> {
        let mut obj = IndexMap::new();

        // Add the tag name as empty string key
        obj.insert(String::new(), Value::String(tag_name.to_string()));

        // Add ancestor tag names and attributes (in order from parent to root)
        for (depth, ancestor) in self.ancestors.iter().rev().enumerate() {
            let prefix = Self::ancestor_prefix(depth + 1);
            // Add ancestor tag name
            obj.insert(prefix.clone(), Value::String(ancestor.tag_name.clone()));
            // Add ancestor attributes
            for (attr_name, attr_value) in &ancestor.attributes {
                let key = format!("{prefix}{attr_name}");
                obj.insert(key, Value::String(attr_value.clone()));
            }
        }

        // Add own attributes
        for (attr_name, attr_value) in own_attributes {
            obj.insert(attr_name.clone(), Value::String(attr_value.clone()));
        }

        // Add virtual attributes
        if let Some(t) = text {
            obj.insert("@text".to_string(), Value::String(t.to_string()));
        }
        obj.insert("@tail".to_string(), Value::String(tail.to_string()));
        obj.insert("@index".to_string(), Value::Number(index.into()));

        obj
    }

    /// Output a JSON line for a node.
    fn emit_node(
        &mut self,
        tag_name: &str,
        own_attributes: &IndexMap<String, String>,
        text: Option<&str>,
        is_self_closing: bool,
    ) {
        let index = self.get_and_increment_sibling_index();
        let tail = self.pending_tail.take().unwrap_or_default();

        let text_content = if is_self_closing { None } else { text };

        let obj = self.create_node_object(tag_name, own_attributes, text_content, &tail, index);
        let json = serde_json::to_string(&obj).expect("Failed to serialize JSON");
        self.output.push(json);
    }

    /// Push a new ancestor onto the stack.
    fn push_ancestor(&mut self, tag_name: String, attributes: IndexMap<String, String>) {
        self.ancestors.push(AncestorInfo {
            tag_name,
            attributes,
        });
        // Initialize sibling counter for children of this element
        let depth = self.ancestors.len();
        while self.sibling_indices.len() <= depth {
            self.sibling_indices.push(0);
        }
        self.sibling_indices[depth] = 0;
    }

    /// Pop an ancestor from the stack.
    fn pop_ancestor(&mut self) {
        self.ancestors.pop();
    }

    /// Set pending tail text for the next sibling.
    fn set_pending_tail(&mut self, tail: String) {
        self.pending_tail = Some(tail);
    }
}

/// Convert XML to JSON Lines format.
///
/// # Arguments
/// * `input` - XML content as a string slice
///
/// # Returns
/// A vector of JSON strings, each representing one XML node.
#[must_use] 
pub fn xml_to_jsonl(input: &str) -> Vec<String> {
    xml_to_jsonl_bytes(input.as_bytes())
}

/// Convert XML to JSON Lines format from bytes.
///
/// # Arguments
/// * `input` - XML content as bytes
///
/// # Returns
/// A vector of JSON strings, each representing one XML node.
#[must_use]
pub fn xml_to_jsonl_bytes(input: &[u8]) -> Vec<String> {
    let mut reader = Reader::from_reader(input);
    reader.config_mut().trim_text(false); // Preserve whitespace

    let mut state = XmlToJsonlState::new();
    let mut buf = Vec::new();

    // Track text content for elements
    let mut current_text: Option<String> = None;
    // Track if we've seen any actual content
    let mut in_element = false;
    // Track pending elements that need their text content
    let mut pending_elements: Vec<(String, IndexMap<String, String>)> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,

            Ok(Event::Decl(decl)) => {
                // XML declaration: <?xml ... ?>
                handle_xml_decl(&mut state, &decl);
            }

            Ok(Event::DocType(doctype)) => {
                // DOCTYPE: <!DOCTYPE ... >
                handle_doctype(&mut state, &doctype);
            }

            Ok(Event::PI(pi)) => {
                // Processing instruction: <?target ... ?>
                // Emit any pending element first
                emit_pending_element(&mut state, &mut pending_elements, current_text.as_ref());
                current_text = None;
                handle_pi(&mut state, &pi);
            }

            Ok(Event::Comment(comment)) => {
                // Comment: <!-- ... -->
                // Emit any pending element first
                emit_pending_element(&mut state, &mut pending_elements, current_text.as_ref());
                current_text = None;
                handle_comment(&mut state, &comment);
            }

            Ok(Event::CData(cdata)) => {
                // CDATA section: <![CDATA[ ... ]]>
                // Emit any pending element first
                emit_pending_element(&mut state, &mut pending_elements, current_text.as_ref());
                current_text = None;
                handle_cdata(&mut state, &cdata);
            }

            Ok(Event::Start(start)) => {
                // Opening tag - emit any pending element first
                emit_pending_element(&mut state, &mut pending_elements, current_text.as_ref());
                current_text = None;

                // Parse tag and attributes
                let (tag_name, attributes) = parse_element(&start);

                // Queue this element to be emitted when we know its text content
                pending_elements.push((tag_name, attributes));
                in_element = true;
            }

            Ok(Event::Empty(empty)) => {
                // Self-closing tag
                emit_pending_element(&mut state, &mut pending_elements, current_text.as_ref());
                current_text = None;

                let (tag_name, attributes) = parse_element(&empty);
                state.emit_node(&tag_name, &attributes, None, true);
            }

            Ok(Event::End(_)) => {
                // Closing tag
                emit_pending_element(&mut state, &mut pending_elements, current_text.as_ref());
                current_text = None;

                state.pop_ancestor();
                in_element = !state.ancestors.is_empty();
            }

            Ok(Event::Text(text)) => {
                // Text content
                let text_str = decode_text(&text);

                if !pending_elements.is_empty() {
                    // This is text content for the most recent pending element
                    current_text = Some(text_str);
                } else if in_element || !state.ancestors.is_empty() {
                    // This is tail text for the previous sibling
                    state.set_pending_tail(text_str);
                } else {
                    // Root level text - treat as tail for previous root node
                    state.set_pending_tail(text_str);
                }
            }

            Ok(Event::GeneralRef(r)) => {
                // Entity references - expand them to their character values
                let entity_name = String::from_utf8_lossy(r.as_ref());
                let expanded = resolve_entity(&entity_name);

                // Add to current text accumulator
                if !pending_elements.is_empty() {
                    // Add to text content for the pending element
                    if let Some(ref mut text) = current_text {
                        text.push_str(&expanded);
                    } else {
                        current_text = Some(expanded);
                    }
                } else if in_element || !state.ancestors.is_empty() {
                    // Add to tail text for the previous sibling
                    if let Some(tail) = state.pending_tail.as_mut() {
                        tail.push_str(&expanded);
                    } else {
                        state.set_pending_tail(expanded);
                    }
                } else {
                    // Root level - add to pending tail
                    if let Some(tail) = state.pending_tail.as_mut() {
                        tail.push_str(&expanded);
                    } else {
                        state.set_pending_tail(expanded);
                    }
                }
            }

            Err(e) => {
                tracing::warn!("XML parsing error: {:?}", e);
                break;
            }
        }

        buf.clear();
    }

    // Handle any remaining pending tail as final empty tail
    if let Some(tail) = state.pending_tail.take() {
        // This tail belongs to the last emitted node, but we've already emitted it
        // We need to update the last output line to include this tail
        if let Some(last) = state.output.last_mut()
            && let Ok(mut obj) = serde_json::from_str::<IndexMap<String, Value>>(last) {
                obj.insert("@tail".to_string(), Value::String(tail));
                *last = serde_json::to_string(&obj).expect("Failed to serialize JSON");
            }
    }

    state.output
}

/// Emit any pending element with its accumulated text content.
fn emit_pending_element(
    state: &mut XmlToJsonlState,
    pending_elements: &mut Vec<(String, IndexMap<String, String>)>,
    current_text: Option<&String>,
) {
    if let Some((tag_name, attributes)) = pending_elements.pop() {
        let text = current_text.map_or("", String::as_str);
        state.emit_node(&tag_name, &attributes, Some(text), false);
        state.push_ancestor(tag_name, attributes);
    }
}

/// Parse element tag name and attributes.
fn parse_element(start: &BytesStart) -> (String, IndexMap<String, String>) {
    let tag_name = String::from_utf8_lossy(start.name().as_ref()).to_string();
    let mut attributes = IndexMap::new();

    for attr in start.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let value = decode_entities(&String::from_utf8_lossy(&attr.value));
        attributes.insert(key, value);
    }

    (tag_name, attributes)
}

/// Decode text content, handling entities.
fn decode_text(text: &BytesText) -> String {
    let raw = String::from_utf8_lossy(text.as_ref());
    decode_entities(&raw)
}

/// Handle XML declaration.
fn handle_xml_decl(state: &mut XmlToJsonlState, decl: &BytesDecl) {
    // Build the content string from the declaration
    let mut content = String::new();

    if let Ok(version) = decl.version() {
        content.push_str(" version=\"");
        content.push_str(&String::from_utf8_lossy(&version));
        content.push('"');
    }

    if let Some(Ok(encoding)) = decl.encoding() {
        content.push_str(" encoding=\"");
        content.push_str(&String::from_utf8_lossy(&encoding));
        content.push('"');
    }

    if let Some(Ok(standalone)) = decl.standalone() {
        content.push_str(" standalone=\"");
        content.push_str(&String::from_utf8_lossy(&standalone));
        content.push('"');
    }

    let attributes = IndexMap::new();
    state.emit_node("?xml", &attributes, Some(&content), false);
}

/// Handle DOCTYPE declaration.
fn handle_doctype(state: &mut XmlToJsonlState, doctype: &BytesText) {
    let content = format!(" {}", String::from_utf8_lossy(doctype.as_ref()));
    let attributes = IndexMap::new();
    state.emit_node("!DOCTYPE", &attributes, Some(&content), false);
}

/// Handle processing instruction.
fn handle_pi(state: &mut XmlToJsonlState, pi: &BytesPI) {
    let target = String::from_utf8_lossy(pi.target());
    let content = String::from_utf8_lossy(pi.content());

    let tag_name = format!("?{target}");
    let attributes = IndexMap::new();
    // content() already returns everything after the target
    // We need to add a leading space to match the spec format
    let text = if content.is_empty() {
        String::new()
    } else {
        format!(" {content}")
    };
    state.emit_node(&tag_name, &attributes, Some(&text), false);
}

/// Handle comment.
fn handle_comment(state: &mut XmlToJsonlState, comment: &BytesText) {
    let content = String::from_utf8_lossy(comment.as_ref()).to_string();
    let attributes = IndexMap::new();
    state.emit_node("!--", &attributes, Some(&content), false);
}

/// Handle CDATA section.
fn handle_cdata(state: &mut XmlToJsonlState, cdata: &BytesCData) {
    let content = String::from_utf8_lossy(cdata.as_ref()).to_string();
    let attributes = IndexMap::new();
    state.emit_node("![CDATA[", &attributes, Some(&content), false);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_jsonl(lines: &[String]) -> Vec<serde_json::Value> {
        lines
            .iter()
            .map(|s| serde_json::from_str(s).expect("Invalid JSON"))
            .collect()
    }

    #[test]
    fn test_simple_element() {
        let xml = "<root>hello</root>";
        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 1);

        let parsed = parse_jsonl(&result);
        assert_eq!(parsed[0][""], "root");
        assert_eq!(parsed[0]["@text"], "hello");
        assert_eq!(parsed[0]["@tail"], "");
        assert_eq!(parsed[0]["@index"], 0);
    }

    #[test]
    fn test_nested_elements() {
        let xml = "<parent><child>text</child></parent>";
        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 2);

        let parsed = parse_jsonl(&result);

        // Parent
        assert_eq!(parsed[0][""], "parent");
        assert_eq!(parsed[0]["@text"], "");
        assert_eq!(parsed[0]["@index"], 0);

        // Child
        assert_eq!(parsed[1][""], "child");
        assert_eq!(parsed[1]["-"], "parent");
        assert_eq!(parsed[1]["@text"], "text");
        assert_eq!(parsed[1]["@index"], 0);
    }

    #[test]
    fn test_attributes() {
        let xml = r#"<root id="1" class="main">content</root>"#;
        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 1);

        let parsed = parse_jsonl(&result);
        assert_eq!(parsed[0][""], "root");
        assert_eq!(parsed[0]["id"], "1");
        assert_eq!(parsed[0]["class"], "main");
        assert_eq!(parsed[0]["@text"], "content");
    }

    #[test]
    fn test_ancestor_attributes() {
        let xml = r#"<a id="1"><b id="2"><c id="3">text</c></b></a>"#;
        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 3);

        let parsed = parse_jsonl(&result);

        // Element c
        assert_eq!(parsed[2][""], "c");
        assert_eq!(parsed[2]["-"], "b");
        assert_eq!(parsed[2]["--"], "a");
        assert_eq!(parsed[2]["id"], "3");
        assert_eq!(parsed[2]["-id"], "2");
        assert_eq!(parsed[2]["--id"], "1");
    }

    #[test]
    fn test_self_closing_tag() {
        let xml = "<root><self-closing/></root>";
        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 2);

        let parsed = parse_jsonl(&result);

        // Self-closing element should not have @text
        assert_eq!(parsed[1][""], "self-closing");
        assert!(!parsed[1].as_object().unwrap().contains_key("@text"));
        assert_eq!(parsed[1]["@tail"], "");
    }

    #[test]
    fn test_empty_tag_vs_self_closing() {
        let xml = "<root><self-closing/><empty></empty></root>";
        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 3);

        let parsed = parse_jsonl(&result);

        // Self-closing: no @text
        assert!(!parsed[1].as_object().unwrap().contains_key("@text"));

        // Empty but not self-closing: has @text=""
        assert_eq!(parsed[2]["@text"], "");
    }

    #[test]
    fn test_sibling_indices() {
        let xml = "<root><a/><b/><c/></root>";
        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 4);

        let parsed = parse_jsonl(&result);

        assert_eq!(parsed[0]["@index"], 0); // root
        assert_eq!(parsed[1]["@index"], 0); // a
        assert_eq!(parsed[2]["@index"], 1); // b
        assert_eq!(parsed[3]["@index"], 2); // c
    }

    #[test]
    fn test_mixed_content() {
        let xml = "<p>Text before <em>emphasis</em> text after</p>";
        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 2);

        let parsed = parse_jsonl(&result);

        assert_eq!(parsed[0][""], "p");
        assert_eq!(parsed[0]["@text"], "Text before ");

        assert_eq!(parsed[1][""], "em");
        assert_eq!(parsed[1]["@text"], "emphasis");
        assert_eq!(parsed[1]["@tail"], " text after");
    }

    #[test]
    fn test_comment() {
        let xml = "<!-- This is a comment --><root/>";
        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 2);

        let parsed = parse_jsonl(&result);

        assert_eq!(parsed[0][""], "!--");
        assert_eq!(parsed[0]["@text"], " This is a comment ");
        assert_eq!(parsed[0]["@index"], 0);
    }

    #[test]
    fn test_cdata() {
        let xml = "<root><![CDATA[Some <data>]]></root>";
        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 2);

        let parsed = parse_jsonl(&result);

        assert_eq!(parsed[1][""], "![CDATA[");
        assert_eq!(parsed[1]["@text"], "Some <data>");
    }

    #[test]
    fn test_xml_declaration() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><root/>"#;
        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 2);

        let parsed = parse_jsonl(&result);

        assert_eq!(parsed[0][""], "?xml");
        assert!(parsed[0]["@text"]
            .as_str()
            .unwrap()
            .contains(r#"version="1.0""#));
        assert!(parsed[0]["@text"]
            .as_str()
            .unwrap()
            .contains(r#"encoding="UTF-8""#));
    }

    #[test]
    fn test_doctype() {
        let xml = "<!DOCTYPE html><html/>";
        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 2);

        let parsed = parse_jsonl(&result);

        assert_eq!(parsed[0][""], "!DOCTYPE");
        assert_eq!(parsed[0]["@text"], " html");
    }

    #[test]
    fn test_processing_instruction() {
        let xml = r#"<?xml-stylesheet type="text/css" href="style.css"?><root/>"#;
        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 2);

        let parsed = parse_jsonl(&result);

        assert_eq!(parsed[0][""], "?xml-stylesheet");
        assert!(parsed[0]["@text"].as_str().unwrap().contains("text/css"));
    }

    #[test]
    fn test_entity_decoding() {
        let xml = "<p>&lt;&gt;&amp;&quot;&apos;</p>";
        let result = xml_to_jsonl(xml);
        let parsed = parse_jsonl(&result);

        assert_eq!(parsed[0]["@text"], "<>&\"'");
    }

    #[test]
    fn test_numeric_entity() {
        let xml = "<p>&#65;&#x42;</p>";
        let result = xml_to_jsonl(xml);
        let parsed = parse_jsonl(&result);

        assert_eq!(parsed[0]["@text"], "AB");
    }

    #[test]
    fn test_html_entity() {
        let xml = "<p>&nbsp;&copy;</p>";
        let result = xml_to_jsonl(xml);
        let parsed = parse_jsonl(&result);

        assert_eq!(parsed[0]["@text"], "\u{00A0}©");
    }

    #[test]
    fn test_whitespace_preservation() {
        let xml = "<root>\n  <child>\n    text\n  </child>\n</root>";
        let result = xml_to_jsonl(xml);
        let parsed = parse_jsonl(&result);

        assert_eq!(parsed[0]["@text"], "\n  ");
        assert_eq!(parsed[1]["@text"], "\n    text\n  ");
        assert_eq!(parsed[1]["@tail"], "\n");
    }

    #[test]
    fn test_deep_nesting() {
        let xml = "<a id=\"1\"><b id=\"2\"><c id=\"3\"><d id=\"4\">text</d></c></b></a>";
        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 4);

        let parsed = parse_jsonl(&result);

        // Element d should have all ancestors
        assert_eq!(parsed[3][""], "d");
        assert_eq!(parsed[3]["-"], "c");
        assert_eq!(parsed[3]["--"], "b");
        assert_eq!(parsed[3]["---"], "a");
        assert_eq!(parsed[3]["id"], "4");
        assert_eq!(parsed[3]["-id"], "3");
        assert_eq!(parsed[3]["--id"], "2");
        assert_eq!(parsed[3]["---id"], "1");
    }

    #[test]
    fn test_namespace_handling() {
        let xml = r#"<foo:bar xmlns:foo="http://example.com">text</foo:bar>"#;
        let result = xml_to_jsonl(xml);
        let parsed = parse_jsonl(&result);

        assert_eq!(parsed[0][""], "foo:bar");
        assert_eq!(parsed[0]["xmlns:foo"], "http://example.com");
    }

    #[test]
    fn test_empty_document() {
        let xml = "";
        let result = xml_to_jsonl(xml);
        assert!(result.is_empty());
    }

    #[test]
    fn test_multiple_root_level_nodes() {
        let xml = "<!-- Comment 1 --><!-- Comment 2 --><root/><!-- Comment 3 -->";
        let result = xml_to_jsonl(xml);
        assert_eq!(result.len(), 4);

        let parsed = parse_jsonl(&result);

        assert_eq!(parsed[0][""], "!--");
        assert_eq!(parsed[0]["@index"], 0);

        assert_eq!(parsed[1][""], "!--");
        assert_eq!(parsed[1]["@index"], 1);

        assert_eq!(parsed[2][""], "root");
        assert_eq!(parsed[2]["@index"], 2);

        assert_eq!(parsed[3][""], "!--");
        assert_eq!(parsed[3]["@index"], 3);
    }
}
