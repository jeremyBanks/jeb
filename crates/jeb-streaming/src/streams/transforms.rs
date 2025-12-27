use {
    crate::Item,
    async_stream::stream,
    futures::{Stream, StreamExt},
    std::pin::pin,
};

/// Transforms a stream of Items by splitting after occurrences of a pattern.
///
/// Handles both `Item::Text` and `Item::Bytes`, buffering until complete
/// segments (including the pattern) are available. Flushes remaining buffers at stream end.
///
/// The pattern must be a valid UTF-8 string to ensure we never break UTF-8 boundaries.
pub fn split_after<S>(input: S, pattern: &str) -> impl Stream<Item = Result<Item, &'static str>> + Send
where
    S: Stream<Item = Result<Item, &'static str>> + Send + 'static,
{
    let pattern = pattern.to_string();
    let pattern_bytes = pattern.as_bytes().to_vec();

    stream! {
        let mut input = pin!(input);
        let mut text_buffer = String::new();
        let mut bytes_buffer = Vec::<u8>::new();

        while let Some(result) = input.next().await {
            match result {
                Ok(item) => {
                    match item {
                        Item::Text(text) => {
                            // Flush bytes buffer if switching types
                            if !bytes_buffer.is_empty() {
                                yield Ok(Item::Bytes(std::mem::take(&mut bytes_buffer).into()));
                            }

                            // Add to text buffer and split on pattern
                            text_buffer.push_str(&text);

                            while let Some(pattern_pos) = text_buffer.find(&pattern) {
                                let end_pos = pattern_pos + pattern.len() - 1;
                                let segment = text_buffer[..=end_pos].to_string();
                                text_buffer.drain(..=end_pos);
                                yield Ok(Item::Text(segment.into()));
                            }
                        }
                        Item::Bytes(bytes) => {
                            // Flush text buffer if switching types
                            if !text_buffer.is_empty() {
                                yield Ok(Item::Text(std::mem::take(&mut text_buffer).into()));
                            }

                            // Add to bytes buffer and split on pattern bytes
                            bytes_buffer.extend_from_slice(&bytes);

                            while let Some(pattern_pos) = bytes_buffer.windows(pattern_bytes.len())
                                .position(|window| window == pattern_bytes.as_slice()) {
                                let end_pos = pattern_pos + pattern_bytes.len() - 1;
                                let segment: Vec<u8> = bytes_buffer.drain(..=end_pos).collect();
                                yield Ok(Item::Bytes(segment.into()));
                            }
                        }
                        other => {
                            // Flush both buffers before passing through
                            if !text_buffer.is_empty() {
                                yield Ok(Item::Text(std::mem::take(&mut text_buffer).into()));
                            }
                            if !bytes_buffer.is_empty() {
                                yield Ok(Item::Bytes(std::mem::take(&mut bytes_buffer).into()));
                            }

                            // Pass through other item types
                            yield Ok(other);
                        }
                    }
                }
                Err(e) => {
                    // Flush both buffers before passing through error
                    if !text_buffer.is_empty() {
                        yield Ok(Item::Text(std::mem::take(&mut text_buffer).into()));
                    }
                    if !bytes_buffer.is_empty() {
                        yield Ok(Item::Bytes(std::mem::take(&mut bytes_buffer).into()));
                    }

                    // Pass through error
                    yield Err(e);
                }
            }
        }

        // Flush remaining buffers at end of stream
        if !text_buffer.is_empty() {
            yield Ok(Item::Text(text_buffer.into()));
        }
        if !bytes_buffer.is_empty() {
            yield Ok(Item::Bytes(bytes_buffer.into()));
        }
    }
}

/// Transforms a stream of Items into lines, splitting on newline characters.
///
/// Handles both `Item::Text` and `Item::Bytes`, buffering until complete lines
/// are available. Flushes remaining buffers at stream end.
pub fn lines<S>(input: S) -> impl Stream<Item = Result<Item, &'static str>> + Send
where
    S: Stream<Item = Result<Item, &'static str>> + Send + 'static,
{
    split_after(input, "\n")
}

/// Transforms a stream of Items into fixed-size chunks.
///
/// For `Item::Text`, chunks by character count. For `Item::Bytes`, chunks by byte count.
/// If `length` is 0, defaults to 65536.
pub fn chunks<S>(input: S, length: usize) -> impl Stream<Item = Result<Item, &'static str>> + Send
where
    S: Stream<Item = Result<Item, &'static str>> + Send + 'static,
{
    let chunk_size = if length == 0 { 65536 } else { length };

    stream! {
        let mut input = pin!(input);
        let mut text_buffer = String::new();
        let mut bytes_buffer = Vec::<u8>::new();

        while let Some(result) = input.next().await {
            match result {
                Ok(item) => {
                    match item {
                        Item::Text(text) => {
                            // Flush bytes buffer if switching types
                            if !bytes_buffer.is_empty() {
                                yield Ok(Item::Bytes(std::mem::take(&mut bytes_buffer).into()));
                            }

                            // Add to text buffer and chunk by character count
                            text_buffer.push_str(&text);

                            while text_buffer.chars().count() >= chunk_size {
                                let chunk_chars: String = text_buffer.chars().take(chunk_size).collect();
                                let byte_len = chunk_chars.len();
                                text_buffer.drain(..byte_len);
                                yield Ok(Item::Text(chunk_chars.into()));
                            }
                        }
                        Item::Bytes(bytes) => {
                            // Flush text buffer if switching types
                            if !text_buffer.is_empty() {
                                yield Ok(Item::Text(std::mem::take(&mut text_buffer).into()));
                            }

                            // Add to bytes buffer and chunk by byte count
                            bytes_buffer.extend_from_slice(&bytes);

                            while bytes_buffer.len() >= chunk_size {
                                let chunk: Vec<u8> = bytes_buffer.drain(..chunk_size).collect();
                                yield Ok(Item::Bytes(chunk.into()));
                            }
                        }
                        other => {
                            // Flush both buffers before passing through
                            if !text_buffer.is_empty() {
                                yield Ok(Item::Text(std::mem::take(&mut text_buffer).into()));
                            }
                            if !bytes_buffer.is_empty() {
                                yield Ok(Item::Bytes(std::mem::take(&mut bytes_buffer).into()));
                            }

                            // Pass through other item types
                            yield Ok(other);
                        }
                    }
                }
                Err(e) => {
                    // Flush both buffers before passing through error
                    if !text_buffer.is_empty() {
                        yield Ok(Item::Text(std::mem::take(&mut text_buffer).into()));
                    }
                    if !bytes_buffer.is_empty() {
                        yield Ok(Item::Bytes(std::mem::take(&mut bytes_buffer).into()));
                    }

                    // Pass through error
                    yield Err(e);
                }
            }
        }

        // Flush remaining buffers at end of stream
        if !text_buffer.is_empty() {
            yield Ok(Item::Text(text_buffer.into()));
        }
        if !bytes_buffer.is_empty() {
            yield Ok(Item::Bytes(bytes_buffer.into()));
        }
    }
}
/// Transforms a stream of Items by converting bytes to lowercase hexadecimal string representation.
///
/// Handles both `Item::Text` and `Item::Bytes`, treating text as UTF-8 bytes.
/// Each byte is converted to a two-character hex string (e.g., `0xDE` → `"de"`).
pub fn to_hex<S>(input: S) -> impl Stream<Item = Result<Item, &'static str>> + Send
where
    S: Stream<Item = Result<Item, &'static str>> + Send + 'static,
{
    stream! {
        let mut input = pin!(input);

        while let Some(result) = input.next().await {
            match result {
                Ok(item) => {
                    match item {
                        Item::Text(text) => {
                            // Convert text to bytes, then to hex
                            let bytes = text.as_bytes();
                            let mut hex = String::with_capacity(bytes.len() * 2);
                            for byte in bytes {
                                hex.push_str(&format!("{:02x}", byte));
                            }
                            yield Ok(Item::Text(hex.into()));
                        }
                        Item::Bytes(bytes) => {
                            // Convert bytes to hex
                            let mut hex = String::with_capacity(bytes.len() * 2);
                            for byte in bytes.iter() {
                                hex.push_str(&format!("{:02x}", byte));
                            }
                            yield Ok(Item::Text(hex.into()));
                        }
                        other => {
                            // Pass through other item types unchanged
                            yield Ok(other);
                        }
                    }
                }
                Err(e) => {
                    // Pass through error
                    yield Err(e);
                }
            }
        }
    }
}

/// Transforms a stream of Items by parsing hexadecimal strings into bytes.
///
/// Handles both `Item::Text` and `Item::Bytes` (treating bytes as ASCII hex).
/// Filters out whitespace before parsing. Returns errors for invalid hex.
pub fn parse_hex<S>(input: S) -> impl Stream<Item = Result<Item, &'static str>> + Send
where
    S: Stream<Item = Result<Item, &'static str>> + Send + 'static,
{
    fn hex_digit_to_value(digit: u8) -> Result<u8, &'static str> {
        match digit {
            b'0'..=b'9' => Ok(digit - b'0'),
            b'a'..=b'f' => Ok(digit - b'a' + 10),
            b'A'..=b'F' => Ok(digit - b'A' + 10),
            _ => Err("invalid hex digit"),
        }
    }

    stream! {
        let mut input = pin!(input);

        while let Some(result) = input.next().await {
            match result {
                Ok(item) => {
                    match item {
                        Item::Text(text) => {
                            // Parse hex string to bytes
                            let hex_bytes: Vec<u8> = text.as_bytes()
                                .iter()
                                .filter(|&&b| !b.is_ascii_whitespace())
                                .copied()
                                .collect();

                            if hex_bytes.len() % 2 != 0 {
                                yield Err("odd number of hex digits");
                                continue;
                            }

                            let mut bytes = Vec::new();
                            let mut has_error = false;
                            for chunk in hex_bytes.chunks(2) {
                                match (hex_digit_to_value(chunk[0]), hex_digit_to_value(chunk[1])) {
                                    (Ok(high), Ok(low)) => bytes.push((high << 4) | low),
                                    _ => {
                                        yield Err("invalid hex digit");
                                        has_error = true;
                                        break;
                                    }
                                }
                            }
                            if !has_error {
                                yield Ok(Item::Bytes(bytes.into()));
                            }
                        }
                        Item::Bytes(bytes) => {
                            // Treat bytes as ASCII hex, parse
                            let hex_bytes: Vec<u8> = bytes
                                .iter()
                                .filter(|&&b| !b.is_ascii_whitespace())
                                .copied()
                                .collect();

                            if hex_bytes.len() % 2 != 0 {
                                yield Err("odd number of hex digits");
                                continue;
                            }

                            let mut result_bytes = Vec::new();
                            let mut has_error = false;
                            for chunk in hex_bytes.chunks(2) {
                                match (hex_digit_to_value(chunk[0]), hex_digit_to_value(chunk[1])) {
                                    (Ok(high), Ok(low)) => result_bytes.push((high << 4) | low),
                                    _ => {
                                        yield Err("invalid hex digit");
                                        has_error = true;
                                        break;
                                    }
                                }
                            }
                            if !has_error {
                                yield Ok(Item::Bytes(result_bytes.into()));
                            }
                        }
                        other => {
                            // Pass through other item types unchanged
                            yield Ok(other);
                        }
                    }
                }
                Err(e) => {
                    // Pass through error
                    yield Err(e);
                }
            }
        }
    }
}

/// Transforms a stream of Items by splitting on ASCII whitespace with cross-chunk buffering.
///
/// Handles both `Item::Text` and `Item::Bytes`, buffering until complete segments are available.
/// Uses `u8::is_ascii_whitespace()` as delimiter. Consecutive whitespace is treated as a single
/// separator (no empty segments). Flushes remaining buffers at stream end.
pub fn split_whitespace<S>(input: S) -> impl Stream<Item = Result<Item, &'static str>> + Send
where
    S: Stream<Item = Result<Item, &'static str>> + Send + 'static,
{
    stream! {
        let mut input = pin!(input);
        let mut text_buffer = String::new();
        let mut bytes_buffer = Vec::<u8>::new();

        while let Some(result) = input.next().await {
            match result {
                Ok(item) => {
                    match item {
                        Item::Text(text) => {
                            // Flush bytes buffer if switching types
                            if !bytes_buffer.is_empty() {
                                yield Ok(Item::Bytes(std::mem::take(&mut bytes_buffer).into()));
                            }

                            // Process text, buffering and splitting on whitespace
                            for ch in text.chars() {
                                if ch.is_ascii_whitespace() {
                                    if !text_buffer.is_empty() {
                                        yield Ok(Item::Text(std::mem::take(&mut text_buffer).into()));
                                    }
                                } else {
                                    text_buffer.push(ch);
                                }
                            }
                        }
                        Item::Bytes(bytes) => {
                            // Flush text buffer if switching types
                            if !text_buffer.is_empty() {
                                yield Ok(Item::Text(std::mem::take(&mut text_buffer).into()));
                            }

                            // Process bytes, buffering and splitting on whitespace
                            for &byte in bytes.iter() {
                                if byte.is_ascii_whitespace() {
                                    if !bytes_buffer.is_empty() {
                                        yield Ok(Item::Bytes(std::mem::take(&mut bytes_buffer).into()));
                                    }
                                } else {
                                    bytes_buffer.push(byte);
                                }
                            }
                        }
                        other => {
                            // Flush both buffers before passing through
                            if !text_buffer.is_empty() {
                                yield Ok(Item::Text(std::mem::take(&mut text_buffer).into()));
                            }
                            if !bytes_buffer.is_empty() {
                                yield Ok(Item::Bytes(std::mem::take(&mut bytes_buffer).into()));
                            }
                            yield Ok(other);
                        }
                    }
                }
                Err(e) => {
                    // Flush both buffers before passing through error
                    if !text_buffer.is_empty() {
                        yield Ok(Item::Text(std::mem::take(&mut text_buffer).into()));
                    }
                    if !bytes_buffer.is_empty() {
                        yield Ok(Item::Bytes(std::mem::take(&mut bytes_buffer).into()));
                    }
                    yield Err(e);
                }
            }
        }

        // Flush remaining buffers at end of stream
        if !text_buffer.is_empty() {
            yield Ok(Item::Text(text_buffer.into()));
        }
        if !bytes_buffer.is_empty() {
            yield Ok(Item::Bytes(bytes_buffer.into()));
        }
    }
}
