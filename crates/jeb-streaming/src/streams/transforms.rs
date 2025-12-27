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
