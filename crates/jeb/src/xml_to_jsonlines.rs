/// XML to JSON Lines conversion
///
/// This module implements a lossless transformation from XML documents to JSON
/// Lines format. Each XML node is converted to a JSON object with complete
/// ancestor context.
use std::collections::HashMap;

/// Decode XML and HTML entity references in text
fn decode_entities(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '&' {
            // Collect characters until ';'
            let mut entity = String::new();
            let mut found_semicolon = false;

            for next_ch in chars.by_ref() {
                if next_ch == ';' {
                    found_semicolon = true;
                    break;
                }
                entity.push(next_ch);
                // Limit entity length to avoid consuming too much
                if entity.len() > 20 {
                    break;
                }
            }

            if found_semicolon {
                // Try to decode the entity
                let decoded = decode_single_entity(&entity);
                result.push_str(&decoded);
            } else {
                // Not a valid entity, keep the & and continue
                result.push('&');
                result.push_str(&entity);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Decode a single entity reference (without & and ;)
fn decode_single_entity(entity: &str) -> String {
    // Numeric character references
    if let Some(hex) = entity
        .strip_prefix("#x")
        .or_else(|| entity.strip_prefix("#X"))
    {
        if let Ok(code) = u32::from_str_radix(hex, 16)
            && let Some(ch) = char::from_u32(code)
        {
            return ch.to_string();
        }
        eprintln!(
            "Warning: Invalid hexadecimal character reference: &{entity}; - replacing with �"
        );
        return "\u{FFFD}".to_string();
    }

    if let Some(decimal) = entity.strip_prefix("#") {
        if let Ok(code) = decimal.parse::<u32>()
            && let Some(ch) = char::from_u32(code)
        {
            return ch.to_string();
        }
        eprintln!("Warning: Invalid decimal character reference: &{entity}; - replacing with �");
        return "\u{FFFD}".to_string();
    }

    // Named entities
    match entity {
        // XML predefined entities
        "amp" => "&".to_string(),
        "lt" => "<".to_string(),
        "gt" => ">".to_string(),
        "quot" => "\"".to_string(),
        "apos" => "'".to_string(),

        // Common HTML entities
        "nbsp" => "\u{00A0}".to_string(),
        "copy" => "©".to_string(),
        "reg" => "®".to_string(),
        "trade" => "™".to_string(),
        "euro" => "€".to_string(),
        "cent" => "¢".to_string(),
        "pound" => "£".to_string(),
        "yen" => "¥".to_string(),
        "mdash" => "—".to_string(),
        "ndash" => "–".to_string(),
        "hellip" => "…".to_string(),
        "middot" => "·".to_string(),
        "bull" => "•".to_string(),
        "lsquo" => "\u{2018}".to_string(),
        "rsquo" => "\u{2019}".to_string(),
        "ldquo" => "\u{201C}".to_string(),
        "rdquo" => "\u{201D}".to_string(),
        "laquo" => "\u{00AB}".to_string(),
        "raquo" => "\u{00BB}".to_string(),
        "iexcl" => "\u{00A1}".to_string(),
        "iquest" => "\u{00BF}".to_string(),
        "sect" => "\u{00A7}".to_string(),
        "para" => "\u{00B6}".to_string(),
        "dagger" => "\u{2020}".to_string(),
        "Dagger" => "\u{2021}".to_string(),
        "permil" => "\u{2030}".to_string(),
        "lsaquo" => "\u{2039}".to_string(),
        "rsaquo" => "\u{203A}".to_string(),
        "times" => "\u{00D7}".to_string(),
        "divide" => "\u{00F7}".to_string(),
        "deg" => "\u{00B0}".to_string(),
        "plusmn" => "\u{00B1}".to_string(),
        "frac14" => "\u{00BC}".to_string(),
        "frac12" => "\u{00BD}".to_string(),
        "frac34" => "\u{00BE}".to_string(),

        // Additional common HTML entities
        "Agrave" => "À".to_string(),
        "Aacute" => "Á".to_string(),
        "Acirc" => "Â".to_string(),
        "Atilde" => "Ã".to_string(),
        "Auml" => "Ä".to_string(),
        "Aring" => "Å".to_string(),
        "AElig" => "Æ".to_string(),
        "Ccedil" => "Ç".to_string(),
        "Egrave" => "È".to_string(),
        "Eacute" => "É".to_string(),
        "Ecirc" => "Ê".to_string(),
        "Euml" => "Ë".to_string(),
        "Igrave" => "Ì".to_string(),
        "Iacute" => "Í".to_string(),
        "Icirc" => "Î".to_string(),
        "Iuml" => "Ï".to_string(),
        "ETH" => "Ð".to_string(),
        "Ntilde" => "Ñ".to_string(),
        "Ograve" => "Ò".to_string(),
        "Oacute" => "Ó".to_string(),
        "Ocirc" => "Ô".to_string(),
        "Otilde" => "Õ".to_string(),
        "Ouml" => "Ö".to_string(),
        "Oslash" => "Ø".to_string(),
        "Ugrave" => "Ù".to_string(),
        "Uacute" => "Ú".to_string(),
        "Ucirc" => "Û".to_string(),
        "Uuml" => "Ü".to_string(),
        "Yacute" => "Ý".to_string(),
        "THORN" => "Þ".to_string(),
        "szlig" => "ß".to_string(),
        "agrave" => "à".to_string(),
        "aacute" => "á".to_string(),
        "acirc" => "â".to_string(),
        "atilde" => "ã".to_string(),
        "auml" => "ä".to_string(),
        "aring" => "å".to_string(),
        "aelig" => "æ".to_string(),
        "ccedil" => "ç".to_string(),
        "egrave" => "è".to_string(),
        "eacute" => "é".to_string(),
        "ecirc" => "ê".to_string(),
        "euml" => "ë".to_string(),
        "igrave" => "ì".to_string(),
        "iacute" => "í".to_string(),
        "icirc" => "î".to_string(),
        "iuml" => "ï".to_string(),
        "eth" => "ð".to_string(),
        "ntilde" => "ñ".to_string(),
        "ograve" => "ò".to_string(),
        "oacute" => "ó".to_string(),
        "ocirc" => "ô".to_string(),
        "otilde" => "õ".to_string(),
        "ouml" => "ö".to_string(),
        "oslash" => "ø".to_string(),
        "ugrave" => "ù".to_string(),
        "uacute" => "ú".to_string(),
        "ucirc" => "û".to_string(),
        "uuml" => "ü".to_string(),
        "yacute" => "ý".to_string(),
        "thorn" => "þ".to_string(),
        "yuml" => "ÿ".to_string(),

        // Unsupported entity
        _ => {
            eprintln!("Warning: Unsupported entity reference: &{entity}; - replacing with �");
            "\u{FFFD}".to_string()
        }
    }
}

/// Represents different types of XML tokens
#[derive(Debug, Clone, PartialEq)]
enum Token {
    /// Start tag: <name attrs>
    StartTag {
        name: String,
        attrs: HashMap<String, String>,
        self_closing: bool,
    },
    /// End tag: </name>
    EndTag { name: String },
    /// Text content
    Text(String),
    /// Comment: <!-- content -->
    Comment(String),
    /// Processing instruction: <?target content?>
    ProcessingInstruction { target: String, content: String },
    /// DOCTYPE declaration
    DocType(String),
    /// CDATA section
    CData(String),
}

/// Simple XML tokenizer
struct Tokenizer {
    input: Vec<char>,
    pos: usize,
}

impl Tokenizer {
    fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += 1;
        Some(ch)
    }

    fn peek_slice(&self, n: usize) -> String {
        self.input.iter().skip(self.pos).take(n).collect()
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.advance();
        }
    }

    fn read_until(&mut self, delimiter: char) -> String {
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if ch == delimiter {
                break;
            }
            result.push(ch);
            self.advance();
        }
        result
    }

    fn read_until_str(&mut self, delimiter: &str) -> String {
        let mut result = String::new();
        let delim_chars: Vec<char> = delimiter.chars().collect();

        while self.pos < self.input.len() {
            if self.peek_slice(delim_chars.len()) == delimiter {
                break;
            }
            if let Some(ch) = self.advance() {
                result.push(ch);
            }
        }
        result
    }

    fn read_name(&mut self) -> String {
        let mut name = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == ':' || ch == '-' || ch == '_' || ch == '.' {
                name.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        name
    }

    fn parse_attributes(&mut self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();

        loop {
            self.skip_whitespace();

            // Check if we've reached the end of the tag
            if self.peek() == Some('>') || self.peek() == Some('/') {
                break;
            }

            if self.peek().is_none() {
                break;
            }

            // Read attribute name
            let name = self.read_name();
            if name.is_empty() {
                break;
            }

            self.skip_whitespace();

            // Expect '='
            if self.peek() != Some('=') {
                // Attribute without value (HTML-style), skip it for now
                continue;
            }
            self.advance(); // skip '='

            self.skip_whitespace();

            // Read attribute value
            let quote = self.peek();
            let value = if quote == Some('"') || quote == Some('\'') {
                self.advance(); // skip opening quote
                let val = self.read_until(quote.unwrap());
                self.advance(); // skip closing quote
                decode_entities(&val)
            } else {
                // Unquoted attribute value
                let val = self.read_name();
                decode_entities(&val)
            };

            attrs.insert(name, value);
        }

        attrs
    }

    fn next_token(&mut self) -> Option<Token> {
        // Read text content until we hit a '<'
        if self.peek() != Some('<') {
            let text = self.read_until('<');
            if !text.is_empty() {
                let decoded = decode_entities(&text);
                return Some(Token::Text(decoded));
            }
        }

        if self.peek() != Some('<') {
            return None;
        }

        self.advance(); // skip '<'

        // Check what kind of tag this is
        let next = self.peek()?;

        if next == '!' {
            self.advance(); // skip '!'

            // Check for comment
            if self.peek_slice(2) == "--" {
                self.advance(); // skip first '-'
                self.advance(); // skip second '-'
                let content = self.read_until_str("-->");
                // Skip the closing -->
                self.advance(); // '-'
                self.advance(); // '-'
                self.advance(); // '>'
                return Some(Token::Comment(content));
            }

            // Check for CDATA
            if self.peek_slice(7) == "[CDATA[" {
                for _ in 0..7 {
                    self.advance();
                }
                let content = self.read_until_str("]]>");
                // Skip the closing ]]>
                self.advance(); // ']'
                self.advance(); // ']'
                self.advance(); // '>'
                return Some(Token::CData(content));
            }

            // Check for DOCTYPE
            if self.peek_slice(7) == "DOCTYPE" {
                for _ in 0..7 {
                    self.advance();
                }
                let content = self.read_until('>');
                self.advance(); // skip '>'
                return Some(Token::DocType(content));
            }

            // Unknown special tag, skip it
            self.read_until('>');
            self.advance();
            return self.next_token();
        }

        if next == '?' {
            self.advance(); // skip '?'
            let target = self.read_name();
            let content = self.read_until_str("?>");
            // Skip the closing ?>
            self.advance(); // '?'
            self.advance(); // '>'
            return Some(Token::ProcessingInstruction { target, content });
        }

        if next == '/' {
            self.advance(); // skip '/'
            let name = self.read_name();
            self.skip_whitespace();
            self.advance(); // skip '>'
            return Some(Token::EndTag { name });
        }

        // Regular start tag
        let name = self.read_name();
        let attrs = self.parse_attributes();

        self.skip_whitespace();

        // Check for self-closing tag
        let self_closing = if self.peek() == Some('/') {
            self.advance(); // skip '/'
            true
        } else {
            false
        };

        self.advance(); // skip '>'

        Some(Token::StartTag {
            name,
            attrs,
            self_closing,
        })
    }

    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            tokens.push(token);
        }
        tokens
    }
}

/// Represents an XML node in the document tree
#[derive(Debug, Clone)]
struct XmlNode {
    /// Tag name (or synthetic name for special nodes)
    tag: String,
    /// Attributes on this node
    attrs: HashMap<String, String>,
    /// Text content before any child elements
    text: Option<String>,
    /// Text content after closing tag
    tail: String,
    /// Index among siblings
    index: usize,
    /// Whether this is a self-closing tag
    self_closing: bool,
    /// Child nodes
    children: Vec<XmlNode>,
}

/// Parse tokens into a tree structure
fn parse_tokens(tokens: Vec<Token>) -> Vec<XmlNode> {
    let mut root_nodes = Vec::new();
    let mut stack: Vec<XmlNode> = Vec::new();
    let mut root_index = 0;

    for token in tokens {
        match token {
            Token::StartTag {
                name,
                attrs,
                self_closing,
            } => {
                let node = XmlNode {
                    tag: name.clone(),
                    attrs,
                    text: if self_closing {
                        None
                    } else {
                        Some(String::new())
                    },
                    tail: String::new(),
                    index: 0, // Will be set later
                    self_closing,
                    children: Vec::new(),
                };

                if self_closing {
                    // Add as child to current parent or as root
                    if let Some(parent) = stack.last_mut() {
                        let idx = parent.children.len();
                        let mut node = node;
                        node.index = idx;
                        parent.children.push(node);
                    } else {
                        let mut node = node;
                        node.index = root_index;
                        root_index += 1;
                        root_nodes.push(node);
                    }
                } else {
                    stack.push(node);
                }
            }

            Token::EndTag { name: _ } => {
                if let Some(mut node) = stack.pop() {
                    // Set index and add to parent or root
                    if let Some(parent) = stack.last_mut() {
                        node.index = parent.children.len();
                        parent.children.push(node);
                    } else {
                        node.index = root_index;
                        root_index += 1;
                        root_nodes.push(node);
                    }
                }
            }

            Token::Text(text) => {
                if let Some(parent) = stack.last_mut() {
                    // If we have children, this is tail text for the last child
                    if let Some(last_child) = parent.children.last_mut() {
                        last_child.tail.push_str(&text);
                    } else {
                        // This is text content for the parent
                        if let Some(ref mut parent_text) = parent.text {
                            parent_text.push_str(&text);
                        }
                    }
                } else if let Some(last_root) = root_nodes.last_mut() {
                    // Tail text for last root node
                    last_root.tail.push_str(&text);
                }
            }

            Token::Comment(content) => {
                let node = XmlNode {
                    tag: "!--".to_string(),
                    attrs: HashMap::new(),
                    text: Some(content),
                    tail: String::new(),
                    index: 0,
                    self_closing: false,
                    children: Vec::new(),
                };

                if let Some(parent) = stack.last_mut() {
                    let idx = parent.children.len();
                    let mut node = node;
                    node.index = idx;
                    parent.children.push(node);
                } else {
                    let mut node = node;
                    node.index = root_index;
                    root_index += 1;
                    root_nodes.push(node);
                }
            }

            Token::ProcessingInstruction { target, content } => {
                let node = XmlNode {
                    tag: format!("?{target}"),
                    attrs: HashMap::new(),
                    text: Some(content),
                    tail: String::new(),
                    index: 0,
                    self_closing: false,
                    children: Vec::new(),
                };

                if let Some(parent) = stack.last_mut() {
                    let idx = parent.children.len();
                    let mut node = node;
                    node.index = idx;
                    parent.children.push(node);
                } else {
                    let mut node = node;
                    node.index = root_index;
                    root_index += 1;
                    root_nodes.push(node);
                }
            }

            Token::DocType(content) => {
                let node = XmlNode {
                    tag: "!DOCTYPE".to_string(),
                    attrs: HashMap::new(),
                    text: Some(content),
                    tail: String::new(),
                    index: 0,
                    self_closing: false,
                    children: Vec::new(),
                };

                let mut node = node;
                node.index = root_index;
                root_index += 1;
                root_nodes.push(node);
            }

            Token::CData(content) => {
                let node = XmlNode {
                    tag: "![CDATA[".to_string(),
                    attrs: HashMap::new(),
                    text: Some(content),
                    tail: String::new(),
                    index: 0,
                    self_closing: false,
                    children: Vec::new(),
                };

                if let Some(parent) = stack.last_mut() {
                    let idx = parent.children.len();
                    let mut node = node;
                    node.index = idx;
                    parent.children.push(node);
                } else {
                    let mut node = node;
                    node.index = root_index;
                    root_index += 1;
                    root_nodes.push(node);
                }
            }
        }
    }

    root_nodes
}

/// Ancestor context for building JSON objects
#[derive(Debug, Clone)]
struct AncestorContext {
    /// Tag names of ancestors (parent first, then grandparent, etc.)
    tags: Vec<String>,
    /// Attributes of ancestors, parallel to tags
    attrs: Vec<HashMap<String, String>>,
}

impl AncestorContext {
    const fn new() -> Self {
        Self {
            tags: Vec::new(),
            attrs: Vec::new(),
        }
    }

    fn push(&mut self, tag: String, attrs: HashMap<String, String>) {
        self.tags.insert(0, tag);
        self.attrs.insert(0, attrs);
    }

    fn pop(&mut self) {
        if !self.tags.is_empty() {
            self.tags.remove(0);
            self.attrs.remove(0);
        }
    }
}

/// Convert a node to a JSON object string with ancestor context
fn node_to_json(node: &XmlNode, ancestors: &AncestorContext) -> String {
    use core::fmt::Write;

    let mut json = String::from("{");

    // Add tag name with empty string key
    write!(
        &mut json,
        "\"\":{}",
        serde_json::to_string(&node.tag).unwrap()
    )
    .unwrap();

    // Add ancestor tags
    for (depth, tag) in ancestors.tags.iter().enumerate() {
        let prefix = "-".repeat(depth + 1);
        write!(
            &mut json,
            ",\"{}\":{}",
            prefix,
            serde_json::to_string(tag).unwrap()
        )
        .unwrap();
    }

    // Add ancestor attributes
    for (depth, attrs) in ancestors.attrs.iter().enumerate() {
        let prefix = "-".repeat(depth + 1);
        for (key, value) in attrs {
            write!(
                &mut json,
                ",\"{}{}\":{}",
                prefix,
                key,
                serde_json::to_string(value).unwrap()
            )
            .unwrap();
        }
    }

    // Add current node's attributes
    for (key, value) in &node.attrs {
        write!(
            &mut json,
            ",\"{}\":{}",
            key,
            serde_json::to_string(value).unwrap()
        )
        .unwrap();
    }

    // Add @text if present
    if let Some(ref text) = node.text {
        write!(
            &mut json,
            ",\"@text\":{}",
            serde_json::to_string(text).unwrap()
        )
        .unwrap();
    }

    // Add @tail
    write!(
        &mut json,
        ",\"@tail\":{}",
        serde_json::to_string(&node.tail).unwrap()
    )
    .unwrap();

    // Add @index
    write!(&mut json, ",\"@index\":{}", node.index).unwrap();

    json.push('}');
    json
}

/// Convert nodes to JSON Lines output, recursively
fn nodes_to_jsonlines(
    nodes: &[XmlNode],
    ancestors: &mut AncestorContext,
    output: &mut Vec<String>,
) {
    for node in nodes {
        // Output this node
        output.push(node_to_json(node, ancestors));

        // Process children
        if !node.children.is_empty() {
            ancestors.push(node.tag.clone(), node.attrs.clone());
            nodes_to_jsonlines(&node.children, ancestors, output);
            ancestors.pop();
        }
    }
}

/// Convert XML string to JSON Lines
#[must_use]
pub fn xml_to_jsonlines(xml: &str) -> Vec<String> {
    let mut tokenizer = Tokenizer::new(xml);
    let tokens = tokenizer.tokenize();
    let nodes = parse_tokens(tokens);

    let mut output = Vec::new();
    let mut ancestors = AncestorContext::new();
    nodes_to_jsonlines(&nodes, &mut ancestors, &mut output);

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_1_simple_nested() {
        let xml = r#"<html lang="en">
  <body id="main">
    <div class="content">Hello</div>
  </body>
</html>"#;

        let result = xml_to_jsonlines(xml);

        // Should have 3 nodes
        assert_eq!(result.len(), 3);

        // Check html node
        assert!(result[0].contains("\"\":\"html\""));
        assert!(result[0].contains("\"lang\":\"en\""));
        assert!(result[0].contains("\"@index\":0"));

        // Check body node
        assert!(result[1].contains("\"\":\"body\""));
        assert!(result[1].contains("\"-\":\"html\""));
        assert!(result[1].contains("\"-lang\":\"en\""));
        assert!(result[1].contains("\"id\":\"main\""));

        // Check div node
        assert!(result[2].contains("\"\":\"div\""));
        assert!(result[2].contains("\"-\":\"body\""));
        assert!(result[2].contains("\"--\":\"html\""));
        assert!(result[2].contains("\"-id\":\"main\""));
        assert!(result[2].contains("\"--lang\":\"en\""));
        assert!(result[2].contains("\"class\":\"content\""));
        assert!(result[2].contains("\"@text\":\"Hello\""));
    }

    #[test]
    fn test_example_2_self_closing_vs_empty() {
        let xml = r#"<root>
  <self-closing/>
  <empty></empty>
</root>"#;

        let result = xml_to_jsonlines(xml);

        // Should have 3 nodes
        assert_eq!(result.len(), 3);

        // Self-closing should NOT have @text
        assert!(!result[1].contains("@text"));
        assert!(result[1].contains("\"\":\"self-closing\""));
        assert!(result[1].contains("\"@index\":0"));

        // Empty tag should have @text: ""
        assert!(result[2].contains("\"@text\":\"\""));
        assert!(result[2].contains("\"\":\"empty\""));
        assert!(result[2].contains("\"@index\":1"));
    }

    #[test]
    fn test_example_3_mixed_content() {
        let xml = r#"<p>Text before <em>emphasis</em> text after</p>"#;

        let result = xml_to_jsonlines(xml);

        // Should have 2 nodes
        assert_eq!(result.len(), 2);

        // p element
        assert!(result[0].contains("\"\":\"p\""));
        assert!(result[0].contains("\"@text\":\"Text before \""));

        // em element
        assert!(result[1].contains("\"\":\"em\""));
        assert!(result[1].contains("\"@text\":\"emphasis\""));
        assert!(result[1].contains("\"@tail\":\" text after\""));
    }

    #[test]
    fn test_example_4_special_nodes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<!-- This is a comment -->
<root><![CDATA[Some <data>]]></root>"#;

        let result = xml_to_jsonlines(xml);

        // Should have 5 nodes
        assert_eq!(result.len(), 5);

        // XML declaration
        assert!(result[0].contains("\"\":\"?xml\""));
        assert!(result[0].contains("\"@index\":0"));

        // DOCTYPE
        assert!(result[1].contains("\"\":\"!DOCTYPE\""));
        assert!(result[1].contains("\"@index\":1"));

        // Comment
        assert!(result[2].contains("\"\":\"!--\""));
        assert!(result[2].contains("\"@text\":\" This is a comment \""));
        assert!(result[2].contains("\"@index\":2"));

        // root
        assert!(result[3].contains("\"\":\"root\""));
        assert!(result[3].contains("\"@index\":3"));

        // CDATA
        assert!(result[4].contains("\"\":\"![CDATA[\""));
        assert!(result[4].contains("\"@text\":\"Some <data>\""));
        assert!(result[4].contains("\"-\":\"root\""));
    }

    #[test]
    fn test_example_5_multiple_root_level() {
        let xml = r#"<!-- Comment 1 -->
<!-- Comment 2 -->
<root/>
<!-- Comment 3 -->"#;

        let result = xml_to_jsonlines(xml);

        // Should have 4 nodes
        assert_eq!(result.len(), 4);

        // All should have sequential indices
        assert!(result[0].contains("\"@index\":0"));
        assert!(result[1].contains("\"@index\":1"));
        assert!(result[2].contains("\"@index\":2"));
        assert!(result[3].contains("\"@index\":3"));

        // root should be self-closing (no @text)
        assert!(!result[2].contains("@text"));
    }

    #[test]
    fn test_example_6_entity_references() {
        let xml = r#"<p>Standard: &lt;&gt;&amp; Numeric: &#65; HTML: &nbsp; Custom: &custom;</p>"#;

        let result = xml_to_jsonlines(xml);

        assert_eq!(result.len(), 1);

        // Check decoded entities
        assert!(result[0].contains("\"@text\":\"Standard: <>& Numeric: A HTML: "));
        // Note: nbsp is non-breaking space U+00A0, and custom entity becomes
        // replacement char
        assert!(result[0].contains("�"));
    }

    #[test]
    fn test_example_7_deep_nesting() {
        let xml = r#"<a id="1"><b id="2"><c id="3"><d id="4">text</d></c></b></a>"#;

        let result = xml_to_jsonlines(xml);

        // Should have 4 nodes
        assert_eq!(result.len(), 4);

        // a
        assert!(result[0].contains("\"\":\"a\""));
        assert!(result[0].contains("\"id\":\"1\""));

        // b
        assert!(result[1].contains("\"\":\"b\""));
        assert!(result[1].contains("\"-\":\"a\""));
        assert!(result[1].contains("\"-id\":\"1\""));
        assert!(result[1].contains("\"id\":\"2\""));

        // c
        assert!(result[2].contains("\"\":\"c\""));
        assert!(result[2].contains("\"-\":\"b\""));
        assert!(result[2].contains("\"--\":\"a\""));
        assert!(result[2].contains("\"-id\":\"2\""));
        assert!(result[2].contains("\"--id\":\"1\""));
        assert!(result[2].contains("\"id\":\"3\""));

        // d (deepest)
        assert!(result[3].contains("\"\":\"d\""));
        assert!(result[3].contains("\"-\":\"c\""));
        assert!(result[3].contains("\"--\":\"b\""));
        assert!(result[3].contains("\"---\":\"a\""));
        assert!(result[3].contains("\"-id\":\"3\""));
        assert!(result[3].contains("\"--id\":\"2\""));
        assert!(result[3].contains("\"---id\":\"1\""));
        assert!(result[3].contains("\"id\":\"4\""));
        assert!(result[3].contains("\"@text\":\"text\""));
    }

    #[test]
    fn test_whitespace_preservation() {
        let xml = "<root>  \n  text  \n  </root>";

        let result = xml_to_jsonlines(xml);

        assert_eq!(result.len(), 1);
        // Whitespace should be preserved exactly
        assert!(result[0].contains("\"@text\":\"  \\n  text  \\n  \""));
    }

    #[test]
    fn test_entity_in_attribute() {
        let xml = r#"<div title="&lt;Hello&gt;">text</div>"#;

        let result = xml_to_jsonlines(xml);

        assert_eq!(result.len(), 1);
        // Entities in attributes should be decoded
        assert!(result[0].contains("\"title\":\"<Hello>\""));
    }
}
